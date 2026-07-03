← [index](./index.md)

# Mylists API

## POST /mylists
- **リクエストボディ** (`CreateMylistRequest`): `name` (必須)
- **成功レスポンス** (201): `ApiOk<Mylist>`

## POST /mylists/{id}/items
- **リクエストボディ** (`AddMylistItemRequest`): `item_id` (必須)
- **成功レスポンス**: 201
- **エラー**: 404 `MYLIST_NOT_FOUND`

## DELETE /mylists/{id}/items/{item_id}
- **成功レスポンス**: 204
- **エラー**: 404 `ITEM_NOT_FOUND`
