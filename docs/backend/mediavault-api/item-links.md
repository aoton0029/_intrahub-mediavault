← [index](./index.md)

# Item Links API

## POST /items/{id}/links
- **リクエストボディ** (`CreateItemLinkRequest`): `url` (必須), `label` (必須)
- **成功レスポンス** (201): `ApiOk<ItemLink>`
- **エラー**: 404 `ITEM_NOT_FOUND`

## DELETE /items/{id}/links/{link_id}
- **成功レスポンス**: 204
- **エラー**: 404 `ITEM_NOT_FOUND`
