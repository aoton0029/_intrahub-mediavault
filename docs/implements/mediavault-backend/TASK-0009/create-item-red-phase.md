# TASK-0009: POST /items（手動作成） Redフェーズ

## 作成したテストケース一覧

DBを必要としないユニットテストとして以下9件を新規作成した（実DB結合テストはGreen/Refactor完了後にdocker-compose経由のPostgreSQLに対し別途追加する）。

1. `repositories::item_repository::tests::detail_table_name_for_anime` 🔵
2. `repositories::item_repository::tests::detail_table_name_for_movie` 🔵
3. `repositories::item_repository::tests::detail_table_name_for_drama` 🔵
4. `repositories::item_repository::tests::detail_table_name_for_manga` 🔵
5. `repositories::item_repository::tests::detail_table_name_for_novel` 🔵
6. `repositories::item_repository::tests::detail_table_name_for_game` 🔵
7. `repositories::item_repository::tests::detail_table_name_for_academic_book` 🔵
8. `repositories::item_repository::tests::detail_table_name_for_paper` 🔵
9. `handlers::items::tests::created_response_returns_201_with_success_envelope` 🔵（TC-001-01対応）

## テストコード

- `backend/mediavault-api/src/repositories/item_repository.rs`
- `backend/mediavault-api/src/handlers/items.rs`

それぞれ `#[cfg(test)] mod tests` 内に実装済み（上記ファイル参照）。

## 期待される失敗内容

- `detail_table_name(media_type: MediaType) -> &'static str` は `todo!()` のため、呼び出すと `not yet implemented: TASK-0009 Greenフェーズで実装する` でpanicしテスト失敗
- `created_response(item: Item) -> axum::response::Response` も同様に `todo!()` でpanic

実行結果（`cargo test -p mediavault-api -- detail_table_name created_response`）:
```
test result: FAILED. 0 passed; 9 failed; 0 ignored; 0 measured; 16 filtered out
```
全9件が意図通り失敗することを確認済み。

## Greenフェーズで実装すべき内容

1. `item_repository::detail_table_name(media_type: MediaType) -> &'static str`
   - `MediaType`の8バリアントをmatch式で対応する詳細テーブル名文字列にマッピングする
2. `item_repository::create_item(pool: &PgPool, request: CreateItemRequest) -> Result<Item, ApiError>`（新規追加）
   - `sqlx::Transaction`を開始
   - `items`テーブルへ`source=Manual`, `external_id=None`でINSERTし`Item`を取得（RETURNING句）
   - `detail_table_name`で解決したテーブルへ、`request.details`（JSON）を対応カラムにマッピングしてINSERT（`details`がNone/空オブジェクトの場合は全カラムデフォルトでINSERT）
   - コミット。いずれかのSQL実行が失敗した場合は`ApiError::new(ApiErrorCode::InternalError, ...)`を返しロールバック
3. `handlers::items::create_item_handler`
   - `Json<serde_json::Value>`を受け取り`parse_create_item_request`（TASK-0008実装済み）でバリデーション
   - 成功時は`item_repository::create_item`を呼び出し、結果を`created_response`でラップ
4. `handlers::items::created_response(item: Item) -> axum::response::Response`
   - `(StatusCode::CREATED, Json(ApiOk::new(item))).into_response()`相当を実装
5. `routes/mod.rs`の`build_router`に`.route("/items", post(create_item_handler))`を追加

## 信頼性レベルサマリー

| カテゴリ | 🔵 | 🟡 | 🔴 | 合計 |
|---|---|---|---|---|
| 新規テスト | 9 | 0 | 0 | 9 |

すべて database-schema.sql のテーブル定義・タスクファイルの完了条件に直接対応するため🔵。
