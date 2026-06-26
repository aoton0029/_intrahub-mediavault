# TDD開発ノート: TASK-0028

## TASK-0028 概要

**タスク**: `PATCH /items/:id/files/:file_id/calibre-link` エンドポイント実装（TDD）

**目的**: item_filesレコード（`file_type=pdf`のみ）の`calibre_book_id`を更新し、Calibre-Web連携情報をアイテム詳細APIのレスポンスに含める。

**工数**: 3時間 | **信頼性**: 🔵🔵🔵-🟡(TC-020-02部分)

---

## 1. 技術スタック

### 使用言語・フレームワーク
- **言語**: Rust edition 2024
- **Webフレームワーク**: Axum 0.8.9 (HTTP/JSONレスポンス)
- **データベース**: PostgreSQL (sqlx 0.8 with macros)
- **非同期ランタイム**: Tokio 1.52.3 (full)
- **シリアライゼーション**: serde 1.0.228 + serde_json 1.0.150
- **UUID・日時**: uuid 1 (v4, serde) + chrono 0.4 (serde)
- **エラーハンドリング**: 統一`ApiError`型 + `ApiErrorCode` enum
- **ログ**: tracing + tracing-subscriber
- **参照元**: backend/mediavault-api/Cargo.toml

### アーキテクチャパターン
- **4層構造**: Handlers → Services(optional) → Repositories → Models
- **ハンドラ層**: リクエスト/レスポンス処理、入力検証
- **リポジトリ層**: sqlx::QueryBuilder による動的SQL構築、DB エラーを`ApiError`に統一変換
- **モデル層**: sqlx::FromRow実装、Serde Serialize/Deserializeスタック
- **参照元**: docs/design/mediavault-backend/architecture.md

### DBトリガー・制約
- `trg_items_updated_at` (BEFORE UPDATE on items): items.updated_atを自動更新
- `item_files`テーブル: `id(UUID PK), item_id(FK), path, label(nullable), file_type(enum), calibre_book_id(nullable), created_at`
- item_id+file_idの紐付け検証は`item_files`テーブルそのもので実施（外部キー制約）
- 参照元: docs/design/mediavault-backend/database-schema.sql

---

## 2. 開発ルール・コーディング規約

### エラーハンドリング規約
- **統一エラー型**: `ApiError::new(code, message)` （src/models/response.rs）
- **エラーコード一覧**: ValidationError(400), Unauthorized(401), ItemNotFound(404), UnprocessableEntity(422), InternalError(500), ExternalApiError(502), FileStorageWriteFailed(500)他
- **DB エラー変換**: repository層で`db_error(err: sqlx::Error) -> ApiError`関数を経由し、`INTERNAL_ERROR/500`に統一変換（sqlx内部エラー・接続エラー情報をクライアントに漏らさない）
- **参照元**: backend/mediavault-api/src/repositories/item_file_repository.rs L12-19, backend/mediavault-api/src/models/response.rs L107-140

### リポジトリ層の実装パターン
- **QueryBuilder方式**: sqlx::QueryBuilder<'_, Postgres>で動的SQL構築（SET句の動的カラム追加など）
- **fetch_one/fetch_optional**: Single Row返却、存在しない場合はNone返却（404判定はハンドラ側）
- **RETURNING句**: UPDATE文の結果をそのまま構造体にマッピング（FROM ROW実装ケース）
- **参照元**: backend/mediavault-api/src/repositories/item_repository.rs, backend/mediavault-api/src/repositories/item_file_repository.rs

### テスト規約
- **配置**: 実装ファイル末尾 `#[cfg(test)] mod tests { ... }` インライン配置
- **ユニットテスト**: `#[test]` (DB非依存)、`cargo test -p mediavault-api` で実行
- **統合テスト**: `#[tokio::test] #[ignore]`、DATABASE_URL環境変数使用、`cargo test -- --ignored`で実行
- **信頼性マーク**: 各テスト関数に 🔵🟡🔴 レベル・【テスト目的】【テスト内容】【期待される動作】【確認内容】コメント記載
- **モック**: wiremock 0.6 (HTTP Mock サーバー、DB接続不要なテストの場合)、tempfile 3 (ファイルシステムテスト)
- **参照元**: backend/mediavault-api/Cargo.toml [dev-dependencies], backend/mediavault-api/src/handlers/item_files.rs #[cfg(test)] L181+

### ハンドラのリクエスト処理
- **パスパラメータ**: `Path<String>`extractorで取得、`parse_item_id(&str) -> Result<Uuid, ApiError>`等の検証関数を経由
- **リクエストボディ**: `Json<serde_json::Value>`で受け取り、`deserialize_request(value) -> Result<T, ApiError>`で型変換（デシリアライズエラーハンドリング）
- **バリデーション**: 検証関数（`parse_create_item_file_request`等）で空文字・不正値をチェック、エラーはVALIDATION_ERROR(400)
- **参照元**: backend/mediavault-api/src/handlers/items.rs, backend/mediavault-api/src/handlers/item_files.rs

### 命名規則
- **モデル**: 単数形（Item, ItemFile, CreateItemFileRequest）
- **リポジトリ関数**: 動詞 + 目的語（create_item_file, get_item_by_id, update_calibre_link等）
- **ハンドラ関数**: 動詞 + 名詞 + _handler（create_item_file_handler, update_calibre_link_handler等）
- **参照元**: 既存実装パターン

---

## 3. 関連実装

### TASK-0026 (item_files基盤)
- **POST /items/:id/files**: パス指定方式でitem_filesレコード作成
- **models/item_file.rs**: FileType(Pdf/Image/Other), ItemFile構造体、CreateItemFileRequest
- **repositories/item_file_repository.rs**: create_item_file関数、item_exists関数、db_error変換
- **handlers/item_files.rs**: create_item_file_handler, upload_item_file_handler
- **学習点**: item_filesテーブルの構造、calibre_book_idはNULL固定設計、file_typeの enum 検証パターン
- **参照元**: backend/mediavault-api/src/models/item_file.rs, backend/mediavault-api/src/repositories/item_file_repository.rs, backend/mediavault-api/src/handlers/item_files.rs

### TASK-0012 (PATCH /items/:id)
- **部分更新パターン**: UpdateItemRequest型、parse_update_item_request検証、QueryBuilderで動的SET句構築
- **学習点**: 全フィールドNoneの場合は現在状態を返す、updated_atはトリガーで自動更新（SET句に含めない）
- **参照元**: backend/mediavault-api/src/models/item.rs (L104-119 UpdateItemRequest), backend/mediavault-api/src/handlers/items.rs (PATCH実装)

### TASK-0023/TASK-0024 (外部API連携)
- **エラーマッピング**: ExternalSearchError → ApiError(ApiErrorCode::ExternalApiTimeout)への変換パターン
- **学習点**: 外部システムとの境界でのエラー型変換、専用的エラーコード導入
- **参照元**: backend/mediavault-api/src/models/response.rs L95-97, backend/mediavault-api/src/models/external_search.rs

### TASK-0025 (POST /items/import)
- **リクエストボディ処理**: deserialize_request パターン
- **参照元**: backend/mediavault-api/src/handlers/items.rs (import_item_handler)

---

## 4. 設計文書・API仕様

### API エンドポイント (PATCH /items/:id/files/:file_id/calibre-link)
**信頼性**: 🔵 api-endpoints.md より

```
PATCH /items/:id/files/:file_id/calibre-link
Content-Type: application/json

Request:
{
  "calibre_book_id": "calibre-12345"
}

Response (200):
{
  "success": true,
  "data": {
    "id": "uuid",
    "item_id": "uuid",
    "path": "/srv/files/pdf/example.pdf",
    "label": "本編PDF",
    "file_type": "pdf",
    "calibre_book_id": "calibre-12345",
    "created_at": "2026-06-26T10:00:00"
  }
}

Error (400 - file_type != pdf):
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "..."
  }
}

Error (404):
{
  "success": false,
  "error": {
    "code": "FILE_NOT_FOUND",
    "message": "..."
  }
}
```

**参照元**: docs/design/mediavault-backend/api-endpoints.md

### データベーススキーマ
- `item_files`: id(UUID PK), item_id(FK to items), path(VARCHAR), label(nullable), file_type(enum: pdf/image/other), calibre_book_id(nullable), created_at
- **calibre_book_id**: NULL許容、Calibre-Web側でのPDF取込完了後に本エンドポイントで更新される想定
- **参照元**: docs/design/mediavault-backend/database-schema.sql

### アイテム詳細API (GET /items/:id) レスポンス拡張
**信頼性**: 🟡 TC-020-02 より

`calibre_book_id`が設定済みのPDFファイル情報について、Calibre-Web遷移用情報（calibre_book_id含む）をレスポンスに付加する。具体的なURL構築方式は環境変数設定値を用いるテンプレート形式を想定し、実装時は変更容易な構造（独立した小型構造体）で定義。

**参照元**: docs/tasks/mediavault-backend/TASK-0028.md L58-64

---

## 5. テスト関連情報

### テストフレームワーク・設定
- **ユニットテスト**: Rust組込の`#[test]`、`cargo test -p mediavault-api`で実行
- **統合テスト**: Tokio Runtime + `#[tokio::test]`、`#[ignore]`マークで別実行
- **HTTP Mock**: wiremock 0.6 (API呼び出しのモック、本タスク範囲外)
- **ファイルシステム**: tempfile 3 (一時ディレクトリテスト)
- **DB接続**: PgPool + DATABASE_URL環境変数、docker-compose db起動前提
- **参照元**: backend/mediavault-api/Cargo.toml [dev-dependencies], backend/mediavault-api/src/handlers/item_files.rs #[cfg(test)]

### 既存テストディレクトリ構成
```
backend/mediavault-api/src/
├── models/
│   ├── item.rs          // テスト L260+
│   ├── item_file.rs     // テスト L58-135
│   └── response.rs      // テスト L240+
├── repositories/
│   ├── item_repository.rs       // テスト L800+
│   └── item_file_repository.rs  // テスト L65-97
├── handlers/
│   ├── items.rs         // テスト L195+
│   └── item_files.rs    // テスト L181+
└── routes/
    └── mod.rs           // テスト L144+ (test_app_state ヘルパー)
```

### テスト用ヘルパー関数
- `test_pool() -> PgPool`: DATABASE_URL環境変数からDB接続
- `test_app_state() -> AppState`: テスト用AppState(db, internal_api_key)構築
- `multipart_body(...)`: multipart/form-data生ボディ組み立て（TASK-0027パターン）
- `insert_test_item(db) -> Uuid`: テスト用itemをDB作成、UUIDを返す
- **参照元**: backend/mediavault-api/src/handlers/item_files.rs L191-364

### テストケース (TASK-0028より)

#### TC-020-01: 正常系 - calibre_book_id更新 🔵
- **Given**: `file_type=pdf`のitem_filesレコード
- **When**: `PATCH /items/:id/files/:file_id/calibre-link` with `{"calibre_book_id": "calibre-12345"}`
- **Then**: 200応答、対象レコードの calibre_book_id が更新される

#### TC-020-02: 詳細API拡張 - Calibre-Web情報付加 🟡
- **Given**: `calibre_book_id`設定済みのPDFを持つアイテム
- **When**: GET /items/:id（詳細取得）
- **Then**: レスポンスの該当ファイル情報に calibre_book_id（およびCalibre-Web遷移情報）が含まれる

#### E01: file_type != pdf で VALIDATION_ERROR(400) 🔵
- **Given**: `file_type=photo`のitem_filesレコード
- **When**: PATCH /items/:id/files/:file_id/calibre-link
- **Then**: 400 VALIDATION_ERROR応答

#### E02: file_id不存在・紐付け不一致で FILE_NOT_FOUND(404) 🔵
- **Given**: 存在しない file_id、または別の item_id に属する file_id
- **When**: PATCH /items/:id/files/:file_id/calibre-link
- **Then**: 404応答

---

## 6. 注意事項

### 技術的制約
- **calibre_book_idの妥当性**: 本タスク範囲外（Calibre-Web側に実在するIDかどうかは検証しない）
- **TC-020-02の信頼性**: 🟡（Calibre-Web連携の実際のURL構成・認証方式が確定していないため）
- **アイテム詳細APIの拡張**: 既存の詳細取得ハンドラ(handlers/items.rs)を拡張し、item_filesジョイン時にcalibre_book_id情報を付加する設計を想定

### セキュリティ・パフォーマンス
- **認証**: 本エンドポイントは現状、内部API未対応（Phase 2で認証スキーム実装予定）
- **SQLインジェクション**: sqlxのパラメータバイント化により対策済み
- **エラー情報漏洩**: db_error関数でDB内部エラーをマスキング

### ファイルパス規約（相対パス）
すべてのファイルパスはプロジェクトルート（d:\Document\apps\MediaVault）からの相対パスで記載：
- ✅ `backend/mediavault-api/src/models/item_file.rs`
- ✅ `docs/design/mediavault-backend/api-endpoints.md`
- ❌ `d:\Document\apps\MediaVault\backend\mediavault-api\src\models\item_file.rs` (使用禁止)

---

## 参考・関連ファイル一覧

### 実装ファイル（新規/修正）
- `backend/mediavault-api/src/models/item_file.rs` - UpdateCalibreLinkRequest DTO追加（既存ファイル）
- `backend/mediavault-api/src/repositories/item_file_repository.rs` - update_calibre_link関数追加（既存ファイル）
- `backend/mediavault-api/src/handlers/item_files.rs` - update_calibre_link_handler関数追加、詳細APIレスポンス拡張（既存ファイル）
- `backend/mediavault-api/src/routes/mod.rs` - PATCH /items/:id/files/:file_id/calibre-link ルート登録（既存ファイル）

### 設計・仕様文書
- `docs/design/mediavault-backend/api-endpoints.md` - API仕様
- `docs/design/mediavault-backend/database-schema.sql` - item_filesテーブル定義
- `docs/design/mediavault-backend/architecture.md` - 4層構造・エラーハンドリング
- `docs/spec/mediavault-backend/` - 要件定義（※存在確認: 未発見）

### テスト・参考ファイル
- `backend/mediavault-api/src/handlers/item_files.rs` - テストパターン（multipart_body等のヘルパー、#[ignore]統合テスト）
- `backend/mediavault-api/src/repositories/item_file_repository.rs` - repository層テスト（test_pool, insert_test_itemパターン）
- `backend/mediavault-api/Cargo.toml` - dev-dependencies確認

### 依存タスク
- **前提**: TASK-0026（item_filesモデル/リポジトリ・パス指定方式の基盤）
- **参照**: TASK-0012（PATCH部分更新パターン）, TASK-0024（エラーコード追加パターン）, TASK-0027（ファイルアップロードパターン）

---

## TDD実装フロー

1. **tsumiki:tdd-requirements** → TASK-0028.md + 要件定義書から詳細受け入れ基準生成
2. **tsumiki:tdd-testcases** → TC-020-01/02/E01/E02から包括的テストケース洗い出し
3. **tsumiki:tdd-red** → テストケース実装（全Red確認）
4. **tsumiki:tdd-green** → 最小限の実装でテスト通過
5. **tsumiki:tdd-refactor** → コード品質改善・リファクタリング
6. **tsumiki:tdd-verify-complete** → すべてのテスト通過確認・開発完了

