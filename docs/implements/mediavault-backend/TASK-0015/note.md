# TASK-0015 開発ノート: タグ・カテゴリCRUD実装

## 1. 技術スタック
- Rust edition 2024 / axum 0.8.9 / sqlx 0.8 (postgres, runtime-tokio, macros, chrono, uuid)
- serde 1.0.228 (derive) / uuid 1 (v4, serde)
- エラーハンドリングは `ApiError`/`ApiErrorCode`、成功レスポンスは `ApiOk<T>` で統一
- 参照元: backend/mediavault-api/Cargo.toml, backend/mediavault-api/src/models/response.rs

## 2. 開発ルール
- ユニットテストは実装ファイル末尾に `#[cfg(test)] mod tests` でインライン配置（別ファイルなし）
- DB非依存の純粋関数テストは `#[test]`、実DB必要な統合テストは `#[tokio::test] #[ignore]` + `DATABASE_URL`（`test_pool()`ヘルパー）
- DBエラーは必ず `db_error()` 相当の関数で`tracing::error!`ログ後、クライアントには内部情報を漏らさない汎用メッセージのみ返す
- 信頼性レベル絵文字（🔵🟡🔴）と日本語コメント（【テスト目的】【テスト内容】【期待される動作】【確認内容】）を各テスト・実装関数に付与
- 参照元: docs/spec/mediavault-backend/note.md, backend/mediavault-api/src/repositories/item_repository.rs

## 3. 関連実装（既存パターン）

### モデル/DTO（item.rs を参考）
- リクエストDTO: `#[derive(Debug, Clone, Deserialize)]`
- バリデーション関数パターン: `validate_title()`（非空チェック、`trim().is_empty()`でVALIDATION_ERROR）
- `deserialize_request::<T>()`: serdeエラー→VALIDATION_ERROR「リクエストの形式が不正です: {err}」
- 参照元: backend/mediavault-api/src/models/item.rs

### ハンドラ（items.rs を参考）
- create系: `State(state): State<AppState>, Json(body)` → モデルでvalidate → repository呼び出し → `created_response(item)`（201）
- delete系: `parse_item_id()`相当でUUIDパース → repository削除 → `rows_affected==0`なら404 → 成功時 `StatusCode::NO_CONTENT.into_response()`
- 戻り値型は `Result<Response, ApiError>`（複数ステータスコードに対応するため）
- 参照元: backend/mediavault-api/src/handlers/items.rs

### リポジトリ（item_repository.rs を参考）
- `db_error(err: sqlx::Error) -> ApiError`: ログ後にInternalErrorへ変換する共通パターンを新規ファイルでも踏襲
- INSERT+RETURNING: `sqlx::query_as(...).bind(...).fetch_one(pool).await.map_err(db_error)?`
- DELETE: `sqlx::query("DELETE FROM ...").execute(pool).await.map_err(db_error)?; Ok(result.rows_affected() > 0)`
- UNIQUE制約違反（PostgreSQLエラーコード`23505`）は`db_error`を通す前に個別ハンドリングが必要（下記参照）
- 参照元: backend/mediavault-api/src/repositories/item_repository.rs

### ルーティング（routes/mod.rs を参考）
- `Router::new().route(path, method(handler)...).with_state(state)` 形式
- 既存: `/items`, `/items/:id`, `/items/:id/status`
- 本タスクで追加: `/tags`, `/tags/:id`, `/items/:id/tags/:tag_id`, `/categories`, `/categories/:id`, `/items/:id/categories/:category_id`
- 参照元: backend/mediavault-api/src/routes/mod.rs

## 4. 設計文書

### DBスキーマ（database-schema.sql）
```sql
CREATE TABLE tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE
);

CREATE TABLE item_tags (
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    tag_id UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (item_id, tag_id)
);

CREATE TABLE categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE
);

CREATE TABLE item_categories (
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    PRIMARY KEY (item_id, category_id)
);
```
- `tags.name` / `categories.name` は `UNIQUE` → 重複INSERT時はPostgreSQLエラーコード `23505`
- `item_tags` / `item_categories` は複合PKのため、重複付与も `23505`（PK制約違反）
- 両FKとも `ON DELETE CASCADE` のため、タグ/カテゴリ削除時に紐付けレコードは自動削除（アプリ側で個別削除不要）
- 参照元: docs/design/mediavault-backend/database-schema.sql

### API仕様（api-endpoints.md / TASK-0015.md）
- `POST /tags` リクエスト: `{ "name": "お気に入り" }` → 201, `{ id, name }`
- `DELETE /tags/:id` → 204
- `POST /items/:id/tags/:tag_id` → item_tagsへ複合キーINSERT（既存の場合はno-opまたは409）
- `DELETE /items/:id/tags/:tag_id` → 複合キーでDELETE
- `POST /categories`, `DELETE /categories/:id` はタグと同様パターン
- `item_categories`への付与・削除エンドポイントも同様（`POST/DELETE /items/:id/categories/:category_id`）
- 参照元: docs/tasks/mediavault-backend/TASK-0015.md, docs/design/mediavault-backend/api-endpoints.md

### エラーレスポンス（response.rs）
既存 `ApiErrorCode`:
```rust
pub enum ApiErrorCode {
    ValidationError,     // 400
    Unauthorized,        // 401
    ItemNotFound,        // 404
    UnprocessableEntity, // 422
    InternalError,       // 500
    ExternalApiError,    // 502
}
```
- タスク仕様の注意事項に「タグ・カテゴリ名の一意制約違反時のエラーコードは共通エラー型に新規追加が必要（例: `DUPLICATE_TAG_NAME`, `DUPLICATE_CATEGORY_NAME`）」と明記されているため、`ApiErrorCode`に新規バリアントを追加し409にマッピングする方針とする。
- タグ/カテゴリ自体が存在しない場合（DELETE対象なし等）は既存の `ItemNotFound` を流用せず、文脈上紛らわしいため新規 `TagNotFound`/`CategoryNotFound` を追加するか、既存`ItemNotFound`を流用するかはテストケース作成時に決定する（テストケース仕様では明示なし、🟡 妥当な推測領域）。
- 参照元: backend/mediavault-api/src/models/response.rs

## 5. テスト関連情報
- テストフレームワーク: Rust標準 `#[test]` / `#[tokio::test]`（追加のテストランナー設定なし）
- 既存テストパターン: 実装ファイル末尾に `mod tests`、DB接続が必要なものは `#[ignore]` + `test_pool()`
- DBエラー変換テスト: `unreachable_pool()` で接続不能なPgPoolを構築し、`db_error`がINTERNAL_ERROR/500に変換されることを確認
- SQL生成検証: `QueryBuilder.sql()` の文字列内容をassert（本タスクは固定SQLのため該当しない可能性が高い）
- 参照元: backend/mediavault-api/src/repositories/item_repository.rs（test_pool/unreachable_poolヘルパー）

## 6. 注意事項
- 既存コードに `tags.rs`, `categories.rs`（handlers/models/repositories）は一切存在しない。すべて新規作成。
- 新規作成ファイル:
  - `backend/mediavault-api/src/models/tag.rs`
  - `backend/mediavault-api/src/models/category.rs`
  - `backend/mediavault-api/src/handlers/tags.rs`
  - `backend/mediavault-api/src/handlers/categories.rs`
  - `backend/mediavault-api/src/repositories/tag_repository.rs`（item_repository.rsの命名規則に合わせる）
  - `backend/mediavault-api/src/repositories/category_repository.rs`
- `mod.rs`（models/handlers/repositories）への新規モジュール登録、`routes/mod.rs`へのルート追加が必須
- item_tags/item_categoriesの付与時、対象item/tag(category)が存在しない場合はFK制約違反（`23503`）となるため、事前存在チェック or FK違反のハンドリングが必要（タスク完了条件には明記なし、実装判断が必要）
- 参照元: docs/tasks/mediavault-backend/TASK-0015.md
