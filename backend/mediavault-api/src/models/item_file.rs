//! item_files（パス指定方式）のモデル・リクエストDTO・バリデーション
//!
//! TASK-0026: POST /items/:id/files（パス指定方式）実装（models/item_link.rsと対称な構造）

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::response::{ApiError, ApiErrorCode};

/// ファイル種別（`pdf`, `image`, `video`, `audio`, `archive`, `other`）
/// 🔵 信頼性レベル: init migrationのfile_type ENUM定義に直接対応
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "file_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    Pdf,
    Image,
    Video,
    Audio,
    Archive,
    Other,
}

impl FileType {
    /// 拡張子（小文字比較・先頭ドットなし）からファイル種別を判定する。未知の拡張子はOther。
    pub fn from_extension(ext: &str) -> FileType {
        match ext.to_ascii_lowercase().as_str() {
            "pdf" => FileType::Pdf,
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "svg" | "avif" | "heic" => {
                FileType::Image
            }
            "mp4" | "mkv" | "avi" | "mov" | "wmv" | "webm" | "m4v" | "flv" | "ts" => {
                FileType::Video
            }
            "mp3" | "flac" | "wav" | "aac" | "ogg" | "m4a" | "opus" | "wma" => FileType::Audio,
            "zip" | "rar" | "7z" | "tar" | "gz" | "cbz" | "cbr" => FileType::Archive,
            _ => FileType::Other,
        }
    }

    /// パスまたはファイル名の拡張子からファイル種別を判定する。拡張子なしはOther。
    pub fn from_path(path: &str) -> FileType {
        std::path::Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(FileType::from_extension)
            .unwrap_or(FileType::Other)
    }
}

/// item_files本体（`POST /items/:id/files`のレスポンスで返す表現）
/// 🔵 信頼性レベル: database-schema.sqlのitem_filesテーブル定義に直接対応
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemFile {
    pub id: Uuid,
    pub item_id: Uuid,
    pub path: String,
    pub label: Option<String>,
    pub file_type: FileType,
    pub calibre_book_id: Option<String>,
    pub created_at: NaiveDateTime,
}

/// `POST /items/:id/files` リクエストDTO（パス指定方式）。
/// file_typeはクライアント指定ではなくpathの拡張子から自動分類する（未知フィールドは無視される）。
#[derive(Debug, Clone, Deserialize)]
pub struct CreateItemFileRequest {
    pub path: String,
    pub label: Option<String>,
}

/// バリデーション済みのファイル登録内容（file_typeは拡張子から導出済み）
#[derive(Debug, Clone)]
pub struct ValidatedItemFileRequest {
    pub path: String,
    pub label: Option<String>,
    pub file_type: FileType,
}

/// 【機能概要】: CreateItemFileRequestのバリデーション（path空文字拒否）と
/// pathの拡張子からのfile_type自動分類を行う
pub fn parse_create_item_file_request(
    request: CreateItemFileRequest,
) -> Result<ValidatedItemFileRequest, ApiError> {
    if request.path.trim().is_empty() {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "pathは必須です",
        ));
    }
    let file_type = FileType::from_path(&request.path);
    Ok(ValidatedItemFileRequest {
        path: request.path,
        label: request.label,
        file_type,
    })
}

/// `PATCH /items/:id/files/:file_id/calibre-link` リクエストDTO
/// 🔵 信頼性レベル: calibre-link-requirements.md 2.1 L38-43・api-endpoints.md リクエスト例に直接対応
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateCalibreLinkRequest {
    pub calibre_book_id: String,
}

/// 【機能概要】: UpdateCalibreLinkRequestのバリデーション（calibre_book_id空文字/空白拒否）を行う
/// 🟡 信頼性レベル: calibre-link-requirements.md 2.1 L42（trim後空文字はVALIDATION_ERROR）・
/// 既存parse_create_item_file_requestと対称なパターンより新規追加
pub fn parse_update_calibre_link_request(
    request: UpdateCalibreLinkRequest,
) -> Result<UpdateCalibreLinkRequest, ApiError> {
    if request.calibre_book_id.trim().is_empty() {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "calibre_book_idは必須です",
        ));
    }
    Ok(request)
}

/// パスパラメータの文字列をfile_id用UUIDへパースする。
///
/// 【機能概要】: `PATCH /items/:id/files/:file_id/calibre-link`の:file_idパスパラメータを検証し、
/// 不正な形式の場合は`VALIDATION_ERROR`（400）を返す
/// 🟡 信頼性レベル: calibre-link-requirements.md 2.1 L37（parse_file_id相当を新規追加）・
/// 既存models::item::parse_item_idと対称なパターンより新規追加
pub fn parse_file_id(raw: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(raw).map_err(|_| {
        ApiError::new(
            ApiErrorCode::ValidationError,
            "file_idはUUID形式である必要があります",
        )
    })
}

/// Calibre-Web遷移用情報（アイテム詳細APIレスポンス拡張用の独立した小型構造体）
///
/// 【機能概要】: `calibre_book_id`設定済みのPDF item_filesについて、アイテム詳細APIの
/// レスポンスに付加するCalibre-Web遷移情報を保持する
/// 【実装方針】: Calibre-Web側のURL構成・認証方式が未確定のため、変更容易な独立構造体として
/// 定義する（calibre-link-requirements.md 3 L97・タスク定義書 L58-64, L102）
/// 🟡 信頼性レベル: calibre-link-testcases.md TC-020-02・TC-020-N03・TC-020-B05より新規追加
#[derive(Debug, Clone, Serialize)]
pub struct CalibreWebLinkInfo {
    pub file_id: Uuid,
    pub calibre_book_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テストケース1: CreateItemFileRequestの正常デシリアライズ
    #[test]
    fn create_item_file_request_deserializes_valid_fields() {
        let value = serde_json::json!({
            "path": "/srv/files/pdf/example.pdf",
            "label": "本編PDF"
        });

        let request: CreateItemFileRequest = serde_json::from_value(value).unwrap();

        assert_eq!(request.path, "/srv/files/pdf/example.pdf");
        assert_eq!(request.label, Some("本編PDF".to_string()));
    }

    /// テストケース1b: labelが省略された場合もデシリアライズできる
    #[test]
    fn create_item_file_request_deserializes_without_label() {
        let value = serde_json::json!({
            "path": "/srv/files/pdf/example.pdf"
        });

        let request: CreateItemFileRequest = serde_json::from_value(value).unwrap();

        assert_eq!(request.label, None);
    }

    /// 後方互換: 従来クライアントがfile_typeを送信してもエラーにならず無視される
    #[test]
    fn create_item_file_request_ignores_legacy_file_type_field() {
        let value = serde_json::json!({
            "path": "/srv/files/pdf/example.pdf",
            "file_type": "image"
        });

        let request: CreateItemFileRequest = serde_json::from_value(value).unwrap();

        assert_eq!(request.path, "/srv/files/pdf/example.pdf");
    }

    /// テストケース4: pathが空文字でVALIDATION_ERROR
    #[test]
    fn parse_create_item_file_request_rejects_empty_path() {
        let request = CreateItemFileRequest {
            path: "".to_string(),
            label: Some("本編PDF".to_string()),
        };

        let result = parse_create_item_file_request(request);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().error.code,
            ApiErrorCode::ValidationError.as_code_str()
        );
    }

    /// 正常な入力は検証を通過し、拡張子からfile_typeが導出される
    #[test]
    fn parse_create_item_file_request_derives_file_type_from_extension() {
        let request = CreateItemFileRequest {
            path: "/srv/files/pdf/example.pdf".to_string(),
            label: Some("本編PDF".to_string()),
        };

        let result = parse_create_item_file_request(request);

        let validated = result.unwrap();
        assert_eq!(validated.file_type, FileType::Pdf);
        assert_eq!(validated.path, "/srv/files/pdf/example.pdf");
    }

    /// FileType::from_extension が代表的な拡張子を正しく分類する（大文字小文字非依存）
    #[test]
    fn file_type_from_extension_classifies_known_extensions() {
        assert_eq!(FileType::from_extension("pdf"), FileType::Pdf);
        assert_eq!(FileType::from_extension("PDF"), FileType::Pdf);
        assert_eq!(FileType::from_extension("jpg"), FileType::Image);
        assert_eq!(FileType::from_extension("PNG"), FileType::Image);
        assert_eq!(FileType::from_extension("webp"), FileType::Image);
        assert_eq!(FileType::from_extension("mp4"), FileType::Video);
        assert_eq!(FileType::from_extension("MKV"), FileType::Video);
        assert_eq!(FileType::from_extension("mp3"), FileType::Audio);
        assert_eq!(FileType::from_extension("flac"), FileType::Audio);
        assert_eq!(FileType::from_extension("zip"), FileType::Archive);
        assert_eq!(FileType::from_extension("cbz"), FileType::Archive);
        assert_eq!(FileType::from_extension("xyz"), FileType::Other);
    }

    /// FileType::from_path がパス・ファイル名の拡張子から分類し、拡張子なしはOtherになる
    #[test]
    fn file_type_from_path_handles_paths_and_missing_extension() {
        assert_eq!(FileType::from_path("/srv/files/pdf/a.pdf"), FileType::Pdf);
        assert_eq!(FileType::from_path("cover.JPG"), FileType::Image);
        assert_eq!(
            FileType::from_path("/data/videos/ep01.mkv"),
            FileType::Video
        );
        assert_eq!(FileType::from_path("noextension"), FileType::Other);
        assert_eq!(
            FileType::from_path("/data/archive.tar.gz"),
            FileType::Archive
        );
        assert_eq!(FileType::from_path(""), FileType::Other);
    }

    /// TC-020-U02: UpdateCalibreLinkRequestがcalibre_book_idを正しくデシリアライズする
    /// 【テスト目的】: `{ "calibre_book_id": "calibre-12345" }` JSONが `UpdateCalibreLinkRequest`
    /// 構造体へ変換できることを確認する
    /// 【テスト内容】: serde_json::json!でリクエストボディ相当のJSONを構築し、デシリアライズする
    /// 【期待される動作】: request.calibre_book_id == "calibre-12345"
    /// 🔵 信頼性レベル: calibre-link-requirements.md 2.1 L38-43・既存
    /// create_item_file_request_deserializes_valid_fields パターンに直接対応
    /// 【Red期待】: UpdateCalibreLinkRequestが未定義のためコンパイルエラーとなる想定
    #[test]
    fn update_calibre_link_request_deserializes_valid_fields() {
        // 【テストデータ準備】: api-endpoints.md / 要件定義書2.1のリクエスト例そのもの 🔵
        // 【初期条件設定】: calibre_book_idのみを持つ正常な入力
        let value = serde_json::json!({ "calibre_book_id": "calibre-12345" });

        // 【実際の処理実行】: serde_json::from_valueでUpdateCalibreLinkRequestへ変換する
        // 【処理内容】: serdeのDeserialize実装による型変換
        let request: UpdateCalibreLinkRequest = serde_json::from_value(value).unwrap();

        // 【結果検証】: フィールド名・型が要件のリクエスト形式に一致していることを確認する
        // 【期待値確認】: calibre_book_idフィールドに入力値がそのまま格納されること
        assert_eq!(request.calibre_book_id, "calibre-12345"); // 【確認内容】: calibre_book_idフィールドが正しくデシリアライズされることを確認 🔵
    }

    /// TC-020-U03: parse_update_calibre_link_requestが非空値を受理する
    /// 【テスト目的】: trim後に非空のcalibre_book_idを持つリクエストがOkを返すことを確認する
    /// 【テスト内容】: 正常な書籍ID値を持つUpdateCalibreLinkRequestをparse_update_calibre_link_requestへ渡す
    /// 【期待される動作】: parse_update_calibre_link_request(req).is_ok() == true
    /// 🔵 信頼性レベル: 既存parse_create_item_file_request_accepts_valid_fieldsパターン・
    /// calibre-link-requirements.md 2.1に対応
    /// 【Red期待】: parse_update_calibre_link_requestが未実装のためコンパイルエラーとなる想定
    #[test]
    fn parse_update_calibre_link_request_accepts_non_empty_value() {
        // 【テストデータ準備】: 正常な書籍IDの代表値 🔵
        // 【初期条件設定】: 非空のcalibre_book_idを持つリクエスト
        let request = UpdateCalibreLinkRequest {
            calibre_book_id: "calibre-12345".to_string(),
        };

        // 【実際の処理実行】: parse_update_calibre_link_requestでバリデーションする
        // 【処理内容】: trim後の空文字判定ロジックを通過させる
        let result = parse_update_calibre_link_request(request);

        // 【結果検証】: 正常値が誤って弾かれないことを確認する
        // 【期待値確認】: Ok(request)が返ること
        assert!(result.is_ok()); // 【確認内容】: 非空のcalibre_book_idが検証を通過することを確認 🔵
        assert_eq!(result.unwrap().calibre_book_id, "calibre-12345"); // 【確認内容】: 検証通過後もcalibre_book_idの値が保持されることを確認 🔵
    }

    /// TC-020-E03: calibre_book_idが空文字でVALIDATION_ERROR(400)
    /// 【テスト目的】: calibre_book_idが空文字の場合にparse_update_calibre_link_requestがErrを返すことを確認する
    /// 【テスト内容】: calibre_book_id=""のリクエストをparse_update_calibre_link_requestへ渡す
    /// 【期待される動作】: Err(ApiError{ code: VALIDATION_ERROR })
    /// 🟡 信頼性レベル: calibre-link-requirements.md 4.2 E03 L115・2.1 L42（trim後空文字はVALIDATION_ERROR）に対応。
    /// 既存parse_create_item_file_request_rejects_empty_pathと対称な必須検証
    /// 【Red期待】: parse_update_calibre_link_requestが未実装のためコンパイルエラーとなる想定
    #[test]
    fn parse_update_calibre_link_request_rejects_empty_string() {
        // 【テストデータ準備】: 無意味な空IDの紐付けを防ぐための境界値（空文字） 🟡
        // 【初期条件設定】: calibre_book_id=""のリクエスト
        let request = UpdateCalibreLinkRequest {
            calibre_book_id: "".to_string(),
        };

        // 【実際の処理実行】: parse_update_calibre_link_requestでバリデーションする
        let result = parse_update_calibre_link_request(request);

        // 【結果検証】: 必須・非空制約に違反しているためErrを返すことを確認する
        // 【期待値確認】: エラーコードがVALIDATION_ERRORであること
        assert!(result.is_err()); // 【確認内容】: 空文字のcalibre_book_idが拒否されることを確認 🟡
        assert_eq!(
            result.unwrap_err().error.code,
            ApiErrorCode::ValidationError.as_code_str()
        ); // 【確認内容】: エラーコードがVALIDATION_ERRORであることを確認 🟡
    }

    /// TC-020-E03b: calibre_book_idが空白のみでVALIDATION_ERROR(400)
    /// 【テスト目的】: calibre_book_idが空白のみ（trim後に空文字になる）場合もErrを返すことを確認する
    /// 【テスト内容】: calibre_book_id="   "のリクエストをparse_update_calibre_link_requestへ渡す
    /// 【期待される動作】: Err(ApiError{ code: VALIDATION_ERROR })
    /// 🟡 信頼性レベル: calibre-link-requirements.md 4.2 E03 L115・2.1 L42（trim後空文字判定）に対応
    /// 【Red期待】: parse_update_calibre_link_requestが未実装のためコンパイルエラーとなる想定
    #[test]
    fn parse_update_calibre_link_request_rejects_whitespace_only() {
        // 【テストデータ準備】: trim後に空文字となる空白のみの入力 🟡
        // 【初期条件設定】: calibre_book_id="   "のリクエスト
        let request = UpdateCalibreLinkRequest {
            calibre_book_id: "   ".to_string(),
        };

        // 【実際の処理実行】: parse_update_calibre_link_requestでバリデーションする
        let result = parse_update_calibre_link_request(request);

        // 【結果検証】: trim後空文字としてErrを返すことを確認する
        assert!(result.is_err()); // 【確認内容】: 空白のみのcalibre_book_idが拒否されることを確認 🟡
        assert_eq!(
            result.unwrap_err().error.code,
            ApiErrorCode::ValidationError.as_code_str()
        ); // 【確認内容】: エラーコードがVALIDATION_ERRORであることを確認 🟡
    }

    /// TC-020-E05: calibre_book_idキー欠落のJSONはUpdateCalibreLinkRequestへのデシリアライズに失敗する
    /// 【テスト目的】: 必須キーcalibre_book_idが欠落したJSONがデシリアライズエラーになることを確認する
    /// 【テスト内容】: serde_json::json!({})をUpdateCalibreLinkRequestへ変換しようとする
    /// 【期待される動作】: 変換結果がErrになる
    /// 🟡 信頼性レベル: calibre-link-requirements.md 4.2 E05 L117・既存deserialize_request実装に対応
    /// 【Red期待】: UpdateCalibreLinkRequestが未定義のためコンパイルエラーとなる想定
    #[test]
    fn update_calibre_link_request_with_missing_key_fails_deserialization() {
        // 【テストデータ準備】: calibre_book_idキーが存在しない不正なボディ 🟡
        // 【初期条件設定】: 空オブジェクト
        let value = serde_json::json!({});

        // 【実際の処理実行】: serde_json::from_valueでUpdateCalibreLinkRequestへ変換を試みる
        let result: Result<UpdateCalibreLinkRequest, _> = serde_json::from_value(value);

        // 【結果検証】: 必須キー欠落によりデシリアライズが失敗することを確認する
        assert!(result.is_err()); // 【確認内容】: calibre_book_idキー欠落でデシリアライズが失敗することを確認 🟡
    }

    /// TC-020-E05b: calibre_book_idの型が数値の場合はUpdateCalibreLinkRequestへのデシリアライズに失敗する
    /// 【テスト目的】: calibre_book_idの型が文字列以外（数値）の場合にデシリアライズが失敗することを確認する
    /// 【テスト内容】: serde_json::json!({ "calibre_book_id": 123 })をUpdateCalibreLinkRequestへ変換しようとする
    /// 【期待される動作】: 変換結果がErrになる
    /// 🟡 信頼性レベル: calibre-link-requirements.md 4.2 E05 L117より新規追加
    /// 【Red期待】: UpdateCalibreLinkRequestが未定義のためコンパイルエラーとなる想定
    #[test]
    fn update_calibre_link_request_with_invalid_type_fails_deserialization() {
        // 【テストデータ準備】: calibre_book_idが文字列ではなく数値型の不正なボディ 🟡
        let value = serde_json::json!({ "calibre_book_id": 123 });

        // 【実際の処理実行】: serde_json::from_valueでUpdateCalibreLinkRequestへ変換を試みる
        let result: Result<UpdateCalibreLinkRequest, _> = serde_json::from_value(value);

        // 【結果検証】: 型不一致によりデシリアライズが失敗することを確認する
        assert!(result.is_err()); // 【確認内容】: calibre_book_idの型不正でデシリアライズが失敗することを確認 🟡
    }

    /// TC-020-B01: parse_file_idが不正なUUID文字列をVALIDATION_ERRORで拒否する
    /// 【テスト目的】: parse_file_idが既存parse_item_idと同様にUUID形式以外の文字列を400で拒否することを確認する
    /// 【テスト内容】: "not-a-uuid"をparse_file_idへ渡す
    /// 【期待される動作】: Err(ApiError{ code: VALIDATION_ERROR })
    /// 🔵 信頼性レベル: calibre-link-requirements.md 4.2 E04 L116・既存parse_item_id・既存TC-019-E05に対応
    /// 【Red期待】: parse_file_idが未実装のためコンパイルエラーとなる想定
    #[test]
    fn parse_file_id_rejects_invalid_uuid_string() {
        // 【テストデータ準備】: UUID形式でない不正な文字列（既存parse_item_idテストと対称） 🔵
        let raw = "not-a-uuid";

        // 【実際の処理実行】: parse_file_idでパースを試みる
        let result = parse_file_id(raw);

        // 【結果検証】: パース失敗時に確実に400 VALIDATION_ERRORへマッピングされることを確認する
        assert!(result.is_err()); // 【確認内容】: 不正なUUID文字列がエラーになることを確認 🔵
        assert_eq!(
            result.unwrap_err().error.code,
            ApiErrorCode::ValidationError.as_code_str()
        ); // 【確認内容】: エラーコードがVALIDATION_ERRORであることを確認 🔵
    }

    /// TC-020-B02b: parse_file_idが正当なUUID文字列を受理する
    /// 【テスト目的】: parse_file_idが正当なUUID文字列を正しくUuidへ変換できることを確認する
    /// 【テスト内容】: Uuid::new_v4()の文字列表現をparse_file_idへ渡す
    /// 【期待される動作】: Ok(uuid)が返り、元の値と一致する
    /// 🟡 信頼性レベル: parse_item_idの既存正常系テストパターンとの対称性からの妥当な推測
    /// 【Red期待】: parse_file_idが未実装のためコンパイルエラーとなる想定
    #[test]
    fn parse_file_id_accepts_valid_uuid_string() {
        // 【テストデータ準備】: 正当なUUID値 🟡
        let id = Uuid::new_v4();

        // 【実際の処理実行】: parse_file_idでパースする
        let result = parse_file_id(&id.to_string());

        // 【結果検証】: 元のUUID値と一致することを確認する
        assert_eq!(result.unwrap(), id); // 【確認内容】: 正当なUUID文字列が誤って拒否されないことを確認 🟡
    }
}
