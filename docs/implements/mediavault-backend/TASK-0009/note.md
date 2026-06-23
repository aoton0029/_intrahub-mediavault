# TASK-0009 開発ノート: POST /items（手動作成）実装

## 1. 技術スタック
- Rust + Axum (0.8.9) + sqlx (0.8, postgres, runtime-tokio, macros, chrono, uuid) + tokio (full)
- レイヤー構成: `routes` → `handlers` → (今回新設) `repositories` → DB
- 参照元: `backend/mediavault-api/Cargo.toml`, `backend/mediavault-api/src/main.rs`

## 2. 開発ルール
- AGENTS.md は repo root に見当たらず、`backend/CLAUDE.md` にビルド/テストコマンドの記載あり。
  - ビルド: `cargo build -p mediavault-api`
  - テスト: `cargo test --workspace`
- `./docs/rule` ディレクトリは存在しない。
- 参照元: `backend/CLAUDE.md`

## 3. 関連実装（TASK-0008 / TASK-0007 / TASK-0005 / TASK-0006の成果物）

### models/item.rs（既存・変更不要）
- `MediaType` enum: `Anime, Movie, Drama, Manga, Novel, Game, AcademicBook, Paper`（snake_case で serde/sqlx 変換、sqlx型名 `media_type`）
- `ItemStatus` enum: `NotStarted, InProgress, Completed`（sqlx型名 `item_status`、デフォルトは `not_started`）
- `ItemSource` enum: `Api, Manual`（sqlx型名 `item_source`）
- `Item` 構造体（`sqlx::FromRow` 実装済み）: id, media_type, title, original_title, description, cover_image_url, release_date, homepage_url, status, consumed_date, rating, is_favorite, source, external_id, created_at, updated_at
- `CreateItemRequest` 構造体: media_type, title, original_title, description, cover_image_url, release_date, homepage_url, rating, is_favorite, details(`Option<serde_json::Value>`)
  - **注意**: `source`/`external_id` はリクエストに含まれない（ハンドラ側で `Manual`/`None` を付与する設計、コメントに明記）
- `parse_create_item_request(value: serde_json::Value) -> Result<CreateItemRequest, ApiError>`: デシリアライズ＋`validate_title`を実行
- `validate_title(title: &str) -> Result<(), ApiError>`: 空白のみならVALIDATION_ERROR
- 参照元: `backend/mediavault-api/src/models/item.rs`

### models/response.rs（既存・変更不要）
- `ApiOk<T> { success: bool, data: T }`、`IntoResponse`実装で `StatusCode::OK` 固定（**201を返したい場合は別途 `(StatusCode::CREATED, Json(ApiOk::new(...)))` を組み立てる必要がある**）
- `ApiError { success: false, error: ApiErrorBody{code, message}, status }`
- `ApiErrorCode`: `ValidationError`(400/VALIDATION_ERROR), `Unauthorized`(401), `ItemNotFound`(404), `UnprocessableEntity`(422), `InternalError`(500), `ExternalApiError`(502)
- 参照元: `backend/mediavault-api/src/models/response.rs`

### main.rs / routes/mod.rs / db/mod.rs（既存・変更が必要）
- `AppState { db: PgPool, internal_api_key: String }`
- `routes::build_router(state: AppState) -> Router`: 現状 `/health` のみ。`.route("/items", post(...))` をここに追加する想定
- `db::create_pool(database_url: &str) -> Result<PgPool, sqlx::Error>`
- handlerパターン例 (`handlers/health.rs`): `async fn health_handler(State(state): State<AppState>) -> impl IntoResponse` で `sqlx::query(...).execute(&state.db)`
- 参照元: `backend/mediavault-api/src/main.rs`, `backend/mediavault-api/src/routes/mod.rs`, `backend/mediavault-api/src/db/mod.rs`, `backend/mediavault-api/src/handlers/health.rs`, `backend/mediavault-api/src/handlers/mod.rs`（現状 `pub mod health;` のみ → `pub mod items;` を追加要）

### 未実装（TASK-0009で新規作成するファイル）
- `backend/mediavault-api/src/handlers/items.rs`（新規）: `create_item_handler`
- `backend/mediavault-api/src/repositories/`ディレクトリ（新規）+ `item_repository.rs`: トランザクションでitems＋詳細テーブルへINSERT
- `backend/mediavault-api/src/repositories/mod.rs`（新規）
- `main.rs` に `mod repositories;` 追加要

## 4. 設計文書

### api-endpoints.md: POST /items 仕様
- リクエスト例: `{ "media_type": "anime", "title": "作品A", "details": {} }`
- レスポンス（成功, 201）: 作成済みitem（UUID付き）
- エラー: `VALIDATION_ERROR`（400, media_type不正等）
- 参照元: `docs/design/mediavault-backend/api-endpoints.md` (POST /items セクション, L85-101)

### database-schema.sql: items + メディア別詳細テーブル
- `items`テーブル: id(UUID PK, gen_random_uuid()), media_type, title(NOT NULL), original_title, description, cover_image_url, release_date, homepage_url, status(DEFAULT 'not_started'), consumed_date, rating, is_favorite(DEFAULT FALSE), source(NOT NULL), external_id, created_at, updated_at
- 詳細テーブル（全て `item_id UUID PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE`、1:1関連）:
  - `anime_details`: episode_count, season_count, studio, genre_list(TEXT[] DEFAULT '{}'), source_type, jikan_id
  - `movie_details`: runtime_minutes, director, genre_list(TEXT[] DEFAULT '{}'), tmdb_id
  - `drama_details`: episode_count, season_count, network, genre_list(TEXT[] DEFAULT '{}'), tmdb_id
  - `manga_details`: volume_count, chapter_count, author, illustrator, magazine, jikan_id
  - `novel_details`: volume_count, author, publisher, isbn, openlibrary_id, google_books_id
  - `game_details`: platform_list(TEXT[] DEFAULT '{}'), developer, publisher, steam_appid, igdb_id
  - `academic_book_details`: author, publisher, isbn, ndl_id, google_books_id
  - `paper_details`: doi, journal_name, volume_issue, page_range, author_list(TEXT[] DEFAULT '{}'), ndl_id
- media_type → 詳細テーブル名のマッピングはmatch式で振り分け（タスクファイルの注意事項にも明記）
- 参照元: `docs/design/mediavault-backend/database-schema.sql` (L45-160)

### TASK-0009タスクファイル本体
- 完了条件・テストケース（TC-001-01, TC-001-E01, TC-001-B01）・実装手順を含む
- 参照元: `docs/tasks/mediavault-backend/TASK-0009.md`

## 5. テスト関連情報
- テストフレームワーク: Rust標準 `#[tokio::test]` / `#[test]`、`#[cfg(test)] mod tests` をソースファイル内に同居させるパターン（jestやplaywrightのような外部設定ファイルなし）
- 既存テスト例: `models/item.rs` 内の `tests` モジュール（`parse_create_item_request`の正常系・異常系）、`models/response.rs` 内の `tests` モジュール（`ApiError`/`ApiOk`のステータスコード確認）
- DB結合テスト（実PostgreSQL使用）の仕組みは現状未整備（testcontainers等は導入されていない）。TASK-0009の統合テスト要件（実DB確認）は、docker-compose経由のテスト用DBに対し `#[sqlx::test]` または手動セットアップでの結合テストとして実装する必要がある可能性がある（要 tdd-requirements/testcases フェーズで方針確定）
- 参照元: `backend/mediavault-api/src/models/item.rs`, `backend/mediavault-api/src/models/response.rs`, `backend/mediavault-api/Cargo.toml`

## 6. 注意事項
- `ApiOk::into_response()` は `StatusCode::OK`(200)固定のため、201を返すには `(StatusCode::CREATED, Json(ApiOk::new(item)))` のような形でハンドラ内で組み立てる必要がある（既存の汎用型をそのまま使えない点に注意）
- `source`は常に`ItemSource::Manual`、`external_id`は常に`None`で items へINSERT（ハンドラ側で付与、CreateItemRequestには含まれない）
- `media_type`ごとに異なる詳細テーブルへ振り分けるロジックはmatch式で実装（`details`未指定時は全カラムNULL/デフォルト値 `'{}'` でINSERT）
- items INSERTと詳細テーブルINSERTは同一トランザクション内で実行し、失敗時はロールバック（`sqlx::Transaction`使用）
- 参照元: `docs/tasks/mediavault-backend/TASK-0009.md`（注意事項セクション）
