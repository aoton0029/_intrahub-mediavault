← [index](./index.md)

# 内部API（`/internal/*`）

すべてのルートに `api_key_auth` ミドルウェアが適用される。`Authorization` ヘッダに `INTERNAL_API_KEY` の値（生値または `Bearer <key>`）が必要。バッチ処理・監視ツールなど、サーバー間連携を想定した用途。

| Method | Path | 説明 |
|--------|------|------|
| POST | /internal/items | アイテム新規作成（公開APIと同一ハンドラを再利用） |
| GET | /internal/items/search | アイテム検索（`title` 等でフィルタ、`list_items_handler` を再利用） |
| PATCH | /internal/items/{id} | アイテム更新（公開APIと同一ハンドラを再利用） |
| POST | /internal/items/{id}/groups | グループの upsert（`item_id, group_type, number` で一意） |
| POST | /internal/groups/{group_id}/episodes | エピソードの upsert（`group_id, episode_number` で一意） |
| POST | /internal/items/{id}/files | ファイル情報登録（公開APIと同一ハンドラを再利用） |

## POST /internal/items
`POST /items` と同じリクエスト/レスポンス仕様。

## GET /internal/items/search
`GET /items` と同じクエリパラメータ・レスポンス仕様（`PaginatedOk<Item[]>`）。

## PATCH /internal/items/{id}
`PATCH /items/{id}` と同じ。
- **エラー**: 404 `ITEM_NOT_FOUND`

## POST /internal/items/{id}/groups
`(item_id, group_type, number)` の組で既存レコードがあれば更新、なければ作成する。
- **成功レスポンス**: 201（新規作成時）/ 200（更新時）
- **エラー**: 404 `ITEM_NOT_FOUND`

## POST /internal/groups/{group_id}/episodes
`(group_id, episode_number)` の組で既存レコードがあれば更新、なければ作成する。
- **成功レスポンス**: 201（新規作成時）/ 200（更新時）
- **エラー**: 404 `GROUP_NOT_FOUND`, 400 `INVALID_GROUP_TYPE_FOR_EPISODES`

## POST /internal/items/{id}/files
`POST /items/{id}/files` と同じ。
- **エラー**: 404 `ITEM_NOT_FOUND`
