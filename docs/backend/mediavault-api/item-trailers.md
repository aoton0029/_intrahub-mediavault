← [index](./index.md)

# Item Trailers API

## POST /items/{id}/trailers
- **リクエストボディ** (`CreateItemTrailerRequest`): `url` (必須), `label` (optional)
- **成功レスポンス** (201): `ApiOk<ItemTrailer>`
- **エラー**: 404 `ITEM_NOT_FOUND`

## DELETE /items/{id}/trailers/{trailer_id}
- **成功レスポンス**: 204
- **エラー**: 404 `ITEM_NOT_FOUND`
