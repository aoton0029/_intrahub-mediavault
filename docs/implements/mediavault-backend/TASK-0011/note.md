# TASK-0011 開発コンテキストノート

## 1. 技術スタック
- Rust + Axum (REST API)
- sqlx (PostgreSQL, QueryBuilder, FromRow)
- tokio (非同期ランタイム)
- 参照元: backend/mediavault-api/src/main.rs, backend/mediavault-api/Cargo.toml

## 2. 開発ルール
- ハンドラは `Result<T, ApiError>` を返す
- DBエラーは `db_error()` で `ApiErrorCode::InternalError` に変換し詳細を漏らさない
- レスポンスは共通エンベロープ（`ApiOk<T>` / `PaginatedOk<T>`）を使う
- 参照元: backend/mediavault-api/src/handlers/items.rs, backend/mediavault-api/src/repositories/item_repository.rs

## 3. 関連実装（参考パターン）
- `list_items_handler` / `create_item_handler`: backend/mediavault-api/src/handlers/items.rs
- `build_list_items_query` / `list_items` / `count_items`: backend/mediavault-api/src/repositories/item_repository.rs
- `detail_table_name(media_type: MediaType) -> &'static str`: backend/mediavault-api/src/repositories/item_repository.rs（8つのmedia_typeを詳細テーブル名にマッピング）
- ルーター定義: backend/mediavault-api/src/routes/mod.rs（`/items` に GET/POSTをぶら下げる形式。`/items/:id` を新規追加する）

## 4. 設計文書
- API仕様: docs/design/mediavault-backend/api-endpoints.md（GET /items/:id, ITEM_NOT_FOUND）
- DBスキーマ: docs/design/mediavault-backend/database-schema.sql
  - `items` テーブル: id(UUID PK), media_type, title, original_title, description, cover_image_url, release_date, homepage_url, status, consumed_date, rating, is_favorite, source, external_id, created_at, updated_at
  - メディア別詳細テーブル（item_id 1:1, ON DELETE CASCADE）: anime_details, movie_details, drama_details, manga_details, novel_details, game_details, academic_book_details, paper_details
  - タグ/カテゴリ中間テーブル: item_tags(item_id, tag_id), item_categories(item_id, category_id)
  - タグ/カテゴリ本体: tags, categories
- タスク定義: docs/tasks/mediavault-backend/TASK-0011.md
- 関連モデル: backend/mediavault-api/src/models/item.rs（Item, MediaType, ItemStatus, ItemSource）
- 共通エラー/レスポンス型: backend/mediavault-api/src/models/response.rs（ApiError, ApiErrorCode::ItemNotFound, ApiOk）

## 5. テスト関連情報
- テストは各 `.rs` ファイル内に `#[cfg(test)]` モジュールとして実装
- 統合テスト（実DB使用）は `#[tokio::test]` + `#[ignore]` で記述し、`DATABASE_URL` 環境変数を要求
- テストDBプール: `sqlx::PgPool::connect("postgres://...")`
- シードヘルパー例: `insert_test_item()`, `seed_items()`, `seed_items_by_media_type()`（item_repository.rs内のテストモジュール参照）
- SQL生成のみを検証する単体テスト（QueryBuilder文字列検証）も存在（list_items関連）
- 参照元: backend/mediavault-api/src/handlers/items.rs, backend/mediavault-api/src/repositories/item_repository.rs

## 6. 注意事項
- `media_type` ごとに異なる詳細テーブルへJOINする必要があるため、match式で動的にクエリを切り替える（detail_table_nameの命名パターンを再利用）
- 存在しないIDは `ApiErrorCode::ItemNotFound`（404）
- 不正なUUID形式は400（Axumのパスパラメータ抽出失敗時の挙動を確認し、必要なら独自パース＋エラー変換を行う）
- タグ・カテゴリは取得して含めるが、item_relations等の他の関連付けは本タスクの対象外（TASK-0017以降で拡張）
- 実装対象ファイル: backend/mediavault-api/src/handlers/items.rs, backend/mediavault-api/src/repositories/item_repository.rs, backend/mediavault-api/src/routes/mod.rs（必要なら）, backend/mediavault-api/src/models/item.rs（詳細レスポンスDTO追加が必要な場合）
