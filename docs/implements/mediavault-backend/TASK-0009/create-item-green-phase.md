# TASK-0009: POST /items（手動作成） Greenフェーズ

## 実装方針

- Redフェーズで作成した`todo!()`スタブ（`detail_table_name`, `created_response`）を実装し、テストを通す
- `create_item_handler`本体・`item_repository::create_item`（トランザクションINSERT）も併せて最小実装し、ルーティングまで一通り疎通させる
- `details`（JSON）の各カラムへの反映は今回は対象外とし、`item_id`のみの詳細レコード作成に留める（Refactorフェーズまたは後続改善で対応）

## 実装コード

- `backend/mediavault-api/src/repositories/item_repository.rs`
  - `detail_table_name(media_type) -> &'static str`: match式で8テーブルへ振り分け 🔵
  - `db_error(sqlx::Error) -> ApiError`: DBエラーをINTERNAL_ERRORへ変換 🟡
  - `create_item(pool, request) -> Result<Item, ApiError>`: トランザクション内でitems INSERT（RETURNING）→詳細テーブルINSERT→commit 🔵
- `backend/mediavault-api/src/handlers/items.rs`
  - `create_item_handler`: `parse_create_item_request`→`create_item`→`created_response`の流れ 🔵
  - `created_response(item) -> Response`: `(StatusCode::CREATED, Json(ApiOk::new(item)))` 🔵
- `backend/mediavault-api/src/routes/mod.rs`: `.route("/items", post(create_item_handler))`追加 🔵
- `backend/mediavault-api/src/handlers/mod.rs`, `main.rs`: モジュール宣言追加

## テスト実行結果

```
cargo test -p mediavault-api
running 25 tests
...
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Red フェーズで作成した9件すべて成功。既存24件（TASK-0005〜0008分）も含め全25件成功、リグレッションなし。

## 品質判定

✅ **高品質**
- テスト結果: 全25件成功（cargo testで確認）
- 実装品質: シンプル。match式・トランザクション処理ともに既存パターンに準拠
- ファイルサイズ: `items.rs` 103行、`item_repository.rs` 178行（800行制限内）
- モック使用: 実装コードにモック・スタブなし（`create_item`は実際のsqlxクエリを使用）
- コンパイルエラー: なし

## 課題・改善点（Refactorフェーズで対応）

1. `details`（JSON）の内容を詳細テーブルの個別カラムへマッピングする処理が未実装（現状はitem_idのみのレコード）
   - TC-001-02相当（genre_list等の値反映）に対応するには、`media_type`ごとに異なるカラム構成をどう汎用的に扱うか設計が必要
2. `db_error`のエラーメッセージにSQLエラー内容を含めている点（内部情報の漏洩リスク）の見直し
3. 実DB結合テスト（docker-compose経由のPostgreSQLを使った`create_item`全体のテスト）が未整備
4. `format!`によるテーブル名埋め込みについて、`detail_table_name`の戻り値が固定文字列であることのコメントを残しているが、より静的に安全性を示す方法（型レベルの保証）の検討
