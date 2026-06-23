# TASK-0011 詳細要件定義: GET /items/:id 詳細取得

## 機能概要
指定したUUIDの`items`レコードを、メディア別詳細テーブル・タグ・カテゴリを含めて取得するAPIエンドポイント。

## エンドポイント
`GET /items/:id`

## 入力
- パスパラメータ `id`: UUID文字列

## 出力（成功時, 200）
共通レスポンスエンベロープ `ApiOk<ItemDetail>` 形式。

```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "media_type": "anime",
    "title": "...",
    "original_title": null,
    "description": null,
    "cover_image_url": null,
    "release_date": null,
    "homepage_url": null,
    "status": "not_started",
    "consumed_date": null,
    "rating": null,
    "is_favorite": false,
    "source": "manual",
    "external_id": null,
    "created_at": "...",
    "updated_at": "...",
    "detail": { /* media_typeに応じた詳細テーブルのカラム群、JSON object。レコードが無ければnull */ },
    "tags": [{ "id": "uuid", "name": "..." }],
    "categories": [{ "id": "uuid", "name": "..." }]
  }
}
```

## エラー
- 該当item無し → `ApiErrorCode::ItemNotFound`（404, code="ITEM_NOT_FOUND"）
- UUID形式不正 → 400（`ApiErrorCode::ValidationError`）
- DBエラー → 500（`ApiErrorCode::InternalError`、詳細非開示）

## 実装方針
1. **ルーティング**: `backend/mediavault-api/src/routes/mod.rs` に `/items/:id` (GET) を追加。既存の `/items` ルートとは別パスとして定義（Axumの `Path<Uuid>` 抽出子を使う）。
2. **UUIDバリデーション**: Axumの`Path<Uuid>`抽出に失敗した場合、デフォルトでは400 Bad Requestが返るが、レスポンス形式を共通エラーエンベロープに合わせる必要があるか確認する。型を`Path<String>`として受け取り手動で`Uuid::parse_str`し、失敗時に`ApiErrorCode::ValidationError`(400)を返す方式を採用する（既存パターンに合わせて明示的なエラーハンドリングを行う）。
3. **リポジトリ関数追加** (`item_repository.rs`):
   - `get_item_by_id(pool, id: Uuid) -> Result<Option<Item>, sqlx::Error>`: itemsテーブルから1件取得。
   - `get_item_detail(pool, media_type: MediaType, item_id: Uuid) -> Result<Option<serde_json::Value>, sqlx::Error>`: `detail_table_name()`を再利用し、`SELECT * FROM {table} WHERE item_id = $1`を実行、結果をJSON Valueとして返す（テーブル毎にカラムが異なるためsqlx::types::Json的な動的取得が必要 → `sqlx::query` + 行から`serde_json::Map`構築、もしくは各detailテーブルへ`sqlx::FromRow`構造体を用意してserde_json::to_valueする）。
   - `get_item_tags(pool, item_id: Uuid) -> Result<Vec<TagRef>, sqlx::Error>`: `item_tags` JOIN `tags`。
   - `get_item_categories(pool, item_id: Uuid) -> Result<Vec<CategoryRef>, sqlx::Error>`: `item_categories` JOIN `categories`。
4. **モデル追加** (`models/item.rs` または新規):
   - `ItemDetail`構造体（Itemの全フィールド + `detail: Option<serde_json::Value>` + `tags: Vec<TagRef>` + `categories: Vec<CategoryRef>`）
   - `TagRef { id: Uuid, name: String }`, `CategoryRef { id: Uuid, name: String }`
5. **ハンドラ追加** (`handlers/items.rs`): `get_item_handler(State(state), Path(id_str): Path<String>) -> Result<impl IntoResponse, ApiError>`
   - UUID parse → 400
   - `get_item_by_id` → None なら404 ItemNotFound
   - `get_item_detail` / `get_item_tags` / `get_item_categories` を呼び出し合成
   - `ApiOk(ItemDetail)`を200で返す
6. **既存パターンの再利用**: `db_error()`によるDBエラー変換、`ApiOk`エンベロープ。

## 信頼性
🟡 個別取得APIの詳細構造（detailのJSON化方式等）はPRD未記載のため設計判断を含む。
