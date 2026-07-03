# MediaVault API 設計

## 基本方針
- RESTful API（Rust / Axum / sqlx / PostgreSQL）
- ベースURL:
  - 公開API: `/api/v1`
  - 内部API: `/internal`（バージョンプレフィックスなし）
- 認証:
  - 公開API（`/api/v1/*`）: **認証なし**。単一ユーザー・セルフホスト用途のためログイン機構は持たない。
  - 内部API（`/internal/*`）: `api_key_auth` ミドルウェアが全ルートに適用される。`Authorization` ヘッダに `INTERNAL_API_KEY` 環境変数の値（生の値、または `Bearer <key>` 形式）を渡す必要がある。キー未設定・不一致は `401 UNAUTHORIZED`。
- レスポンス形式: JSON

---

## 共通レスポンス形式

### 成功時（`ApiOk<T>`）
```json
{ "success": true, "data": { /* T */ } }
```
特記のない限り HTTP `200`。作成系は `201`、削除系は `204 No Content`（ボディなし）。

### ページネーション付き成功時（`PaginatedOk<T>`）
```json
{
  "success": true,
  "data": [ /* T[] */ ],
  "pagination": { "page": 1, "limit": 20, "total": 123 }
}
```

### エラー時（`ApiError`）
```json
{ "success": false, "error": { "code": "ITEM_NOT_FOUND", "message": "..." } }
```

### エラーコード一覧

| コード | HTTPステータス | 説明 |
|---|---|---|
| VALIDATION_ERROR | 400 | リクエストの値が不正（UUID形式不正、必須項目欠如など） |
| UNAUTHORIZED | 401 | 内部APIキー不一致・未設定 |
| ITEM_NOT_FOUND | 404 | 指定した item が存在しない |
| UNPROCESSABLE_ENTITY | 422 | 汎用の処理不能エラー |
| INTERNAL_ERROR | 500 | サーバ内部エラー（DB接続失敗など） |
| EXTERNAL_API_ERROR | 502 | 外部API呼び出し全般のエラー |
| DUPLICATE_TAG_NAME | 409 | タグ名が重複 |
| TAG_NOT_FOUND | 404 | 指定した tag が存在しない |
| DUPLICATE_CATEGORY_NAME | 409 | カテゴリ名が重複 |
| CATEGORY_NOT_FOUND | 404 | 指定した category が存在しない |
| MYLIST_NOT_FOUND | 404 | 指定した mylist が存在しない |
| DUPLICATE_RELATION | 409 | 同一の item 関連がすでに存在 |
| GROUP_NOT_FOUND | 404 | 指定した item group が存在しない |
| INVALID_GROUP_TYPE_FOR_EPISODES | 400 | `volume` タイプの group に episode を作成しようとした |
| DUPLICATE_EPISODE_NUMBER | 409 | 同一 group 内で episode_number が重複 |
| STAFF_NOT_FOUND | 404 | 指定した staff が存在しない |
| INVALID_PROVIDER | 400 | `provider` パスパラメータが未対応の値 |
| API_KEY_NOT_CONFIGURED | 422 | 外部検索に必要なAPIキーが未登録 |
| EXTERNAL_API_TIMEOUT | 502 | 外部APIの呼び出しタイムアウト・失敗 |
| ITEM_ALREADY_IMPORTED | 409 | 既に同一ソースからインポート済み |
| FILE_STORAGE_WRITE_FAILED | 500 | アップロードファイルの保存に失敗 |
| FILE_NOT_FOUND | 404 | 指定した item file が存在しない |
| STEAM_API_KEY_INVALID | 401 | Steam Web API キーが無効 |

### ページネーション正規化
`page` / `limit` クエリパラメータは以下のルールで補正される（`normalize_pagination`）:
- `page < 1` → `1`
- `limit` 未指定 → `20`（デフォルト）
- `limit < 1` → `20`
- `limit > 100` → `100`

---

## 主要Enum

| Enum | 値 |
|---|---|
| `media_type` | anime, movie, drama, manga, novel, game, academic_book, paper |
| `item_status` | （例: unwatched/watching/completed 等、item のステータス管理に使用） |
| `item_source` | アイテムの取得経路（手動登録／外部API取込／CSVインポート等） |
| `group_type` | season, volume, chapter |
| `relation_type` | reference, dlc |
| `file_type` | pdf, image, other |
| `api_provider` | tmdb, igdb, ndl, steam, open_library, ani_list（jikanは認証不要のため対象外） |

---

## エンドポイント一覧（公開API `/api/v1`）

| Method | Path | 説明 |
|--------|------|------|
| GET | /health | ヘルスチェック |
| GET | /items | アイテム一覧取得（フィルタ・ページネーション） |
| POST | /items | アイテム新規作成 |
| GET | /items/search | 外部API横断検索 |
| POST | /items/import | 外部検索結果からアイテムをインポート |
| GET | /items/{id} | アイテム詳細取得 |
| PATCH | /items/{id} | アイテム更新 |
| DELETE | /items/{id} | アイテム削除 |
| PATCH | /items/{id}/status | ステータス更新 |
| POST | /tags | タグ作成 |
| DELETE | /tags/{id} | タグ削除 |
| POST | /items/{id}/tags/{tag_id} | アイテムにタグ付与 |
| DELETE | /items/{id}/tags/{tag_id} | アイテムからタグ削除 |
| POST | /categories | カテゴリ作成 |
| DELETE | /categories/{id} | カテゴリ削除 |
| POST | /items/{id}/categories/{category_id} | アイテムにカテゴリ付与 |
| DELETE | /items/{id}/categories/{category_id} | アイテムからカテゴリ削除 |
| POST | /mylists | マイリスト作成 |
| POST | /mylists/{id}/items | マイリストにアイテム追加 |
| DELETE | /mylists/{id}/items/{item_id} | マイリストからアイテム削除 |
| POST | /item-relations | アイテム関連作成 |
| DELETE | /item-relations/{id} | アイテム関連削除 |
| POST | /items/{id}/groups | グループ作成（season/volume/chapter） |
| GET | /items/{id}/groups | グループ一覧取得 |
| POST | /groups/{group_id}/episodes | エピソード作成 |
| GET | /groups/{group_id}/episodes | エピソード一覧取得 |
| POST | /staff | スタッフ作成 |
| POST | /items/{id}/staff | アイテムにスタッフ紐付け |
| DELETE | /items/{id}/staff/{item_staff_id} | アイテムのスタッフ紐付け削除 |
| POST | /items/{id}/files | アイテムファイル情報登録 |
| POST | /items/{id}/files/upload | アイテムファイルアップロード（multipart） |
| PATCH | /items/{id}/files/{file_id}/calibre-link | Calibre連携ID更新 |
| POST | /items/{id}/links | 外部リンク追加 |
| DELETE | /items/{id}/links/{link_id} | 外部リンク削除 |
| POST | /items/{id}/trailers | 予告編リンク追加 |
| DELETE | /items/{id}/trailers/{trailer_id} | 予告編リンク削除 |
| PUT | /settings/api-keys/{provider} | 外部APIキー登録・更新 |
| POST | /import/booklog | Booklog CSVインポート |
| POST | /import/steam | Steamライブラリインポート |

---

## Health

### GET /health
DBへの疎通確認込みのヘルスチェック。

- **認証**: 不要
- **成功レスポンス** (200): `{"success":true,"data":{"status":"ok"}}`
- **エラー**: 500 `INTERNAL_ERROR`（DB接続失敗時）

---

## Items

### GET /items
アイテム一覧取得。フィルタ・ページネーション対応。

- **認証**: 不要
- **クエリパラメータ** (`ListItemsQuery`):
  - `media_type` (string, optional)
  - `tag_id` (uuid, optional)
  - `category_id` (uuid, optional)
  - `is_favorite` (bool, optional)
  - `status` (string, optional)
  - `title` (string, optional) — 部分一致検索
  - `page` (u32, optional, default 1)
  - `limit` (u32, optional, default 20, max 100)
- **成功レスポンス** (200): `PaginatedOk<Item[]>`

### POST /items
アイテム新規作成。

- **認証**: 不要
- **リクエストボディ** (`CreateItemRequest`): 共通フィールド + `media_type` 別詳細フィールド
- **成功レスポンス** (201): `ApiOk<Item>`
- **エラー**: 400 `VALIDATION_ERROR`

### GET /items/search
外部プロバイダAPIを横断検索する（`media_type` に応じてプロバイダを自動選択）。`/items/{id}` より前にルーティング登録。

- **認証**: 不要
- **クエリパラメータ** (`ItemSearchQuery`): `media_type` (必須), `q` (必須, 検索語)
- **成功レスポンス** (200): `ApiOk<ExternalSearchResult[]>`
- **エラー**: 422 `API_KEY_NOT_CONFIGURED`, 502 `EXTERNAL_API_TIMEOUT` / `EXTERNAL_API_ERROR`

### POST /items/import
外部検索結果からアイテムをインポートして作成する。`/items/{id}` より前にルーティング登録。

- **認証**: 不要
- **リクエストボディ** (`ImportItemRequest`): 外部ID・media_type等
- **成功レスポンス** (201): `ApiOk<Item>`
- **エラー**: 409 `ITEM_ALREADY_IMPORTED`, 400 `VALIDATION_ERROR`

### GET /items/{id}
アイテム詳細取得（関連情報含む）。

- **認証**: 不要
- **パスパラメータ**: `id` (uuid)
- **成功レスポンス** (200): `ApiOk<ItemDetail>`
- **エラー**: 404 `ITEM_NOT_FOUND`, 400 `VALIDATION_ERROR`（UUID形式不正）

### PATCH /items/{id}
アイテム更新（部分更新）。

- **認証**: 不要
- **リクエストボディ** (`UpdateItemRequest`): 全フィールド Optional
- **成功レスポンス** (200): `ApiOk<Item>`
- **エラー**: 404 `ITEM_NOT_FOUND`, 400 `VALIDATION_ERROR`

### DELETE /items/{id}
アイテム削除。

- **認証**: 不要
- **成功レスポンス**: 204 No Content
- **エラー**: 404 `ITEM_NOT_FOUND`

### PATCH /items/{id}/status
ステータス更新（視聴済み・読了などの状態遷移）。

- **認証**: 不要
- **リクエストボディ** (`UpdateStatusRequest`): `status` (必須), `consumed_date` (optional)
- **成功レスポンス** (200): `ApiOk<Item>`
- **エラー**: 404 `ITEM_NOT_FOUND`

---

## Tags

### POST /tags
- **リクエストボディ** (`CreateTagRequest`): `name` (必須)
- **成功レスポンス** (201): `ApiOk<Tag>`
- **エラー**: 409 `DUPLICATE_TAG_NAME`

### DELETE /tags/{id}
- **成功レスポンス**: 204
- **エラー**: 404 `TAG_NOT_FOUND`

### POST /items/{id}/tags/{tag_id}
アイテムにタグを付与。
- **成功レスポンス**: 201（空ボディ）

### DELETE /items/{id}/tags/{tag_id}
アイテムからタグを削除。
- **成功レスポンス**: 204
- **エラー**: 404 `TAG_NOT_FOUND`

---

## Categories
タグと構造は同一。

### POST /categories
- **リクエストボディ** (`CreateCategoryRequest`): `name` (必須)
- **成功レスポンス** (201): `ApiOk<Category>`
- **エラー**: 409 `DUPLICATE_CATEGORY_NAME`

### DELETE /categories/{id}
- **成功レスポンス**: 204
- **エラー**: 404 `CATEGORY_NOT_FOUND`

### POST /items/{id}/categories/{category_id}
- **成功レスポンス**: 201（空ボディ）

### DELETE /items/{id}/categories/{category_id}
- **成功レスポンス**: 204
- **エラー**: 404 `CATEGORY_NOT_FOUND`

---

## Mylists

### POST /mylists
- **リクエストボディ** (`CreateMylistRequest`): `name` (必須)
- **成功レスポンス** (201): `ApiOk<Mylist>`

### POST /mylists/{id}/items
- **リクエストボディ** (`AddMylistItemRequest`): `item_id` (必須)
- **成功レスポンス**: 201
- **エラー**: 404 `MYLIST_NOT_FOUND`

### DELETE /mylists/{id}/items/{item_id}
- **成功レスポンス**: 204
- **エラー**: 404 `ITEM_NOT_FOUND`

---

## Item Relations

### POST /item-relations
アイテム間の関連（参照・DLCなど）を作成。

- **リクエストボディ** (`CreateItemRelationRequest`): `item_id`, `related_item_id`, `relation_type` (すべて必須)
- **成功レスポンス** (201): `ApiOk<ItemRelation>`
- **エラー**: 400 `VALIDATION_ERROR`（自己参照）, 409 `DUPLICATE_RELATION`

### DELETE /item-relations/{id}
- **成功レスポンス**: 204
- **エラー**: 404 `ITEM_NOT_FOUND`

---

## Item Groups（season / volume / chapter）

### POST /items/{id}/groups
- **リクエストボディ** (`CreateItemGroupRequest`): `group_type` (必須), `group_name` (必須), `number` (optional), `display_order` (デフォルト 0), `parent_item_id` (optional, ネスト用)
- **成功レスポンス** (201): `ApiOk<ItemGroup>`
- **エラー**: 404 `ITEM_NOT_FOUND`

### GET /items/{id}/groups
- **成功レスポンス** (200): `ApiOk<ItemGroup[]>`
- **エラー**: 404 `ITEM_NOT_FOUND`

---

## Item Episodes

### POST /groups/{group_id}/episodes
- **リクエストボディ** (`CreateItemEpisodeRequest`): `episode_number` (必須), `title` / `original_title` / `air_date` / `description` (optional)
- **成功レスポンス** (201): `ApiOk<ItemEpisode>`
- **エラー**: 404 `GROUP_NOT_FOUND`, 400 `INVALID_GROUP_TYPE_FOR_EPISODES`（`volume` タイプの group には作成不可、DBトリガーでも二重チェック）, 409 `DUPLICATE_EPISODE_NUMBER`

### GET /groups/{group_id}/episodes
- **成功レスポンス** (200): `ApiOk<ItemEpisode[]>`
- **エラー**: 404 `GROUP_NOT_FOUND`

---

## Staff

### POST /staff
- **リクエストボディ** (`CreateStaffRequest`): `name` (必須), `external_id` / `image_url` (optional)
- **成功レスポンス** (201): `ApiOk<Staff>`

### POST /items/{id}/staff
アイテムにスタッフを紐付け（役割・キャラ名付き）。
- **リクエストボディ** (`CreateItemStaffRequest`): `staff_id` (必須), `role` (必須), `character_name` (optional)
- **成功レスポンス** (201): `ApiOk<ItemStaff>`
- **エラー**: 404 `STAFF_NOT_FOUND`

### DELETE /items/{id}/staff/{item_staff_id}
- **成功レスポンス**: 204
- **エラー**: 404 `ITEM_NOT_FOUND`

---

## Item Files

### POST /items/{id}/files
ファイルパス情報のみ登録（実体アップロードなし）。
- **リクエストボディ** (`CreateItemFileRequest`): `path` (必須), `label` (optional), `file_type` (必須)
- **成功レスポンス** (201): `ApiOk<ItemFile>`
- **エラー**: 404 `ITEM_NOT_FOUND`, 400 `VALIDATION_ERROR`

### POST /items/{id}/files/upload
実ファイルをアップロードして保存。ボディサイズ上限は本エンドポイントのみ100MBに拡張（`DefaultBodyLimit::max`）。
- **Content-Type**: `multipart/form-data`（`file`, `file_type`, `label` optional）
- **成功レスポンス** (201): `ApiOk<ItemFile>`
- **エラー**: 404 `ITEM_NOT_FOUND`, 400 `VALIDATION_ERROR`, 500 `FILE_STORAGE_WRITE_FAILED`

### PATCH /items/{id}/files/{file_id}/calibre-link
PDFファイルとCalibre書籍IDを紐付ける。
- **リクエストボディ** (`UpdateCalibreLinkRequest`): `calibre_book_id` (必須)
- **成功レスポンス** (200): `ApiOk<ItemFile>`
- **エラー**: 404 `FILE_NOT_FOUND`, 400 `VALIDATION_ERROR`（対象が pdf 以外の file_type、または id 不正）

---

## Item Links

### POST /items/{id}/links
- **リクエストボディ** (`CreateItemLinkRequest`): `url` (必須), `label` (必須)
- **成功レスポンス** (201): `ApiOk<ItemLink>`
- **エラー**: 404 `ITEM_NOT_FOUND`

### DELETE /items/{id}/links/{link_id}
- **成功レスポンス**: 204
- **エラー**: 404 `ITEM_NOT_FOUND`

---

## Item Trailers

### POST /items/{id}/trailers
- **リクエストボディ** (`CreateItemTrailerRequest`): `url` (必須), `label` (optional)
- **成功レスポンス** (201): `ApiOk<ItemTrailer>`
- **エラー**: 404 `ITEM_NOT_FOUND`

### DELETE /items/{id}/trailers/{trailer_id}
- **成功レスポンス**: 204
- **エラー**: 404 `ITEM_NOT_FOUND`

---

## Settings

### PUT /settings/api-keys/{provider}
外部API連携キーを登録・更新（upsert）。

- **パスパラメータ**: `provider` ∈ `tmdb`, `igdb`, `ndl`, `steam`, `open_library`, `ani_list`（`jikan` はキー不要のため対象外）
- **リクエストボディ** (`UpdateApiKeyRequest`): `api_key` (必須)
- **成功レスポンス** (200): `ApiOk<ApiCredential>`
- **エラー**: 400 `INVALID_PROVIDER`

---

## Import

### POST /import/booklog
Booklog エクスポートCSVを取り込む。行単位で成否を集計し、失敗があっても200を返す。

- **Content-Type**: `multipart/form-data`（フィールド名 `file` または `csv`）
- **成功レスポンス** (200): `ApiOk<ImportSummary>`
  ```json
  {
    "success_count": 10,
    "failure_count": 2,
    "failures": [ { "row_number": 5, "reason": "..." } ]
  }
  ```
- **エラー**: 400 `VALIDATION_ERROR`（ファイル未指定・空）

### POST /import/steam
Steam Web API 経由でユーザーのゲームライブラリをインポートする。

- **リクエストボディ** (`SteamImportRequest`): `steam_id` (必須)
- **成功レスポンス** (200): `ApiOk<ImportSummary>`
- **エラー**: 400 `VALIDATION_ERROR`, 401 `STEAM_API_KEY_INVALID`, 502 `EXTERNAL_API_TIMEOUT`

---

## 内部API（`/internal/*`）

すべてのルートに `api_key_auth` ミドルウェアが適用される。`Authorization` ヘッダに `INTERNAL_API_KEY` の値（生値または `Bearer <key>`）が必要。バッチ処理・監視ツールなど、サーバー間連携を想定した用途。

| Method | Path | 説明 |
|--------|------|------|
| POST | /internal/items | アイテム新規作成（公開APIと同一ハンドラを再利用） |
| GET | /internal/items/search | アイテム検索（`title` 等でフィルタ、`list_items_handler` を再利用） |
| PATCH | /internal/items/{id} | アイテム更新（公開APIと同一ハンドラを再利用） |
| POST | /internal/items/{id}/groups | グループの upsert（`item_id, group_type, number` で一意） |
| POST | /internal/groups/{group_id}/episodes | エピソードの upsert（`group_id, episode_number` で一意） |
| POST | /internal/items/{id}/files | ファイル情報登録（公開APIと同一ハンドラを再利用） |

### POST /internal/items
`POST /items` と同じリクエスト/レスポンス仕様。

### GET /internal/items/search
`GET /items` と同じクエリパラメータ・レスポンス仕様（`PaginatedOk<Item[]>`）。

### PATCH /internal/items/{id}
`PATCH /items/{id}` と同じ。
- **エラー**: 404 `ITEM_NOT_FOUND`

### POST /internal/items/{id}/groups
`(item_id, group_type, number)` の組で既存レコードがあれば更新、なければ作成する。
- **成功レスポンス**: 201（新規作成時）/ 200（更新時）
- **エラー**: 404 `ITEM_NOT_FOUND`

### POST /internal/groups/{group_id}/episodes
`(group_id, episode_number)` の組で既存レコードがあれば更新、なければ作成する。
- **成功レスポンス**: 201（新規作成時）/ 200（更新時）
- **エラー**: 404 `GROUP_NOT_FOUND`, 400 `INVALID_GROUP_TYPE_FOR_EPISODES`

### POST /internal/items/{id}/files
`POST /items/{id}/files` と同じ。
- **エラー**: 404 `ITEM_NOT_FOUND`
