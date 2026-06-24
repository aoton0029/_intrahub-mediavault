# TASK-0015 Redフェーズ記録: タグ・カテゴリCRUD実装

## 実装したテストケース一覧

| テストケース | ファイル | テスト関数 | 信頼性 |
|---|---|---|---|
| TC-1 (model) | backend/mediavault-api/src/models/tag.rs | `create_tag_request_deserializes_valid_name` | 🔵 |
| TC-11 | backend/mediavault-api/src/models/tag.rs | `create_tag_with_empty_name_returns_validation_error` | 🟡 |
| TC-15 | backend/mediavault-api/src/models/tag.rs | `create_tag_with_max_length_name_succeeds` | 🟡 |
| TC-16 | backend/mediavault-api/src/models/tag.rs | `create_tag_request_missing_name_field_fails_deserialization` | 🔵 |
| TC-2 (model) | backend/mediavault-api/src/models/category.rs | `create_category_request_deserializes_valid_name` | 🔵 |
| TC-12 | backend/mediavault-api/src/models/category.rs | `create_category_with_empty_name_returns_validation_error` | 🟡 |
| TC-16相当(category) | backend/mediavault-api/src/models/category.rs | `create_category_request_missing_name_field_fails_deserialization` | 🔵 |
| TC-1 (repo) | backend/mediavault-api/src/repositories/tag_repository.rs | `create_tag_inserts_and_returns_tag` | 🔵 |
| TC-7 | backend/mediavault-api/src/repositories/tag_repository.rs | `create_tag_with_duplicate_name_returns_conflict_error` | 🟡 |
| TC-9 | backend/mediavault-api/src/repositories/tag_repository.rs | `delete_nonexistent_tag_returns_not_found` | 🟡 |
| TC-3 | backend/mediavault-api/src/repositories/tag_repository.rs | `attach_tag_to_item_inserts_item_tags_row` | 🟡 |
| TC-4 | backend/mediavault-api/src/repositories/tag_repository.rs | `detach_tag_from_item_deletes_item_tags_row` | 🟡 |
| TC-6 | backend/mediavault-api/src/repositories/tag_repository.rs | `delete_tag_cascades_item_tags` | 🟡 |
| TC-13 | backend/mediavault-api/src/repositories/tag_repository.rs | `attach_tag_to_nonexistent_item_returns_not_found` | 🟡 |
| TC-14 | backend/mediavault-api/src/repositories/tag_repository.rs | `attach_already_attached_tag_returns_conflict_or_noop` | 🟡 |
| (DBエラー変換) | backend/mediavault-api/src/repositories/tag_repository.rs | `create_tag_db_error_maps_to_internal_error` | 🟡 |
| TC-2 (repo) | backend/mediavault-api/src/repositories/category_repository.rs | `create_category_inserts_and_returns_category` | 🔵 |
| TC-8 | backend/mediavault-api/src/repositories/category_repository.rs | `create_category_with_duplicate_name_returns_conflict_error` | 🟡 |
| TC-10 | backend/mediavault-api/src/repositories/category_repository.rs | `delete_nonexistent_category_returns_not_found` | 🟡 |
| TC-5 | backend/mediavault-api/src/repositories/category_repository.rs | `attach_and_detach_category_to_item` | 🟡 |

合計20テスト（テストケース定義書の16項目＋DBエラー変換等の補完テスト）。

## モジュール登録

- `backend/mediavault-api/src/models/mod.rs`: `pub mod category;` / `pub mod tag;` を追加
- `backend/mediavault-api/src/repositories/mod.rs`: `pub mod category_repository;` / `pub mod tag_repository;` を追加
- handlers/routesの登録はGreenフェーズで実装本体と合わせて行う（現時点ではモデル・リポジトリ層のみ）

## 実行コマンドと結果

```bash
cd backend
cargo test -p mediavault-api tag_repository
```

**結果**: コンパイルエラー（期待通りのRed状態）

```
error[E0425]: cannot find function `create_tag` in this scope
error[E0425]: cannot find function `delete_tag` in this scope
error[E0425]: cannot find function `attach_tag_to_item` in this scope
error[E0425]: cannot find function `detach_tag_from_item` in this scope
...
error: could not compile `mediavault-api` (bin "mediavault-api" test) due to 29 previous errors
```

`models/tag.rs`, `models/category.rs` 側も同様に `CreateTagRequest`, `validate_tag_name`, `CreateCategoryRequest`, `validate_category_name` が未定義のためコンパイルエラーとなる（同一クレートのため上記コマンド一発で全体のコンパイルエラーとして検出される）。

## Greenフェーズで実装すべき内容

### models/tag.rs
- `pub struct Tag { pub id: Uuid, pub name: String }`（`#[derive(sqlx::FromRow, Serialize, ...)]`）
- `pub struct CreateTagRequest { pub name: String }`（`#[derive(Deserialize)]`）
- `pub fn validate_tag_name(name: &str) -> Result<(), ApiError>`（trim().is_empty()チェック。最大長チェックは要否を実装時に判断、DBのVARCHAR(100)制約に委ねる場合はバリデーション不要）

### models/category.rs
- tag.rsと完全に対称: `Category`, `CreateCategoryRequest`, `validate_category_name`

### repositories/tag_repository.rs
- `db_error(err: sqlx::Error) -> ApiError`（item_repository.rsと同パターン、ログ＋汎用メッセージ）
- `fn unique_violation_error(err: &sqlx::Error) -> Option<ApiError>`相当: SQLSTATE `23505`を検出し`DUPLICATE_TAG_NAME`（新規ApiErrorCode、409）へ変換
- `pub async fn create_tag(pool: &PgPool, name: String) -> Result<Tag, ApiError>`
- `pub async fn delete_tag(pool: &PgPool, id: Uuid) -> Result<bool, ApiError>`
- `pub async fn attach_tag_to_item(pool: &PgPool, item_id: Uuid, tag_id: Uuid) -> Result<(), ApiError>`（FK違反23503→404相当、複合PK違反23505→409相当）
- `pub async fn detach_tag_from_item(pool: &PgPool, item_id: Uuid, tag_id: Uuid) -> Result<bool, ApiError>`
- `test_pool()` / `unreachable_pool()` ヘルパー実装（item_repository.rsと同一内容）

### repositories/category_repository.rs
- tag_repository.rsと完全に対称: `create_category`, `delete_category`, `attach_category_to_item`, `detach_category_from_item`

### response.rs（新規ApiErrorCode追加）
- `DuplicateTagName` → `("DUPLICATE_TAG_NAME", StatusCode::CONFLICT)`
- `DuplicateCategoryName` → `("DUPLICATE_CATEGORY_NAME", StatusCode::CONFLICT)`

### handlers/tags.rs, handlers/categories.rs, routes/mod.rs
- ハンドラ実装とルート登録（`POST /tags`, `DELETE /tags/:id`, `POST/DELETE /items/:id/tags/:tag_id`, カテゴリ版も同様）はGreenフェーズで追加する。HTTPレベルの統合テストはverify-completeまでに必要に応じて追加する。
