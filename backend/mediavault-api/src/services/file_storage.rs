//! ファイルサーバー書込サービス（TASK-0027: POST /items/:id/files/upload）
//!
//! multipart受信したバイナリを`file_type`に応じたベースディレクトリへ一意なファイル名で書き込み、
//! ベースディレクトリからの相対パスを返す。書込失敗時のロールバック（DBレコード未作成）・
//! パストラバーサル防止・コンテナ内非保存（REQ-402）を担う。
//!
//! 設計決定（テストケース定義書 第8章の未確定事項をRed着手前に確定）:
//! 1. file_type="photo" は既存 FileType::Image の表記ゆれとして扱う（enum拡張なし）。
//! 2. 保存先は `STORAGE_ROOT` 配下に `FileType` ごとのサブディレクトリを持つ構成とし、
//!    6種すべてを別ディレクトリへ分ける（`resolve_base_dir` / `subdir_spec` 参照）。
//!    この領域は MediaVault-api だけが書き込む「MediaVault専用領域」であり、
//!    読み取り専用のアニメ・実写・マンガ領域とは分離する。
//! 3. 書込失敗時は `ApiErrorCode::FileStorageWriteFailed`（500）を返す（models/response.rsに追加済み）。
//! 4. 書込失敗注入は `FileWriter` トレイトで行う。本番は `TokioFileWriter`、テストでは
//!    常に失敗する `FailingFileWriter` を注入する（DIフレームワーク不使用、trait objectで十分）。
//! 5. ボディサイズ上限はルーター層の `DefaultBodyLimit` で対応する前提とし、本サービスでは未テスト。
//! 6. 空（0バイト）ファイルのアップロードは許容する（特別な拒否を行わない）。

use std::io;
use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::models::item_file::FileType;
use crate::models::response::{ApiError, ApiErrorCode};

/// アップロード保存先ルートの環境変数名。MediaVault専用領域を指す。
const STORAGE_ROOT_ENV: &str = "STORAGE_ROOT";

/// `STORAGE_ROOT` 未設定時の既定値。ホストとコンテナで同じパスを使う
/// MediaVault専用領域（ホスト側 `/srv/mediavault`）を指す。カレントディレクトリを
/// 指さないことでコンテナ内非保存（REQ-402）を担保する。
const DEFAULT_STORAGE_ROOT: &str = "/srv/mediavault";

/// `FileType` ごとの、サブディレクトリ上書き用環境変数名と既定サブディレクトリ名を返す。
///
/// 保存先は「ルート1本 + 種別サブディレクトリ」で構成する。種別を第1階層に置くことで、
/// mergerfs の `category.create=epmfs` が種別単位で物理ディスクをまとめてくれる
/// （大きい動画と小さい画像が同じディスクを奪い合わない）。
fn subdir_spec(file_type: FileType) -> (&'static str, &'static str) {
    match file_type {
        FileType::Pdf => ("STORAGE_SUBDIR_PDF", "pdf"),
        FileType::Image => ("STORAGE_SUBDIR_IMAGE", "image"),
        FileType::Video => ("STORAGE_SUBDIR_VIDEO", "video"),
        FileType::Audio => ("STORAGE_SUBDIR_AUDIO", "audio"),
        FileType::Archive => ("STORAGE_SUBDIR_ARCHIVE", "archive"),
        FileType::Other => ("STORAGE_SUBDIR_OTHER", "other"),
    }
}

/// サブディレクトリ環境変数の値を検証する。ルート配下に収まらない値（絶対パス・`..` を含む）は
/// ルート外への脱出になるため拒否し、`None` を返して既定値へフォールバックさせる。
/// `generate_object_name` がファイル名側のトラバーサルを潰しているのと対になる防御。
fn validate_subdir(env_name: &str, value: &str) -> Option<String> {
    use std::path::Component;

    if value.is_empty() {
        return None;
    }

    let path = Path::new(value);
    let is_safe = path
        .components()
        .all(|c| matches!(c, Component::Normal(_) | Component::CurDir));

    if is_safe {
        Some(value.to_string())
    } else {
        tracing::warn!(
            "{env_name} の値 {value:?} はルート外を指す可能性があるため無視し、既定値を使用します"
        );
        None
    }
}

/// 書込先ベースディレクトリを環境変数から解決する（デフォルト値つき）。
///
/// `STORAGE_ROOT`（MediaVault専用領域のルート）配下に、`FileType` ごとのサブディレクトリを
/// 結合したパスを返す。サブディレクトリは `STORAGE_SUBDIR_*` で上書きでき、
/// `video/2026` のようなネストした相対パスも許容する。
///
/// 【重要】ここが返すのはアップロードファイルの書込先（MediaVault専用領域）だけである。
/// 読み取り専用のアニメ・実写・マンガ領域へは MediaVault は書き込まず、
/// `POST /items/:id/files` による絶対パス登録（リンク）で参照するのみ。
/// 🟡 信頼性レベル: 要件定義書 第3章「API制約 / 環境変数」より
pub fn resolve_base_dir(file_type: FileType) -> PathBuf {
    let root = std::env::var(STORAGE_ROOT_ENV)
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| DEFAULT_STORAGE_ROOT.to_string());

    let (env_name, default_subdir) = subdir_spec(file_type);
    let subdir = std::env::var(env_name)
        .ok()
        .and_then(|v| validate_subdir(env_name, &v))
        .unwrap_or_else(|| default_subdir.to_string());

    PathBuf::from(root).join(subdir)
}

/// クライアント指定の元ファイル名から、サーバー側で一意なオブジェクト名（UUID + 元拡張子）を生成する。
/// 元のbasenameは破棄し、拡張子のみ継承する。`..`・パス区切りを含む拡張子は無害化する。
/// 🔵 信頼性レベル: 要件定義書 第3章セキュリティ要件（UUID + 元拡張子）・TC-019-U04に直接対応
pub fn generate_object_name(original_filename: &str) -> String {
    // 【実装内容】: クライアントが指定したファイル名のbasenameは信頼せず破棄する。
    // 拡張子部分のみ抽出し、さらに `..`・`/`・`\` を含む場合は無害化（除去）してから
    // UUID v4 + 拡張子の安全な名前を組み立てる 🔵
    let uuid = Uuid::new_v4();

    // 【拡張子抽出】: 最後の '.' 以降を拡張子候補とする。先頭がドットのみ（拡張子なし）の場合は無視する
    let extension = original_filename
        .rsplit_once('.')
        .map(|(_, ext)| ext)
        .filter(|ext| !ext.is_empty());

    match extension {
        Some(ext) => {
            // 【無害化】: パストラバーサル成分（".."・"/"・"\\"）を拡張子文字列から除去する 🟡
            let sanitized_ext: String = ext.chars().filter(|c| *c != '/' && *c != '\\').collect();
            let sanitized_ext = sanitized_ext.replace("..", "");

            if sanitized_ext.is_empty() {
                // 【境界対応】: 無害化後に拡張子が空になった場合は拡張子なし名を返す（panic回避） 🟡
                uuid.to_string()
            } else {
                format!("{uuid}.{sanitized_ext}")
            }
        }
        // 【境界対応】: 拡張子のない元ファイル名（TC-019-B03）はUUIDのみの名前とする 🟡
        None => uuid.to_string(),
    }
}

/// ファイル書込を抽象化するトレイト（書込失敗注入のためテストでフェイク実装を注入可能にする）。
/// 🟡 信頼性レベル: テストケース定義書 第0章・第5章「書込失敗注入」より新規設計
pub trait FileWriter: Send + Sync {
    /// `path` へ `bytes` を書き込む。失敗時は `io::Error` を返す。
    fn write(&self, path: &Path, bytes: &[u8]) -> Result<(), io::Error>;

    /// `path` のファイルを削除する（ロールバック用）。
    fn remove(&self, path: &Path) -> Result<(), io::Error>;
}

/// 本番用の実装。`tokio::fs`（同期コンテキストでは`std::fs`）でファイルシステムへ書き込む。
/// 🔵 信頼性レベル: 要件定義書 第3章「tokio::fsによるストリーミング書込」より
pub struct TokioFileWriter;

impl FileWriter for TokioFileWriter {
    fn write(&self, path: &Path, bytes: &[u8]) -> Result<(), io::Error> {
        // 【実装内容】: FileWriterトレイトは同期I/Fのため、書込先の親ディレクトリを作成した上で
        // std::fs（同期）を用いて書き込む。呼び出し元（store_file）は非同期文脈から
        // tokio::task::spawn_blocking等で呼ぶことを想定せず、ここでは単純にstd::fsで実装する 🔵
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, bytes)
    }

    fn remove(&self, path: &Path) -> Result<(), io::Error> {
        std::fs::remove_file(path)
    }
}

/// 書込失敗を常に注入するテスト用フェイク実装（TC-019-E01 / IT-019-01(b)）。
/// 🟡 信頼性レベル: テストケース定義書 第0章「書込失敗注入」に対応する新規ヘルパー
#[derive(Default)]
pub struct FailingFileWriter;

impl FileWriter for FailingFileWriter {
    fn write(&self, _path: &Path, _bytes: &[u8]) -> Result<(), io::Error> {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "simulated write failure (FailingFileWriter)",
        ))
    }

    fn remove(&self, _path: &Path) -> Result<(), io::Error> {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "simulated remove failure (FailingFileWriter)",
        ))
    }
}

/// 書込結果（ベースディレクトリ・絶対書込先・DBへ保存する相対パス）。
#[derive(Debug, Clone)]
pub struct StoredFile {
    pub base_dir: PathBuf,
    pub absolute_path: PathBuf,
    /// `item_files.path` に保存する、ベースディレクトリからの相対パス
    pub relative_path: String,
}

/// 【機能概要】: file_typeに応じたベースディレクトリへバイナリを書き込み、相対パスを返す。
/// 失敗時は `ApiErrorCode::FileStorageWriteFailed` を返す（DBレコードは作成させない）。
/// 🟡 信頼性レベル: 要件定義書 第2章データフロー・第3章セキュリティ要件・TC-019-01/02/E01/U05に対応
pub fn store_file(
    writer: &dyn FileWriter,
    file_type: FileType,
    original_filename: &str,
    bytes: &[u8],
) -> Result<StoredFile, ApiError> {
    // 【処理方針】: ベースディレクトリ解決→一意名生成→書込→相対パス算出の順に実行する。
    // 書込が失敗した場合はDBレコードを作らせないため、ここでApiError(FileStorageWriteFailed)
    // へ変換して返す（write-then-record順序保証） 🟡
    let base_dir = resolve_base_dir(file_type);
    let object_name = generate_object_name(original_filename);
    let absolute_path = base_dir.join(&object_name);

    // 【実処理実行】: FileWriterトレイト経由で実際の書込を行う
    writer.write(&absolute_path, bytes).map_err(|err| {
        // 【エラー捕捉】: I/Oエラー詳細（パス・権限等）はサーバーログのみに出力し、
        // クライアントには一般的なメッセージのみを返す（情報漏洩防止） 🟡
        tracing::error!("file_storage write failed: path={absolute_path:?}, error={err}");
        ApiError::new(
            ApiErrorCode::FileStorageWriteFailed,
            "ファイル書込に失敗しました",
        )
    })?;

    // 【相対パス算出】: item_files.pathにはベースディレクトリからの相対パスのみを保存する 🔵
    let relative_path = object_name;

    Ok(StoredFile {
        base_dir,
        absolute_path,
        relative_path,
    })
}

/// 【機能概要】: 既に書き込んだファイルを削除する（DB INSERT失敗時のロールバック、TC-019-E02/IT-019-02）。
/// 🟡 信頼性レベル: 要件定義書 第4章EDGE-003対称・TC-019-E02より
pub fn cleanup_file(writer: &dyn FileWriter, stored: &StoredFile) -> Result<(), io::Error> {
    // 【実装内容】: DB登録失敗時に書込済みファイルを削除し、孤立ファイルを残さない 🟡
    writer.remove(&stored.absolute_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// `STORAGE_ROOT` / `STORAGE_SUBDIR_*` はプロセス全体で共有される環境変数であり、
    /// cargo testはデフォルトで各`#[test]`関数を別スレッドで並列実行するため、
    /// 複数のテストが同時にこれらの環境変数をset_var/remove_varすると競合（flaky failure）が起きる。
    /// このMutexで当該環境変数を触るテスト全体を直列化し、テスト間の独立性を保証する。
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    /// テスト用に一時ベースディレクトリを作成する（`tempfile`クレート使用）。
    /// 🔵 信頼性レベル: テストケース定義書「テストヘルパー」`temp_storage_root()`に対応
    fn temp_storage_root() -> tempfile::TempDir {
        tempfile::tempdir().expect("一時ディレクトリの作成に失敗しました")
    }

    /// TC-019-U04: file_storageがUUID+元拡張子の一意ファイル名を生成する
    /// 🔵 信頼性レベル: 要件定義書 第3章セキュリティ要件・完了条件L29に直接対応
    #[test]
    fn generate_object_name_returns_uuid_with_original_extension() {
        // 【テスト目的】: クライアント指定filenameのbasenameを破棄し、UUID+元拡張子の名前を生成することを確認する
        // 【テスト内容】: 日本語名・大文字拡張子を含む複数のfilenameに対しgenerate_object_name()を呼ぶ
        // 【期待される動作】: 戻り値の拡張子が元拡張子と一致し、basename部がUuid::parse_strで解析可能
        // 🔵 信頼性レベル: 要件定義書 第3章・テストケース定義書TC-019-U04に直接対応

        // 【テストデータ準備】: 日本語名・空白入り・大文字拡張子という代表値を用意する
        // 【初期条件設定】: 特になし（純粋関数）
        let result = generate_object_name("お気に入り.pdf");

        // 【実際の処理実行】: generate_object_name()を呼び出す
        // 【処理内容】: 元basenameを破棄しUUID+拡張子の名前を生成するロジック（未実装のためtodo!()でpanicする想定）
        let (stem, ext) = result
            .rsplit_once('.')
            .expect("生成名は拡張子を含むはずです");

        // 【結果検証】: 拡張子がpdfであり、basenameがUUID形式であることを確認
        // 【期待値確認】: 元のbasename「お気に入り」が結果に混入していないこと
        assert_eq!(ext, "pdf"); // 【確認内容】: 元拡張子.pdfが保持されることを確認 🔵
        assert!(Uuid::parse_str(stem).is_ok()); // 【確認内容】: basename部がUUID形式であることを確認 🔵
    }

    /// TC-019-U04-b: generate_object_nameを2回呼んでも結果が衝突しない
    /// 🔵 信頼性レベル: 完了条件L29（名前衝突防止）に直接対応
    #[test]
    fn generate_object_name_does_not_collide_on_repeated_calls() {
        // 【テスト目的】: 同一入力に対して2回呼んでも異なる名前が生成されることを確認する
        // 【テスト内容】: 同じfilenameでgenerate_object_name()を2回呼び、結果を比較する
        // 【期待される動作】: 2回の戻り値が異なる（UUID部がランダム生成されるため）
        // 🔵 信頼性レベル: 完了条件L29「ファイル名の重複防止」に直接対応

        // 【実際の処理実行】: 同一filenameで2回呼び出す
        let first = generate_object_name("cover.JPG");
        let second = generate_object_name("cover.JPG");

        // 【結果検証】: 2回の生成名が異なることを確認
        // 【期待値確認】: UUIDがランダムであるため衝突しないこと
        assert_ne!(first, second); // 【確認内容】: 連続生成で同一名が返らないことを確認 🔵
    }

    /// TC-019-B03: 拡張子のない元ファイル名でも安全に一意名を生成する
    /// 🟡 信頼性レベル: 要件定義書 第3章（拡張子抽出時の正規化）からの妥当な推測
    #[test]
    fn generate_object_name_handles_filename_without_extension() {
        // 【テスト目的】: 拡張子を含まないfilenameでもpanicせず安全に処理できることを確認する
        // 【テスト内容】: "scan"（拡張子なし）でgenerate_object_name()を呼ぶ
        // 【期待される動作】: panicせず、戻り値の中にパストラバーサル成分（..や/）が含まれない
        // 🟡 信頼性レベル: 要件定義書 第3章からの妥当な推測（拡張子抽出ロジックの境界）

        // 【実際の処理実行】: 拡張子なしfilenameで呼び出す
        let result = generate_object_name("scan");

        // 【結果検証】: 親ディレクトリ参照・パス区切りを含まないことを確認
        // 【期待値確認】: 拡張子なし入力でも不正なパス成分が混入しないこと
        assert!(!result.contains("..")); // 【確認内容】: 親ディレクトリ参照が結果に含まれないことを確認 🟡
        assert!(!result.contains('/') && !result.contains('\\')); // 【確認内容】: パス区切り文字が結果に含まれないことを確認 🟡
    }

    /// TC-019-E06: ファイル名に親ディレクトリ参照（..）を含む値でもパストラバーサルを起こさない
    /// 🟡 信頼性レベル: 要件定義書 第3章セキュリティ（パストラバーサル防止）からの妥当な推測
    #[test]
    fn generate_object_name_neutralizes_path_traversal_in_filename() {
        // 【テスト目的】: 悪意あるfilename（../../escape.pdf）からも安全な名前のみが生成されることを確認する
        // 【テスト内容】: "../../escape.pdf" でgenerate_object_name()を呼ぶ
        // 【期待される動作】: 戻り値はUUID+拡張子のみで構成され、".."やパス区切りを含まない
        // 🟡 信頼性レベル: 要件定義書 第3章セキュリティ要件・TC-019-E06より

        // 【実際の処理実行】: パストラバーサルを試みるfilenameで呼び出す
        let result = generate_object_name("../../escape.pdf");

        // 【結果検証】: 生成名に".."やパス区切りが含まれないことを確認
        // 【期待値確認】: クライアントfilenameを信頼しない設計が機能していること
        assert!(!result.contains("..")); // 【確認内容】: 親ディレクトリ参照が無害化されていることを確認 🟡
        assert!(!result.contains('/') && !result.contains('\\')); // 【確認内容】: パス区切り文字が無害化されていることを確認 🟡
        assert!(result.ends_with(".pdf")); // 【確認内容】: 拡張子.pdfのみが継承されることを確認 🟡
    }

    /// TC-019-U05: file_storageが保存後にベースディレクトリからの相対パスを返す
    /// 🔵 信頼性レベル: 要件定義書 第2章・note.md「相対パス保存」に直接対応
    #[test]
    fn store_file_returns_relative_path_under_base_dir() {
        // 【テスト目的】: store_file()がベースディレクトリ配下に書込み、絶対パスを含まない相対パスを
        // 返すことを確認する
        // 【テスト内容】: 一時ディレクトリをSTORAGE_ROOTとして用いてstore_file()を呼ぶ
        // 【期待される動作】: 返却relative_pathが絶対パスでなく、base.join(relative_path)が実在ファイル
        // 🔵 信頼性レベル: 要件定義書 第2章出力仕様「相対パス保存」・TC-019-U05に直接対応

        // 【テストデータ準備】: 一時ベースディレクトリと環境変数を設定する
        // 【初期条件設定】: STORAGE_ROOTを一時ディレクトリに切り替える（PDFは既定サブディレクトリ pdf/ 配下になる）
        // 環境変数はプロセス全体で共有されるため、ENV_MUTEXで他のテストとの並列実行による競合を防ぐ
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        clear_storage_env();
        let root = temp_storage_root();
        unsafe {
            std::env::set_var(STORAGE_ROOT_ENV, root.path());
        }
        let writer = TokioFileWriter;

        // 【実際の処理実行】: store_file()でpdfバイト列を書き込む
        // 【処理内容】: 一意名生成→書込→相対パス算出
        let result = store_file(&writer, FileType::Pdf, "example.pdf", b"%PDF-1.4 dummy");

        // 【結果検証】: 戻り値のrelative_pathが絶対パス・ベースディレクトリ文字列を含まないことを確認
        // 【期待値確認】: item_files.pathに絶対パスが漏れないことの保証（要件定義書第2章）
        let stored = result.expect("正常書込が成功するはず");
        assert!(!stored.relative_path.starts_with('/')); // 【確認内容】: 相対パスが先頭'/'を持たないことを確認 🔵
        assert!(stored.base_dir.join(&stored.relative_path).exists()); // 【確認内容】: base.join(相対パス)が実ファイルとして存在することを確認 🔵
        assert_eq!(stored.base_dir, root.path().join("pdf")); // 【確認内容】: 種別サブディレクトリ配下に書かれることを確認 🔵

        // 【テスト後処理】: 他のテストに影響しないよう環境変数を未設定に戻す
        clear_storage_env();
    }

    /// TC-019-E01-SERVICE: 書込失敗注入（FailingFileWriter）でApiError(FileStorageWriteFailed)が返る
    /// 🟡 信頼性レベル: テストケース定義書 第0章「書込失敗注入」・要件定義書EDGE-003・TC-019-E01より
    #[test]
    fn store_file_with_failing_writer_returns_file_storage_write_failed_error() {
        // 【テスト目的】: FileWriterトレイトの書込失敗注入により、store_file()がDBレコードを作らせる前に
        // ApiError(FileStorageWriteFailed)を返すことを確認する
        // 【テスト内容】: FailingFileWriterを注入してstore_file()を呼ぶ
        // 【期待される動作】: Err(ApiError)が返り、error.codeが"FILE_STORAGE_WRITE_FAILED"
        // 🟡 信頼性レベル: TC-019-E01・EDGE-003「ファイルサーバー書込失敗時はitem_filesレコードを作成しない」より

        // 【テストデータ準備】: 常に失敗するFileWriterフェイクを用意する
        // 【初期条件設定】: 特になし（FailingFileWriterはどのpathでも失敗する）
        let writer = FailingFileWriter;

        // 【実際の処理実行】: store_file()に書込失敗フェイクを注入して呼び出す
        let result = store_file(&writer, FileType::Pdf, "example.pdf", b"dummy bytes");

        // 【結果検証】: エラーが返り、ワイヤーコードがFILE_STORAGE_WRITE_FAILEDであることを確認
        // 【期待値確認】: 物理ストレージ障害がクライアントへ一般化されたメッセージで伝わること
        let err = result.expect_err("書込失敗時はErrが返るはず");
        assert_eq!(err.error.code, "FILE_STORAGE_WRITE_FAILED"); // 【確認内容】: 書込失敗が専用エラーコードへマッピングされることを確認 🟡
    }

    /// `STORAGE_ROOT` と全 `STORAGE_SUBDIR_*` を未設定に戻すヘルパー。
    /// 各テストは `ENV_MUTEX` を保持した状態でこれを呼び、他テストの設定残りの影響を排除する。
    fn clear_storage_env() {
        unsafe {
            std::env::remove_var(STORAGE_ROOT_ENV);
            for ft in ALL_FILE_TYPES {
                std::env::remove_var(subdir_spec(ft).0);
            }
        }
    }

    /// テストで全種別を走査するための一覧（`FileType` に追加があればここも更新する）
    const ALL_FILE_TYPES: [FileType; 6] = [
        FileType::Pdf,
        FileType::Image,
        FileType::Video,
        FileType::Audio,
        FileType::Archive,
        FileType::Other,
    ];

    /// TC-019-01/02: FileType 6種すべてがSTORAGE_ROOT配下の「互いに異なる」既定サブディレクトリへ解決される
    /// 🔵 信頼性レベル: 要件定義書 第1章「file_typeに応じたディレクトリ分岐」に直接対応
    #[test]
    fn resolve_base_dir_maps_every_file_type_to_a_distinct_subdir_under_root() {
        // 【テスト目的】: 種別ごとに保存先が分かれること（＝どの2種も同じディレクトリに混ざらないこと）を確認する
        // 【テスト内容】: STORAGE_ROOTのみ設定し、6種すべてのresolve_base_dirの戻り値を集める
        // 【期待される動作】: すべてSTORAGE_ROOT配下であり、6種の解決結果に重複がない
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        clear_storage_env();
        let root = temp_storage_root();
        unsafe {
            std::env::set_var(STORAGE_ROOT_ENV, root.path());
        }

        let resolved: Vec<PathBuf> = ALL_FILE_TYPES
            .iter()
            .map(|ft| resolve_base_dir(*ft))
            .collect();

        for (ft, base) in ALL_FILE_TYPES.iter().zip(&resolved) {
            // 【確認内容】: 各種別の保存先がSTORAGE_ROOT配下に収まることを確認 🔵
            assert!(
                base.starts_with(root.path()),
                "{ft:?} の保存先がルート外: {base:?}"
            );
            // 【確認内容】: ルート直下ではなく種別サブディレクトリが付与されることを確認 🔵
            assert_ne!(
                base.as_path(),
                root.path(),
                "{ft:?} にサブディレクトリが付与されていない"
            );
        }

        let mut unique = resolved.clone();
        unique.sort();
        unique.dedup();
        // 【確認内容】: 6種の保存先がすべて異なる＝種別ごとに保存先が分離されていることを確認 🔵
        assert_eq!(
            unique.len(),
            ALL_FILE_TYPES.len(),
            "種別間で保存先が重複している: {resolved:?}"
        );

        clear_storage_env();
    }

    /// 種別ごとのサブディレクトリを環境変数で上書きできる（ネストした相対パスも許容する）
    /// 🔵 信頼性レベル: 「保存先を環境変数で設定できるようにする」要件に直接対応
    #[test]
    fn resolve_base_dir_honors_subdir_override_env_var() {
        // 【テスト目的】: STORAGE_SUBDIR_* による保存先の上書きが効くことを確認する
        // 【期待される動作】: 指定したネスト付き相対パスがSTORAGE_ROOTへ結合される
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        clear_storage_env();
        unsafe {
            std::env::set_var(STORAGE_ROOT_ENV, "/tmp/mediavault-test-vault");
            std::env::set_var("STORAGE_SUBDIR_VIDEO", "video/2026");
        }

        let base = resolve_base_dir(FileType::Video);

        // 【確認内容】: 上書き値がそのままルート配下へ結合されることを確認 🔵
        assert_eq!(base, PathBuf::from("/tmp/mediavault-test-vault/video/2026"));

        clear_storage_env();
    }

    /// STORAGE_SUBDIR_* にルート外を指す値（絶対パス・`..`）が渡されても既定値へフォールバックする
    /// 🟡 信頼性レベル: generate_object_nameのパストラバーサル防止と対になる新規のセキュリティ要件
    #[test]
    fn resolve_base_dir_rejects_subdir_escaping_the_storage_root() {
        // 【テスト目的】: 設定ミス・悪意ある設定値でルート外へ書き込まれないことを確認する
        // 【テスト内容】: 絶対パスと`..`入りの値をそれぞれ与えてresolve_base_dirを呼ぶ
        // 【期待される動作】: いずれも無視され、ルート配下の既定サブディレクトリが使われる
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        clear_storage_env();
        unsafe {
            std::env::set_var(STORAGE_ROOT_ENV, "/tmp/mediavault-test-vault");
        }

        for malicious in ["/etc", "../../etc", "video/../../etc"] {
            unsafe {
                std::env::set_var("STORAGE_SUBDIR_VIDEO", malicious);
            }
            let base = resolve_base_dir(FileType::Video);
            // 【確認内容】: ルート外への脱出が拒否され、既定サブディレクトリへ戻ることを確認 🟡
            assert_eq!(
                base,
                PathBuf::from("/tmp/mediavault-test-vault/video"),
                "{malicious:?} がルート外への脱出を許している"
            );
        }

        clear_storage_env();
    }

    /// TC-019-E07: ベースディレクトリ未設定時もデフォルトがカレントディレクトリにならない（REQ-402）
    /// 🟡 信頼性レベル: note.md・要件定義書第3章REQ-402より妥当推測
    #[test]
    fn resolve_base_dir_default_does_not_point_to_current_directory() {
        // 【テスト目的】: 環境変数未設定時のデフォルト値がカレントディレクトリ（"."）にならず、
        // マウント済みのMediaVault専用領域を指すことを確認する（REQ-402: コンテナ内非保存）
        // 【期待される動作】: 戻り値がDEFAULT_STORAGE_ROOT配下になる
        // 【注記】: デプロイ先はLinuxコンテナだが本テストはWindows上でも走るため、`is_absolute()`
        // （WindowsではUnix絶対パスをfalseと判定する）ではなくプレフィックス一致で検証する。
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        clear_storage_env();

        for ft in ALL_FILE_TYPES {
            let base = resolve_base_dir(ft);
            // 【確認内容】: デフォルトパスがコンテナ内カレントでなく専用領域配下であることを確認 🟡
            assert!(
                base.starts_with(DEFAULT_STORAGE_ROOT),
                "{ft:?} のデフォルトパスが専用領域外: {base:?}"
            );
            assert!(
                !base.starts_with("."),
                "{ft:?} のデフォルトパスがカレントディレクトリ相対: {base:?}"
            );
        }
    }

    /// IT-019-02: DB登録失敗を模してcleanup_file()で書込済みファイルが削除される
    /// 🟡 信頼性レベル: 要件定義書EDGE-003対称・TC-019-E02/IT-019-02より
    #[test]
    fn cleanup_file_removes_previously_written_file() {
        // 【テスト目的】: DB INSERT失敗を模した状況でcleanup_file()を呼ぶと、書込済みファイルが
        // 削除され孤立ファイルが残らないことを確認する
        // 【テスト内容】: store_file()で書込んだ後、cleanup_file()を呼びファイル不在を確認する
        // 【期待される動作】: cleanup_file()呼び出し後、stored.absolute_pathが存在しない
        // 🟡 信頼性レベル: TC-019-E02・IT-019-02「書込成功後のDB失敗時ファイル削除」より

        // 【テストデータ準備】: 一時ディレクトリへ実際に書込んでおく
        // 環境変数はプロセス全体で共有されるため、ENV_MUTEXで他のテストとの並列実行による競合を防ぐ
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        clear_storage_env();
        let root = temp_storage_root();
        unsafe {
            std::env::set_var(STORAGE_ROOT_ENV, root.path());
        }
        let writer = TokioFileWriter;
        let stored = store_file(&writer, FileType::Pdf, "example.pdf", b"dummy")
            .expect("前提となる書込が成功するはず");

        // 【実際の処理実行】: cleanup_file()でロールバック削除を実行する
        let result = cleanup_file(&writer, &stored);

        // 【結果検証】: 削除が成功し、ファイルが実在しないことを確認
        assert!(result.is_ok()); // 【確認内容】: cleanup_file()がエラーなく完了することを確認 🟡
        assert!(!stored.absolute_path.exists()); // 【確認内容】: ロールバック後に書込済みファイルが存在しないことを確認 🟡

        // 【テスト後処理】: 他のテストに影響しないよう環境変数を未設定に戻す
        clear_storage_env();
    }
}
