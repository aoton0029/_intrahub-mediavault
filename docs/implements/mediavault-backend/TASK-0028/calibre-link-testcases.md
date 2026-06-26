# TDDテストケース定義書: TASK-0028 PATCH /items/:id/files/:file_id/calibre-link

**機能名**: calibre-link（item_files の calibre_book_id 更新 + アイテム詳細APIへのCalibre-Web遷移情報付加）
**タスクID**: TASK-0028
**要件名**: mediavault-backend
**出力ファイル**: `docs/implements/mediavault-backend/TASK-0028/calibre-link-testcases.md`
**作成日**: 2026-06-26

> **信頼性レベル凡例**
> - 🔵 **青信号**: 要件定義書・タスク定義書・既存実装を参照し、ほぼ推測なし
> - 🟡 **黄信号**: 上記資料からの妥当な推測
> - 🔴 **赤信号**: 資料に明確な記載がなく推測を含む

---

## 0. 前提・エラーコード追加事項（テスト実装の前提条件）

本タスクのテストを成立させるために、実装側で以下の**新規追加**が必要（テストケースが参照する前提）:

1. 🟡 **`ApiErrorCode::FileNotFound`（`"FILE_NOT_FOUND"` / 404）variant を `backend/mediavault-api/src/models/response.rs` に追加**。
   現状の `ApiErrorCode` enum には未定義（参照: response.rs L52-105）。`code_and_status()`（L109-150）への追加も必要。
   既存の `ApiKeyNotConfigured`（TASK-0024）/ `FileStorageWriteFailed`（TASK-0027）の追加パターンと同様。
2. 🟡 **`:file_id` 用UUID検証関数**（`parse_file_id` 相当）。既存 `parse_item_id`（item.rs L291-298）と同等構造を流用/追加。
3. 🔵 **`UpdateCalibreLinkRequest { calibre_book_id: String }` DTO** + **`parse_update_calibre_link_request`** を `models/item_file.rs` に追加。
4. 🟡 **`update_calibre_link` リポジトリ関数**（対象行取得 → file_type≠pdf判定 → UPDATE RETURNING の2段階方式）を `item_file_repository.rs` に追加。
5. 🟡 **詳細API拡張**: `ItemDetail`（または詳細レスポンス）に PDF item_files の `calibre_book_id` / Calibre-Web 遷移情報を付加する小型構造体を追加（変更容易な独立構造体）。

> **重要（検証順序）**: テストケース2（file_type≠pdf → 400）とテストケース3（不存在/不一致 → 404）を区別するため、
> リポジトリ層は **(a) id+item_id で対象行を取得 → (b) 0行なら 404(FileNotFound) → (c) 取得行の file_type≠pdf なら 400(ValidationError) → (d) pdfなら UPDATE** の順で判定する。
> 単純な `WHERE ... AND file_type='pdf'` のみでは両ケースとも0行になり区別不能（要件定義書 第3章 L92 に明記）。

---

## 1. 開発言語・フレームワーク

- **プログラミング言語**: Rust (edition 2024)
  - **言語選択の理由**: バックエンドAPI全体がRust + Axumで構築されている（Cargo.toml）。型システムによる網羅検証・コンパイル時エラー検出がTDDに適する。
  - **テストに適した機能**: 組込 `#[test]` / `#[tokio::test]`、`Result`型による明示的エラー、enum網羅 `match`。
- **テストフレームワーク**: Rust標準テスト（`#[test]` / `#[tokio::test]`） + tower::ServiceExt（`oneshot`）によるルーター統合テスト
  - **フレームワーク選択の理由**: 既存テスト（response.rs, item_file.rs, item_file_repository.rs, item_files.rs）が全てこの構成。新規フレームワーク導入不要。
  - **テスト実行環境**:
    - ユニットテスト（DB非依存）: `cargo test -p mediavault-api`
    - 統合テスト（DB依存）: `#[tokio::test] #[ignore]` + `DATABASE_URL` 環境変数、`cargo test -p mediavault-api -- --ignored`（docker-compose db 起動前提）
  - 🔵 信頼性レベル: note.md 第5章・既存テスト構成に直接対応

### テスト配置方針

| テスト種別 | 配置ファイル | 実行方法 | 備考 |
|---|---|---|---|
| エラーコードマッピング（DB非依存） | `src/models/response.rs` #[cfg(test)] | `#[test]` 通常実行 | TC-020-U01 |
| DTO検証（DB非依存） | `src/models/item_file.rs` #[cfg(test)] | `#[test]` 通常実行 | TC-020-U02〜U05 |
| リポジトリ（DB依存） | `src/repositories/item_file_repository.rs` #[cfg(test)] | `#[tokio::test] #[ignore]` | TC-020-R01〜R05 |
| ハンドラ/ルーター（DB依存） | `src/handlers/item_files.rs`, `src/handlers/items.rs` #[cfg(test)] | `#[tokio::test] #[ignore]` | TC-020-01/02, E01〜E05 |

---

## 2. 正常系テストケース（基本的な動作）

### TC-020-U02: UpdateCalibreLinkRequest の正常デシリアライズ（DB非依存）

- **テスト名**: UpdateCalibreLinkRequestがcalibre_book_idを正しくデシリアライズする
  - **何をテストするか**: `{ "calibre_book_id": "calibre-12345" }` JSONが `UpdateCalibreLinkRequest` 構造体へ変換できること
  - **期待される動作**: `request.calibre_book_id == "calibre-12345"`
- **入力値**: `serde_json::json!({ "calibre_book_id": "calibre-12345" })`
  - **入力データの意味**: api-endpoints.md / 要件定義書 2.1 のリクエスト例そのもの（代表的な正常入力）
- **期待される結果**: デシリアライズ成功、`calibre_book_id` フィールドに値が格納される
  - **期待結果の理由**: DTO定義が要件のリクエスト形式に一致していることを保証する
- **テストの目的**: DTO定義（フィールド名・型）の正しさ確認
  - **確認ポイント**: フィールド名が `calibre_book_id`（snake_case）であること、型が `String` であること
- 🔵 信頼性レベル: 要件定義書 2.1 L38-43・既存 `create_item_file_request_deserializes_valid_fields` パターンに直接対応

### TC-020-U03: 非空の calibre_book_id がバリデーションを通過する（DB非依存）

- **テスト名**: parse_update_calibre_link_requestが非空値を受理する
  - **何をテストするか**: trim後に非空の `calibre_book_id` を持つリクエストが `Ok` を返すこと
  - **期待される動作**: `parse_update_calibre_link_request(req).is_ok() == true`
- **入力値**: `UpdateCalibreLinkRequest { calibre_book_id: "calibre-12345".to_string() }`
  - **入力データの意味**: 正常な書籍IDの代表値
- **期待される結果**: `Ok(request)` が返り、`calibre_book_id` が保持される
  - **期待結果の理由**: 非空値は正当な入力であり、検証を通過すべき
- **テストの目的**: バリデーション関数の正常パス確認
  - **確認ポイント**: 正常値を誤って弾かないこと
- 🔵 信頼性レベル: 既存 `parse_create_item_file_request_accepts_valid_fields` パターン・要件定義書 2.1 に対応

### TC-020-R01: リポジトリ update_calibre_link がpdfレコードを更新する（DB依存・統合）

- **テスト名**: update_calibre_linkがfile_type=pdfのレコードのcalibre_book_idを更新し更新後行を返す
  - **何をテストするか**: 実DBで `file_type=pdf` の item_files 行に対し `update_calibre_link` を呼ぶと `calibre_book_id` が更新され、更新後の `ItemFile` が返ること
  - **期待される動作**: 返却 `ItemFile.calibre_book_id == Some("calibre-12345")`、DB上の該当行も更新済み
- **入力値**: 事前にinsertした `file_type=pdf` の item_files 行の `item_id`/`file_id`、`calibre_book_id="calibre-12345"`
  - **入力データの意味**: TC-020-01 のリポジトリ層相当。実DB更新の最小ケース
- **期待される結果**: `Ok(ItemFile{ calibre_book_id: Some("calibre-12345"), file_type: Pdf, .. })`、`SELECT` で再確認しても更新済み
  - **期待結果の理由**: UPDATE ... RETURNING が正しく動作し、永続化されていることを保証する
- **テストの目的**: リポジトリのコアUPDATEロジックの確認
  - **確認ポイント**: RETURNINGの値とDB永続値が一致すること
- 🔵 信頼性レベル: タスク定義書 L51-56・要件定義書 2.3 L75・TC-020-01 に直接対応

### TC-020-01: PATCH calibre-link がpdfレコードを更新し200を返す（DB依存・ルーター統合）

- **テスト名**: PATCH /items/:id/files/:file_id/calibre-link が200を返しcalibre_book_idを更新する
  - **何をテストするか**: ルーター経由で実在itemの `file_type=pdf` ファイルに対しPATCHすると200・更新後レコードが返ること
  - **期待される動作**: HTTP 200、`data.calibre_book_id == "calibre-12345"`、`data.file_type == "pdf"`
- **入力値**:
  - URI: `PATCH /items/{item_id}/files/{file_id}/calibre-link`（item_id/file_idは事前insert済みpdf行）
  - Body: `{ "calibre_book_id": "calibre-12345" }`
  - **入力データの意味**: TC-020-01（タスク定義書 L68-71）のエンドツーエンド正常系
- **期待される結果**: `response.status() == StatusCode::OK`、レスポンスbodyの `data.calibre_book_id` が更新値
  - **期待結果の理由**: 完了条件「更新成功時、更新後の item_files レコードを200で返す」に対応
- **テストの目的**: エンドポイント全体（ルート登録・ハンドラ・リポジトリ）の正常動作確認
  - **確認ポイント**: ルートが登録されていること、200であること、レスポンスに更新値が反映されること
- 🔵 信頼性レベル: タスク定義書 単体テスト要件 テストケース1（TC-020-01）・完了条件 L24,27 に直接対応

### TC-020-N01: 冪等性 — 同一calibre_book_idで複数回PATCHしても結果が変わらない（DB依存・統合）

- **テスト名**: 同じcalibre_book_idで2回PATCHしても200・同一結果が返る
  - **何をテストするか**: 同じ値での再実行が冪等であること（要件定義書 2.3 L76）
  - **期待される動作**: 1回目も2回目も200、`calibre_book_id` は同値、エラーや重複違反が起きない
- **入力値**: 同一 `item_id`/`file_id` に対し `{ "calibre_book_id": "calibre-12345" }` を2回送信
  - **入力データの意味**: 連携バッチが再送した場合の冪等性検証（実運用シナリオ）
- **期待される結果**: 2回とも `StatusCode::OK`、最終 `calibre_book_id == "calibre-12345"`
  - **期待結果の理由**: 更新は冪等という要件（要件定義書 2.3 L76）の保証
- **テストの目的**: 冪等性の確認
  - **確認ポイント**: 2回目が一意制約違反等で失敗しないこと
- 🟡 信頼性レベル: 要件定義書 2.3 L76（冪等性明記）からの妥当な推測（テストケースとしては要件記述から導出）

### TC-020-N02: 既存calibre_book_idの上書き更新（DB依存・統合）

- **テスト名**: 既にcalibre_book_id設定済みのpdf行を別の値で上書き更新できる
  - **何をテストするか**: `calibre_book_id` が `"old-1"` のpdf行に対し `"calibre-99999"` でPATCHすると上書きされること
  - **期待される動作**: 200、`data.calibre_book_id == "calibre-99999"`
- **入力値**: 事前に `calibre_book_id="old-1"` をセットしたpdf行、Body `{ "calibre_book_id": "calibre-99999" }`
  - **入力データの意味**: Calibre-Web側の再取込で書籍IDが変わった場合（実運用シナリオ）
- **期待される結果**: `StatusCode::OK`、旧値が新値で置き換わる
  - **期待結果の理由**: NULL→値だけでなく値→値の更新も成立すべき
- **テストの目的**: 上書き更新（非NULL初期状態）の確認
  - **確認ポイント**: NOT NULL初期値でも問題なく更新されること
- 🟡 信頼性レベル: 更新仕様（要件定義書 2.3）からの妥当な推測

### TC-020-02: アイテム詳細APIレスポンスにCalibre-Web遷移情報が含まれる（DB依存・ルーター統合）

- **テスト名**: GET /items/:id がcalibre_book_id設定済みPDFのCalibre-Web遷移情報を含む
  - **何をテストするか**: `calibre_book_id` 設定済みPDFを持つアイテムの詳細取得で、該当ファイル情報に `calibre_book_id`（およびCalibre-Web遷移情報）が含まれること
  - **期待される動作**: 200、レスポンスJSONの該当ファイル要素に `calibre_book_id` が含まれる（遷移情報構造体含む）
- **入力値**:
  - 事前準備: itemを作成 → pdf item_fileを作成 → calibre-link PATCHで `calibre_book_id="calibre-12345"` をセット
  - URI: `GET /items/{item_id}`
  - **入力データの意味**: TC-020-02（タスク定義書 L83-86）のエンドツーエンド検証
- **期待される結果**: `StatusCode::OK`、レスポンスbodyから `calibre-12345` が見つかる（item_filesまたは遷移情報フィールド経由）
  - **期待結果の理由**: 完了条件「calibre_book_id 設定済みPDFについて詳細APIにCalibre-Web遷移情報が含まれる」に対応
- **テストの目的**: 詳細API拡張（TC-020-02）の確認
  - **確認ポイント**: 既存 `ItemDetail` に item_files/遷移情報が付加されていること、`calibre_book_id IS NOT NULL && file_type=pdf` の条件で付加されること
- 🟡 信頼性レベル: タスク定義書 単体テスト要件 テストケース4（TC-020-02）に対応。URL構築方式未確定のため遷移情報の具体形は実装時確定（要件定義書 第3章 L97）

### TC-020-N03: calibre_book_id未設定PDFは遷移情報を付加しない（DB依存・統合）

- **テスト名**: calibre_book_id=NULLのPDFは詳細APIで遷移情報を付加されない
  - **何をテストするか**: `calibre_book_id IS NULL` のPDF（PATCH未実行）について、詳細APIが遷移情報を付加しないこと
  - **期待される動作**: 200、該当ファイルの `calibre_book_id` は null（遷移情報なし、または遷移情報フィールドがnull/省略）
- **入力値**: itemを作成 → pdf item_fileを作成（calibre_book_id未設定のまま）→ `GET /items/{item_id}`
  - **入力データの意味**: 付加条件 `calibre_book_id IS NOT NULL` の境界（負側）を検証
- **期待される結果**: `StatusCode::OK`、遷移情報が付加されない
  - **期待結果の理由**: 付加条件を満たさないものに誤って遷移情報を付けないことの保証
- **テストの目的**: 詳細API付加条件（NOT NULL）の確認
  - **確認ポイント**: 条件分岐が `calibre_book_id IS NOT NULL` で正しく機能すること
- 🟡 信頼性レベル: 要件定義書 2.3 L77（`calibre_book_id IS NOT NULL` かつ `file_type='pdf'` 条件）からの妥当な推測

---

## 3. 異常系テストケース（エラーハンドリング）

### TC-020-U01: ApiErrorCode::FileNotFound が404・FILE_NOT_FOUND にマッピングされる（DB非依存）

- **テスト名**: FileNotFoundが404 NOT_FOUND・ワイヤーコードFILE_NOT_FOUNDになる
  - **エラーケースの概要**: 新規追加するエラーコードvariantのHTTPステータス・ワイヤーコード文字列マッピング
  - **エラー処理の重要性**: 404応答の一貫性。既存ItemNotFound("ITEM_NOT_FOUND")と文字列が異なる専用コードであることを保証
- **入力値**: `ApiError::new(ApiErrorCode::FileNotFound, "ファイルが見つかりません")`
  - **不正な理由**: （正常系の構築だがエラー応答の検証）file_id不存在時に返すエラー
  - **実際の発生シナリオ**: 存在しないfile_idへのPATCH時にハンドラがこのエラーを構築する
- **期待される結果**: `response.status() == StatusCode::NOT_FOUND`、`err.error.code == "FILE_NOT_FOUND"`
  - **エラーメッセージの内容**: 内部情報を含まないユーザー向けメッセージ
  - **システムの安全性**: 新variantが500（デフォルト誤マッピング）に落ちないこと
- **テストの目的**: 新規エラーコードのマッピング検証
  - **品質保証の観点**: TASK-0024/0027同様、enum追加時の取りこぼし防止
- 🟡 信頼性レベル: 要件定義書 2.2 L67-71・第6章 L170（response.rsに未定義・新規追加必要）に対応。既存 `file_storage_write_failed_returns_500_with_expected_wire_code` パターン踏襲

### E01 / TC-020-E01: file_type≠pdf で VALIDATION_ERROR(400) を返す（DB依存・ルーター統合）

- **テスト名**: file_type=image(photo)のレコードへのPATCHは400 VALIDATION_ERRORでレコードは更新されない
  - **エラーケースの概要**: 対象 item_files の `file_type` が `pdf` 以外（例: image/photo）
  - **エラー処理の重要性**: PDF以外にCalibre書籍IDを紐付けるのは仕様違反。誤った関連付けを防ぐ
- **入力値**:
  - 事前準備: itemを作成 → `file_type=image` の item_fileを作成
  - URI: `PATCH /items/{item_id}/files/{file_id}/calibre-link`、Body `{ "calibre_book_id": "calibre-12345" }`
  - **不正な理由**: calibre_book_id は PDF専用フィールド（database-schema.sql・REQ-103）
  - **実際の発生シナリオ**: 連携処理が誤って画像ファイルIDを指定した場合
- **期待される結果**: `StatusCode::BAD_REQUEST`、`error.code == "VALIDATION_ERROR"`、該当行の `calibre_book_id` は更新されない（NULLのまま）
  - **エラーメッセージの内容**: 「pdf以外のファイルには紐付けできない」旨
  - **システムの安全性**: 拒否時にDBが変更されないこと
- **テストの目的**: file_type検証（400）の確認・404との区別
  - **品質保証の観点**: 不存在(404)ではなく「存在するがpdfでない(400)」を正しく区別すること
- 🔵 信頼性レベル: タスク定義書 テストケース2（L73-76）・要件定義書 4.2 E01 L113 に直接対応

### E02 / TC-020-E02a: 存在しないfile_idで FILE_NOT_FOUND(404)（DB依存・ルーター統合）

- **テスト名**: 存在しないfile_idへのPATCHは404 FILE_NOT_FOUNDを返す
  - **エラーケースの概要**: `file_id` がitem_filesテーブルに存在しない
  - **エラー処理の重要性**: 存在しないリソースへの更新を明示的に拒否する
- **入力値**:
  - 事前準備: itemのみ作成（item_fileは作成しない）
  - URI: `PATCH /items/{item_id}/files/{random_uuid}/calibre-link`、Body `{ "calibre_book_id": "calibre-12345" }`
  - **不正な理由**: 指定file_idに対応するレコードが存在しない
  - **実際の発生シナリオ**: 削除済みファイルや誤ったIDを指定した場合
- **期待される結果**: `StatusCode::NOT_FOUND`、`error.code == "FILE_NOT_FOUND"`
  - **エラーメッセージの内容**: 「ファイルが見つかりません」旨
  - **システムの安全性**: DBは変更されない
- **テストの目的**: 不存在file_id時の404確認
  - **品質保証の観点**: 0行ヒット時に404を返すこと
- 🔵 信頼性レベル: タスク定義書 テストケース3（L78-81）・要件定義書 4.2 E02 L114 に直接対応

### TC-020-E02b: item_idとfile_idの紐付け不一致で FILE_NOT_FOUND(404)（DB依存・ルーター統合）

- **テスト名**: 別itemに属するfile_idを指定すると404 FILE_NOT_FOUNDを返す
  - **エラーケースの概要**: file_idは実在するが、URLの `item_id` とは別のitemに属する（紐付け不一致）
  - **エラー処理の重要性**: 他itemのファイルを誤更新するのを防ぐ（権限/整合性境界）
- **入力値**:
  - 事前準備: itemA作成 → itemAにpdf item_file（file_id_A）作成。別途itemB作成
  - URI: `PATCH /items/{item_B_id}/files/{file_id_A}/calibre-link`、Body `{ "calibre_book_id": "calibre-12345" }`
  - **不正な理由**: file_id_A は itemA に属し、itemB には属さない（WHERE id=$2 AND item_id=$3 が0行）
  - **実際の発生シナリオ**: パスの組み合わせを誤った場合
- **期待される結果**: `StatusCode::NOT_FOUND`、`error.code == "FILE_NOT_FOUND"`、file_id_A の `calibre_book_id` は更新されない
  - **エラーメッセージの内容**: 「ファイルが見つかりません」旨
  - **システムの安全性**: 他itemのファイルが変更されないこと
- **テストの目的**: item_id+file_id紐付け検証（404）の確認
  - **品質保証の観点**: クロスitemの誤更新防止
- 🔵 信頼性レベル: タスク定義書 テストケース3（L78-81「別のitem_idに属するfile_id」）・要件定義書 4.2 E02 に直接対応

### E03 / TC-020-E03: calibre_book_idが空文字/空白のみで VALIDATION_ERROR(400)（DB非依存 + ルーター統合）

- **テスト名**: calibre_book_idが空文字/空白のみのリクエストは400 VALIDATION_ERROR
  - **エラーケースの概要**: `calibre_book_id` が空文字 `""` または空白のみ `"   "`
  - **エラー処理の重要性**: 無意味な空IDの紐付けを防ぐ（必須・非空制約）
- **入力値**:
  - DB非依存: `parse_update_calibre_link_request(UpdateCalibreLinkRequest { calibre_book_id: "".to_string() })` および `"   ".to_string()`
  - ルーター統合: Body `{ "calibre_book_id": "" }`
  - **不正な理由**: trim後に空文字は必須非空制約に違反（要件定義書 2.1 L42）
  - **実際の発生シナリオ**: 連携処理がID未取得のまま空文字を送信した場合
- **期待される結果**: `Err(ApiError{ code: VALIDATION_ERROR })` / ルーター経由で `StatusCode::BAD_REQUEST`
  - **エラーメッセージの内容**: 「calibre_book_idは必須です」旨
  - **システムの安全性**: DBは変更されない
- **テストの目的**: 空文字バリデーション（400）の確認
  - **品質保証の観点**: 既存 `parse_create_item_file_request_rejects_empty_path` と対称な必須検証
- 🟡 信頼性レベル: 要件定義書 4.2 E03 L115・2.1 L42（trim後空文字はVALIDATION_ERROR）に対応。既存path空文字検証パターンと対称

### E05 / TC-020-E05: 不正JSON・calibre_book_idキー欠落で VALIDATION_ERROR(400)（DB非依存 + ルーター統合）

- **テスト名**: calibre_book_idキー欠落/型不正のボディは400 VALIDATION_ERROR
  - **エラーケースの概要**: リクエストボディに `calibre_book_id` キーが無い、または型が不正（数値等）
  - **エラー処理の重要性**: デシリアライズ失敗を統一エラーで返し、内部パニックを防ぐ
- **入力値**:
  - キー欠落: `serde_json::json!({})` を `deserialize_request::<UpdateCalibreLinkRequest>` に渡す
  - 型不正: `{ "calibre_book_id": 123 }`
  - ルーター統合: Body `{}`（キー欠落）
  - **不正な理由**: 必須キー欠落/型不一致でserdeデシリアライズが失敗する
  - **実際の発生シナリオ**: クライアント実装ミスや誤ったContent
- **期待される結果**: `deserialize_request` が `Err(VALIDATION_ERROR)` / ルーター経由で `StatusCode::BAD_REQUEST`
  - **エラーメッセージの内容**: 「リクエストの形式が不正です: ...」（既存 deserialize_request メッセージ）
  - **システムの安全性**: パニックせず400で返す
- **テストの目的**: デシリアライズエラーハンドリングの確認
  - **品質保証の観点**: 既存 `deserialize_request`（item.rs L139-148）の挙動踏襲
- 🟡 信頼性レベル: 要件定義書 4.2 E05 L117・既存 deserialize_request 実装に対応

### TC-020-R02: リポジトリ層 — 不存在item_id+file_idでNone/NotFoundを返す（DB依存・統合）

- **テスト名**: update_calibre_linkが不存在の組み合わせで404相当（None/Err）を返す
  - **エラーケースの概要**: 対象行が存在しない場合のリポジトリ戻り値
  - **エラー処理の重要性**: ハンドラの404判定の基礎となる
- **入力値**: ランダムな `item_id`/`file_id`（DBに存在しない）、`calibre_book_id="calibre-12345"`
  - **不正な理由**: 対象行が0件
  - **実際の発生シナリオ**: 存在しないIDの指定
- **期待される結果**: リポジトリ設計に応じ `Ok(None)` もしくは `Err(FileNotFound)`（実装方針に合わせる。ハンドラで最終的に404）
  - **エラーメッセージの内容**: 該当なし（リポジトリ戻り値）
  - **システムの安全性**: UPDATE が実行されないこと
- **テストの目的**: リポジトリの不存在ハンドリング確認
  - **確認ポイント**: 2段階判定の (b) 0行→不存在 が機能すること
- 🟡 信頼性レベル: 要件定義書 第3章 L91-92（fetch_optionalで0行→None、ハンドラで404）からの妥当な推測

### TC-020-R03: リポジトリ層 — 存在するがfile_type≠pdfで400相当を返す（DB依存・統合）

- **テスト名**: update_calibre_linkが存在するimage行に対しValidationError相当を返しUPDATEしない
  - **エラーケースの概要**: 行は存在するが `file_type=image`
  - **エラー処理の重要性**: 404と400を区別する2段階判定の中核
- **入力値**: 事前insertした `file_type=image` 行の `item_id`/`file_id`、`calibre_book_id="calibre-12345"`
  - **不正な理由**: pdf以外への紐付けは仕様違反
  - **実際の発生シナリオ**: E01のリポジトリ層相当
- **期待される結果**: `Err(ApiError{ code: VALIDATION_ERROR })`（またはハンドラで400に変換できる戻り値）、該当行は未更新
  - **エラーメッセージの内容**: pdf以外拒否の旨
  - **システムの安全性**: UPDATE 実行されない
- **テストの目的**: 2段階判定 (c) file_type≠pdf→400 の確認
  - **確認ポイント**: 「存在する画像行」が404でなく400になること（不存在と区別）
- 🔵 信頼性レベル: 要件定義書 第3章 L92（2段階方式）・タスク定義書 テストケース2 に直接対応

---

## 4. 境界値テストケース（最小値、最大値、null等）

### E04 / TC-020-B01: :id が不正UUID形式で VALIDATION_ERROR(400)（DB非依存/ルーター統合）

- **テスト名**: :idがUUID形式でない場合は400 VALIDATION_ERROR
  - **境界値の意味**: パスパラメータ妥当性の境界（パース可否）
  - **境界値での動作保証**: UUIDパース前段での早期リターン
- **入力値**: URI `PATCH /items/not-a-uuid/files/{valid_uuid}/calibre-link`（または `parse_item_id("not-a-uuid")`）
  - **境界値選択の根拠**: 既存 `post_item_file_upload_with_invalid_uuid_path_returns_400` と対称
  - **実際の使用場面**: 不正な手入力URL・実装バグ
- **期待される結果**: `StatusCode::BAD_REQUEST` / `parse_item_id` が `Err(VALIDATION_ERROR)`
  - **境界での正確性**: パース失敗時に確実に400へ
  - **一貫した動作**: 既存parse_item_idと同一挙動
- **テストの目的**: :id UUID検証の確認
  - **堅牢性の確認**: 不正入力でパニックせず400
- 🔵 信頼性レベル: 要件定義書 4.2 E04 L116・既存 parse_item_id・既存 TC-019-E05 に対応

### TC-020-B02: :file_id が不正UUID形式で VALIDATION_ERROR(400)（DB非依存/ルーター統合）

- **テスト名**: :file_idがUUID形式でない場合は400 VALIDATION_ERROR
  - **境界値の意味**: 2つ目のパスパラメータ（file_id）の妥当性境界
  - **境界値での動作保証**: 新規 `parse_file_id`（または同等検証）の早期リターン
- **入力値**: URI `PATCH /items/{valid_uuid}/files/not-a-uuid/calibre-link`（または `parse_file_id("not-a-uuid")`）
  - **境界値選択の根拠**: :id検証と対称な、file_id専用の検証境界（要件定義書 2.1 L37 で新規追加が必要と明記）
  - **実際の使用場面**: 不正なfile_id指定
- **期待される結果**: `StatusCode::BAD_REQUEST` / `Err(VALIDATION_ERROR)`
  - **境界での正確性**: file_idパース失敗で確実に400
  - **一貫した動作**: :idと同一の検証挙動
- **テストの目的**: :file_id UUID検証の確認（新規検証関数）
  - **堅牢性の確認**: file_id不正でもパニックせず400
- 🟡 信頼性レベル: 要件定義書 2.1 L37・4.2 E04 L116（parse_file_id相当を追加）からの妥当な推測

### TC-020-B03: calibre_book_idの最小有効長（1文字）で更新成功（DB依存・統合）

- **テスト名**: 1文字のcalibre_book_idでも200で更新される
  - **境界値の意味**: 非空制約の下限（trim後1文字）
  - **境界値での動作保証**: 空(NG)と1文字(OK)の境界が正しいこと
- **入力値**: pdf行に対し Body `{ "calibre_book_id": "a" }`
  - **境界値選択の根拠**: 空文字(拒否)の直上の最小受理値
  - **実際の使用場面**: 極端に短い書籍IDがあり得る場合
- **期待される結果**: `StatusCode::OK`、`data.calibre_book_id == "a"`
  - **境界での正確性**: 1文字を誤って空扱いしないこと
  - **一貫した動作**: 空(400)と1文字(200)の境界が明確
- **テストの目的**: 非空制約の下限境界確認
  - **堅牢性の確認**: 最小有効入力で安定動作
- 🟡 信頼性レベル: 非空制約（要件定義書 2.1 L42）からの妥当な境界値推測

### TC-020-B04: 前後空白を含む calibre_book_id の扱い（DB依存・統合）

- **テスト名**: 前後に空白を含むが中身が非空のcalibre_book_idは受理される
  - **境界値の意味**: trim検証と保存値の境界（trimは検証用、保存は原文 or trim後かを確定）
  - **境界値での動作保証**: `"  calibre-12345  "` が「空白のみ(400)」ではなく非空として受理されること
- **入力値**: Body `{ "calibre_book_id": "  calibre-12345  " }`
  - **境界値選択の根拠**: trim後非空（E03の空白のみとの対比）
  - **実際の使用場面**: 連携処理が余分な空白を付与した場合
- **期待される結果**: `StatusCode::OK`。保存値は**原文保持（trimしない）**方針で確定した
  - **境界での正確性**: 「空白のみ(400)」と「空白付き非空(200)」を区別
  - **一貫した動作**: trim判定（`parse_update_calibre_link_request`内の`trim().is_empty()`）は
    バリデーションのみに使われ、保存値（`UpdateCalibreLinkRequest.calibre_book_id`そのもの）は
    trimせず原文のままリポジトリへ渡される
- **テストの目的**: trim検証と保存値方針の境界確認
  - **堅牢性の確認**: 空白混在入力での一貫動作
- 🔵 信頼性レベル: **Greenフェーズで決定・実装確定**。既存`models/item_file.rs`の
  `parse_create_item_file_request`（pathフィールド）と同様に「trim判定はバリデーションのみに使用し、
  構造体の値自体は変更しない」という既存コードパターンに倣う方針として確定した
  （`backend/mediavault-api/src/models/item_file.rs` `parse_update_calibre_link_request`実装に直接対応）。
  要件定義書には保存方針の明記がないため、既存実装パターンとの一貫性を理由に決定した（旧🔴信号を解消）。

### TC-020-B05: 複数PDFファイルのうちcalibre_book_id設定済みのみ遷移情報付加（DB依存・統合）

- **テスト名**: 詳細APIで複数ファイル混在時、calibre_book_id設定済みPDFのみ遷移情報が付く
  - **境界値の意味**: 詳細レスポンス内の複数item_files混在（pdf+image、設定済み+未設定）の境界
  - **境界値での動作保証**: 付加条件が行ごとに正しく適用されること
- **入力値**: 1つのitemに (a)calibre設定済みpdf, (b)未設定pdf, (c)image を作成 → `GET /items/{item_id}`
  - **境界値選択の根拠**: 付加条件 `file_type=pdf AND calibre_book_id IS NOT NULL` の行単位適用を検証
  - **実際の使用場面**: 1アイテムに複数ファイルがある一般的な構成
- **期待される結果**: `StatusCode::OK`、(a)のみ遷移情報付き、(b)(c)は付かない
  - **境界での正確性**: 行ごとの条件分岐が正しいこと
  - **一貫した動作**: 1つのレスポンス内で条件が混在しても正しく判定
- **テストの目的**: 詳細API付加条件の行単位適用確認
  - **堅牢性の確認**: 混在ケースでの正確な分岐
- 🟡 信頼性レベル: 要件定義書 2.3 L77・TC-020-02 からの妥当な推測（行単位適用の検証）

---

## 5. テストケース一覧（サマリー）

| ID | カテゴリ | 概要 | 層 | DB | 信頼性 |
|---|---|---|---|---|---|
| TC-020-U01 | 異常系 | FileNotFound→404/FILE_NOT_FOUNDマッピング | response.rs | 不要 | 🟡 |
| TC-020-U02 | 正常系 | UpdateCalibreLinkRequest デシリアライズ | item_file.rs | 不要 | 🔵 |
| TC-020-U03 | 正常系 | 非空calibre_book_id 検証通過 | item_file.rs | 不要 | 🔵 |
| TC-020-R01 | 正常系 | リポジトリpdf更新（RETURNING） | repo | 必要 | 🔵 |
| TC-020-R02 | 異常系 | リポジトリ不存在→None/NotFound | repo | 必要 | 🟡 |
| TC-020-R03 | 異常系 | リポジトリ image行→400相当（404と区別） | repo | 必要 | 🔵 |
| TC-020-01 | 正常系 | PATCH 200・更新後レコード返却 | handler/router | 必要 | 🔵 |
| TC-020-N01 | 正常系 | 冪等性（2回PATCH同結果） | router | 必要 | 🟡 |
| TC-020-N02 | 正常系 | 既存値の上書き更新 | router | 必要 | 🟡 |
| TC-020-02 | 正常系 | 詳細APIにCalibre-Web遷移情報付加 | items handler/router | 必要 | 🟡 |
| TC-020-N03 | 正常系 | NULL値PDFは遷移情報を付加しない | items handler/router | 必要 | 🟡 |
| TC-020-E01 | 異常系 | file_type≠pdf→400 VALIDATION_ERROR | router | 必要 | 🔵 |
| TC-020-E02a | 異常系 | 不存在file_id→404 FILE_NOT_FOUND | router | 必要 | 🔵 |
| TC-020-E02b | 異常系 | 紐付け不一致→404 FILE_NOT_FOUND | router | 必要 | 🔵 |
| TC-020-E03 | 異常系 | calibre_book_id空文字/空白→400 | item_file.rs/router | 一部必要 | 🟡 |
| TC-020-E05 | 異常系 | キー欠落/型不正→400 | item_file.rs/router | 一部必要 | 🟡 |
| TC-020-B01 | 境界値 | :id不正UUID→400 | router | 不要/必要 | 🔵 |
| TC-020-B02 | 境界値 | :file_id不正UUID→400 | router | 不要/必要 | 🟡 |
| TC-020-B03 | 境界値 | 1文字calibre_book_id→200 | router | 必要 | 🟡 |
| TC-020-B04 | 境界値 | 前後空白付き非空→200（保存方針は要確定） | router | 必要 | 🔴 |
| TC-020-B05 | 境界値 | 複数ファイル混在時の行単位付加 | items handler/router | 必要 | 🟡 |

**合計テストケース数: 21件**
- 正常系: 8件（U02, U03, R01, 01, N01, N02, 02, N03）
- 異常系: 8件（U01, R02, R03, E01, E02a, E02b, E03, E05）
- 境界値: 5件（B01, B02, B03, B04, B05）

**信頼性分布**: 🔵 8件 / 🟡 12件 / 🔴 1件

---

## 6. テストケース実装時の日本語コメント指針

### テストケース開始時のコメント（例: TC-020-01）

```rust
// 【テスト目的】: PATCH /items/:id/files/:file_id/calibre-link がfile_type=pdfのレコードの
//                 calibre_book_idを更新し200で更新後レコードを返すことを確認する
// 【テスト内容】: 実在itemにpdf item_fileを作成し、ルーター経由でcalibre-linkをPATCHする
// 【期待される動作】: 200・data.calibre_book_id=="calibre-12345"・data.file_type=="pdf"
// 🔵 信頼性レベル: タスク定義書 テストケース1（TC-020-01）に直接対応
// 【Red期待】: ルート未登録・update_calibre_link_handler未実装のため404/コンパイルエラーとなる想定
```

### Given（準備フェーズ）

```rust
// 【テストデータ準備】: insert_test_item でitemを作成し、file_type=pdfのitem_fileをINSERTする
// 【初期条件設定】: 対象item_fileのcalibre_book_idはNULL（未設定）の状態
// 【前提条件確認】: DATABASE_URL設定済み・docker-compose db起動済み（#[ignore]統合テスト）
```

### When（実行フェーズ）

```rust
// 【実際の処理実行】: app.oneshot で PATCH /items/{item_id}/files/{file_id}/calibre-link を呼ぶ
// 【処理内容】: ハンドラ→parse_item_id/parse_file_id→deserialize→repository::update_calibre_link
// 【実行タイミング】: 対象行作成直後（NULL状態）に1回実行
```

### Then（検証フェーズ）

```rust
// 【結果検証】: HTTPステータスとレスポンスボディのcalibre_book_idを検証する
// 【期待値確認】: 200・更新値が反映されていること
// 【品質保証】: 完了条件「更新後レコードを200で返す」を保証する
assert_eq!(response.status(), StatusCode::OK); // 【確認内容】: pdfレコードのcalibre-link更新が200で受理されることを確認 🔵
```

### セットアップヘルパー（既存パターン流用）

```rust
// 【テスト用ヘルパー】: test_app_state() / insert_test_item() は handlers/item_files.rs の
//                       既存ヘルパーと同一パターン。pdf item_file作成用ヘルパーを追加する想定
async fn insert_pdf_item_file(db: &PgPool, item_id: Uuid) -> Uuid {
    // INSERT INTO item_files (item_id, path, label, file_type, calibre_book_id)
    // VALUES ($1, '/srv/files/pdf/example.pdf', '本編PDF', 'pdf', NULL) RETURNING id
}
```

---

## 7. 要件定義との対応関係

- **参照した機能概要**: 要件定義書 第1章（calibre-link機能の概要・REQ-103）
- **参照した入力・出力仕様**: 要件定義書 第2章（2.1入力・2.2出力・2.3データフロー）
- **参照した制約条件**: 要件定義書 第3章（2段階検証順序 L92・FILE_NOT_FOUND新規追加 L71・エラーハンドリング）
- **参照した使用例**: 要件定義書 第4章（TC-020-01/02, E01〜E05・データフロー）
- **参照したタスク定義**: `docs/tasks/mediavault-backend/TASK-0028.md`（単体テスト要件 テストケース1〜4 L66-86・完了条件 L24-29）
- **参照した既存実装**:
  - `backend/mediavault-api/src/models/response.rs`（ApiErrorCode追加パターン・既存エラーマッピングテスト）
  - `backend/mediavault-api/src/models/item_file.rs`（ItemFile/FileType/DTO検証パターン）
  - `backend/mediavault-api/src/repositories/item_file_repository.rs`（db_error/item_exists/test_pool）
  - `backend/mediavault-api/src/handlers/item_files.rs`（test_app_state/insert_test_item/oneshotルーター統合テストパターン）
  - `backend/mediavault-api/src/handlers/items.rs`（get_item_handler・deserialize_request・parse_item_id）

---

## 8. 品質判定

```
✅ 高品質:
- テストケース分類: 正常系8・異常系8・境界値5でバランス良く網羅。リポジトリ層/ハンドラ層/モデル層を分離
- 期待値定義: 各ケースでHTTPステータス・error.code・DB状態を明確化
- 技術選択: Rust + #[test]/#[tokio::test] + tower::oneshot（既存構成と完全一致）で確定
- 実装可能性: 既存テストヘルパー（test_app_state/insert_test_item/test_pool）を流用可能
- 信頼性レベル: 🔵8・🟡12・🔴1（赤はTC-020-B04のtrim保存方針のみ・実装時確定で解消可能）

⚠️ 実装時に確定が必要な項目:
1. FILE_NOT_FOUNDエラーコードの新規追加（TC-020-U01の前提）
2. parse_file_id相当の新規検証関数（TC-020-B02の前提）
3. リポジトリの2段階判定（取得→pdf検証→更新）の戻り値型（TC-020-R02/R03）
4. 詳細API遷移情報の具体構造（TC-020-02/N03/B05）— URL構築方式未確定のため独立小型構造体で実装
5. calibre_book_idのtrim保存方針（TC-020-B04・🔴）

総合評価: 高品質（コア機能ケースは🔵、拡張・新規追加部分は🟡で実装時に確定）
```

---

## 次のステップ

次のお勧めステップ: `/tsumiki:tdd-red mediavault-backend TASK-0028` でRedフェーズ（失敗テスト作成）を開始します。
