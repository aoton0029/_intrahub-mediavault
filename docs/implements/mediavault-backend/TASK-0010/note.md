# TASK-0010 開発ノート: GET /items（一覧・絞り込み）実装

## 1. 技術スタック

- **言語・フレームワーク**: Rust + Axum (0.8.9) + sqlx (0.8, postgres, runtime-tokio, macros, chrono, uuid)
- **非同期ランタイム**: tokio (full)
- **レイヤー構成**: routes → handlers → repositories → DB
- **レイヤーパターン**: Layered Architecture（既存実装TASK-0009に準拠）
  - routes: `src/routes/mod.rs` でエンドポイント定義
  - handlers: `src/handlers/items.rs` でビジネスロジック（クエリパラメータ抽出・バリデーション）
  - repositories: `src/repositories/item_repository.rs` でDB操作
  - models: `src/models/item.rs` / `src/models/response.rs` でDTO・レスポンス定義
- **参照元**: `backend/mediavault-api/Cargo.toml`, `backend/mediavault-api/src/main.rs`, `docs/design/mediavault-backend/architecture.md`

## 2. 開発ルール

### プロジェクト共通ルール
- **AGENTS.md**: repo root に見当たらず、`backend/CLAUDE.md` にビルド/テストコマンドの記載あり
  - ビルド: `cargo build -p mediavault-api`
  - テスト: `cargo test --workspace`
  - Docker: `cd backend && cp .env.example .env && docker compose up -d db && docker compose ps`
- **追加ルールディレクトリ**: `./docs/rule` ディレクトリは存在しない

### 実装パターン（TASK-0009より継承）
- **ApiOk/ApiError**: `src/models/response.rs` で定義済み
  - `ApiOk<T> { success: bool, data: T }` → 200ステータス固定（ただし明示的に別ステータスが必要な場合は `(StatusCode::XXX, Json(...))` で組み立て）
  - `ApiError { success: false, error: ApiErrorBody{code, message}, status }`
  - レスポンス形式: `{ "success": true, "data": [...], "pagination": {...} }` （共通フォーマット）
- **トランザクション処理**: items INSERT と詳細テーブル INSERT は同一トランザクション内で実行し、失敗時はロールバック（TASK-0009で実装済み）
- **ハンドラ署名**: `async fn handler(State(state): State<AppState>, ...) -> Result<impl IntoResponse, ApiError>`
- **エラー返却**: `Result<T, ApiError>` 形式でハンドラから返す（Axumが自動的に `IntoResponse` 変換）

### DB操作ルール
- **sqlx マクロ使用**: `sqlx::query!()` / `sqlx::query_as!()` でコンパイル時SQL型チェック（オプション）
- **DB エラー処理**: sqlx::Error を ApiError::InternalError に変換して返す（TASK-0009の `db_error()` 関数を参考に）
- **クエリビルダー**: 動的 WHERE 句構築には `sqlx::QueryBuilder` を使用（TASK-0010で新規に活用）
- **参照元**: `backend/mediavault-api/src/repositories/item_repository.rs`

### コメント・信頼性レベル規約
- 各実装セクションの先頭に機能概要・実装方針・テスト対応を明記
- 信頼性レベル（🔵🟡🔴）を付記し、参照元を示す
- 参照元: 設計文書・タスクファイル・既存実装パターン

## 3. 関連実装（TASK-0008 / TASK-0009 の成果物）

### models/item.rs（既存・変更不要）
- **Enum定義**
  - `MediaType`: `Anime, Movie, Drama, Manga, Novel, Game, AcademicBook, Paper` (snake_case, sqlx型名 `media_type`)
  - `ItemStatus`: `NotStarted, InProgress, Completed` (sqlx型名 `item_status`)
  - `ItemSource`: `Api, Manual` (sqlx型名 `item_source`)
- **Item 構造体** (`sqlx::FromRow`実装済み):
  - id, media_type, title, original_title, description, cover_image_url, release_date, homepage_url
  - status, consumed_date, rating, is_favorite, source, external_id, created_at, updated_at
- **CreateItemRequest 構造体** (TASK-0009):
  - リクエスト body から直接マッピング、`source`/`external_id` は含まれない（ハンドラで付与）
- **parse_create_item_request** 関数: デシリアライズ + `validate_title` 実行
- **validate_title** 関数: 空白のみなら VALIDATION_ERROR
- 参照元: `backend/mediavault-api/src/models/item.rs`

### models/response.rs（既存・変更不要）
- **ApiOk<T>**: `{ success: true, data: T }` (200 固定、別ステータスは明示的に組み立て)
- **ApiError**: `{ success: false, error: { code: string, message: string } }` + `status: StatusCode`
- **ApiErrorCode**: `ValidationError`(400), `Unauthorized`(401), `ItemNotFound`(404), `UnprocessableEntity`(422), `InternalError`(500), `ExternalApiError`(502)
- 参照元: `backend/mediavault-api/src/models/response.rs`

### handlers/items.rs（既存・TASK-0009で create_item_handler 実装済み）
- **create_item_handler** (TASK-0009): POST /items リクエストを処理
- **created_response** (TASK-0009): 201 レスポンス構築
- TASK-0010 で新規に **list_items_handler** を追加する想定
- 参照元: `backend/mediavault-api/src/handlers/items.rs`

### repositories/item_repository.rs（既存・TASK-0009で create_item 実装済み）
- **detail_table_name** 関数: media_type → テーブル名マッピング
- **db_error** 関数: sqlx::Error を ApiError に変換
- **create_item** 関数: トランザクションで items + 詳細テーブルへ INSERT
- TASK-0010 で新規に **list_items** / **count_items** 関数を追加する想定
- 参照元: `backend/mediavault-api/src/repositories/item_repository.rs`

### routes/mod.rs（既存・TASK-0007で骨格実装済み）
- `pub fn build_router(state: AppState) -> Router`: エンドポイント定義
- 現状: `.route("/health", get(...))` + `.route("/items", post(...))`
- TASK-0010 で `.route("/items", get(...))` を追加する想定（同じパス・異なるメソッド）
- 参照元: `backend/mediavault-api/src/routes/mod.rs`

### main.rs（既存・TASK-0007で実装済み）
- **AppState** struct: `{ db: PgPool, internal_api_key: String }`
- **main()**: DATABASE_URL 環境変数から接続プール作成、Axum サーバー起動
- 参照元: `backend/mediavault-api/src/main.rs`

### 未実装（TASK-0010で追加するファイル・関数）
- `backend/mediavault-api/src/handlers/items.rs` に **list_items_handler** 追加
- `backend/mediavault-api/src/repositories/item_repository.rs` に **list_items** / **count_items** 関数追加
- `backend/mediavault-api/src/models/item.rs` に **ListItemsQuery** 構造体追加（クエリパラメータDTO）
- `backend/mediavault-api/src/routes/mod.rs` に `.route("/items", get(...))` 追加

## 4. 設計文書

### api-endpoints.md: GET /items 仕様
- **信頼性レベル**: 🔵 (REQ-001・user-stories 1.4より)
- **説明**: コレクション一覧取得（絞り込み対応）
- **クエリパラメータ**: 
  - `media_type` (Optional<MediaType>)
  - `tag_id` (Optional<Uuid>)
  - `category_id` (Optional<Uuid>)
  - `is_favorite` (Optional<bool>)
  - `status` (Optional<ItemStatus>)
  - `page` (Optional<u32>, デフォルト 1)
  - `limit` (Optional<u32>, デフォルト 20, 最大 100)
  - すべてオプション
- **各フィルタ条件**: AND 結合で適用
- **ページネーション**: 
  - limit は 1〜100 の範囲にクランプ
  - OFFSET = (page - 1) * limit
  - total は同条件での COUNT(*) クエリで取得
- **レスポンス（成功）**: items配列 + pagination
  - 形式: `{ "success": true, "data": [...], "pagination": { "page": 1, "limit": 20, "total": 100 } }`
- **参照元**: `docs/design/mediavault-backend/api-endpoints.md` (GET /items セクション, L63-73)

### database-schema.sql: items テーブル + インデックス定義
- **items テーブル** (L45-68):
  - id UUID PRIMARY KEY (gen_random_uuid())
  - media_type media_type NOT NULL
  - title VARCHAR(500) NOT NULL (CHECK: 空でない)
  - original_title, description, cover_image_url, release_date, homepage_url, rating, is_favorite
  - status item_status NOT NULL DEFAULT 'not_started'
  - consumed_date, source, external_id, created_at, updated_at
- **インデックス定義** (L70-73):
  - `idx_items_media_type ON items(media_type)` - media_type 絞り込み用 🔵
  - `idx_items_status ON items(status)` - status 絞り込み用 🔵
  - `idx_items_is_favorite ON items(is_favorite)` - is_favorite 絞り込み用 🔵
- **タグ・カテゴリテーブル** (別タスクで実装):
  - tag_id / category_id 指定時は item_tags / item_categories を JOIN または サブクエリで絞り込む
- **参照元**: `docs/design/mediavault-backend/database-schema.sql` (L45-160)

### architecture.md: レイヤードアーキテクチャ設計
- **パターン**: Layered Architecture (routes → handlers → repositories → db/DB)
- **選択理由**: sqlx コンパイル時チェック活かし、DBアクセス集約、外部API は api-client-lib に依存
- **DB アクセス方法**: sqlx (async, PgPool, QueryBuilder 活用)
- **インデックス活用**: 一覧・絞り込み API は `idx_items_media_type`, `idx_items_status`, `idx_items_is_favorite` で高速化
- 参照元: `docs/design/mediavault-backend/architecture.md`

### TASK-0010 タスクファイル本体
- **完了条件**:
  - [ ] GET /items が media_type, tag_id, category_id, is_favorite, status, page, limit のクエリパラメータをすべてoptionalで受け付ける
  - [ ] 各フィルタ条件は AND 結合で適用
  - [ ] page デフォルト1, limit デフォルト20・最大100
  - [ ] レスポンスが `{ "success": true, "data": [...], "pagination": {...} }` 形式
  - [ ] idx_items_media_type, idx_items_status, idx_items_is_favorite インデックスを活用するクエリ
- **単体テスト要件**:
  - TC-001: 絞り込みなしの一覧取得
  - TC-002: media_type による絞り込み
  - TC-003: 複数条件の AND 絞り込み
  - TC-004: limit の最大値クランプ
- **統合テスト要件**: tag_id/category_id 絞り込みは多対多テーブル（item_tags/item_categories）とのJOINを伴うため、実DB での統合テスト実施
- **参照元**: `docs/tasks/mediavault-backend/TASK-0010.md`

## 5. テスト関連情報

### テストフレームワーク・構成
- **フレームワーク**: Rust 標準 `#[tokio::test]` / `#[test]`
- **パターン**: `#[cfg(test)] mod tests { ... }` をソースファイル内に同居（jestやplaywrightのような外部設定ファイルなし）
- **テストコンテナ**: testcontainers 等は現状未導入（DB結合テストは docker-compose 経由のテスト用DBまたは手動セットアップが必要と見込まれる）

### 既存テスト例
- `backend/mediavault-api/src/models/item.rs` 内:
  - `tests::parse_create_item_request_*` (正常系・異常系)
  - `tests::validate_title_*` (バリデーション)
- `backend/mediavault-api/src/models/response.rs` 内:
  - `tests::api_error_*`, `tests::api_ok_*` (ステータスコード確認)
- `backend/mediavault-api/src/handlers/items.rs` 内:
  - `tests::created_response_returns_201_with_success_envelope` (レスポンスフォーマット確認)
- `backend/mediavault-api/src/repositories/item_repository.rs` 内:
  - `tests::detail_table_name_for_*` (8パターン、テーブル名マッピング確認)
  - 統合テスト例はまだ少ないため、TASK-0010 で整備が必要

### ユニットテストの実装パターン
- クエリパラメータ抽出・バリデーション: ユニットテスト で cover
- リポジトリ層の動的 WHERE 句構築: ユニットテスト（クエリ文字列確認）+ 統合テスト（実DB確認）の両面
- ページネーション計算（offset・limit クランプ）: ユニットテスト で cover

### 統合テスト手法
- **方針**: docker-compose 経由のテスト用DB（主流）または testcontainers 導入
- **現状**: sqlx-cli マイグレーション実行済みのため、テスト時も同じマイグレーション実行で DB スキーマ初期化
- **参照元**: `backend/mediavault-api/Cargo.toml`, `backend/CLAUDE.md`

## 6. 注意事項

### 技術的制約
- **不正なクエリパラメータ**: `page=abc` 等の不正値は Axum デシリアライズエラーとして 400 返却
- **limit 最大値**: limit > 100 の場合は 100 にクランプ（DBクエリ前に検証）
- **ページング: limit=0 の場合**: 妥当なデフォルト値への調整やエラー返却の方針を設計フェーズで確定必要（現状TASK-0010ではケース明示されていないため、要件確認）
- **tag_id/category_id 絞り込み**: item_tags / item_categories 中間テーブルとのJOIN または サブクエリが必要
  - 中間テーブル構造: item_tags (item_id UUID, tag_id UUID)、item_categories (item_id UUID, category_id UUID)
  - 多対多関係のため、GROUP BY でアイテムを集約する必要がある場合あり（設計フェーズで最適な SQL パターンを確定）

### パフォーマンス要件
- **応答時間**: 数千件規模での1秒以内応答（NFR-002）
- **インデックス活用**: idx_items_media_type, idx_items_status, idx_items_is_favorite の 3つは設計時点で定義済み
- **ページネーション**: OFFSET で大規模オフセットの場合、書き込みテーブルになるため、keyset pagination への移行検討余地あり（本フェーズでは要件外）

### セキュリティ
- **SQLインジェクション対策**: sqlx の bind() メソッドで完全に防ぐ（クエリビルダーも同様）
- **入力検証**: media_type / status 等は Rust enum で型安全（Axum デシリアライズ失敗時は 400）
- **認証**: ユーザー認証なし、APIキー認証は内部API (`/internal/*`) のみ対象

### デバッグ情報
- DB エラーは tracing::error!() でサーバーログに詳細出力（クライアント向けは汎用メッセージ）
- 参照元: `backend/mediavault-api/src/repositories/item_repository.rs` (db_error 関数)

### 既知の実装上の検討項目
- **中間テーブル JOIN パターン**: tag_id/category_id 絞り込み時の最適な SQL (JOIN vs サブクエリ) は tdd-requirements/testcases フェーズで確定必要
- **データベース結合テスト**: testcontainers 導入か docker-compose 経由か、テスト環境設定方針の確定が必要（TASK-0010 実装時に方針決定か他タスクで先行実施か）

## 7. 関連ファイルパス（相対パス）

### コード実装ファイル
- `backend/mediavault-api/src/handlers/items.rs` - ハンドラ（list_items_handler 追加）
- `backend/mediavault-api/src/repositories/item_repository.rs` - リポジトリ（list_items/count_items 関数追加）
- `backend/mediavault-api/src/models/item.rs` - モデル（ListItemsQuery 構造体追加）
- `backend/mediavault-api/src/routes/mod.rs` - ルーター（GET /items エンドポイント追加）

### 依存ファイル
- `backend/mediavault-api/Cargo.toml` - 依存パッケージ（既に必要な deps は定義済み）
- `backend/mediavault-api/src/main.rs` - アプリケーション起動（変更不要）
- `backend/mediavault-api/src/models/response.rs` - レスポンス型（参考用）

### 設計・要件文書
- `docs/design/mediavault-backend/api-endpoints.md` - API仕様
- `docs/design/mediavault-backend/architecture.md` - アーキテクチャ
- `docs/design/mediavault-backend/database-schema.sql` - DBスキーマ
- `docs/spec/mediavault-backend/requirements.md` - 要件定義
- `docs/tasks/mediavault-backend/TASK-0010.md` - 本タスク仕様

### 参考実装（前タスク）
- `docs/implements/mediavault-backend/TASK-0009/note.md` - POST /items 実装ノート（パターン参考用）
