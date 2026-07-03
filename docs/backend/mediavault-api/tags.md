← [index](./index.md)

# Tags API

## POST /tags
- **リクエストボディ** (`CreateTagRequest`): `name` (必須)
- **成功レスポンス** (201): `ApiOk<Tag>`
- **エラー**: 409 `DUPLICATE_TAG_NAME`

## DELETE /tags/{id}
- **成功レスポンス**: 204
- **エラー**: 404 `TAG_NOT_FOUND`

## POST /items/{id}/tags/{tag_id}
アイテムにタグを付与。
- **成功レスポンス**: 201（空ボディ）

## DELETE /items/{id}/tags/{tag_id}
アイテムからタグを削除。
- **成功レスポンス**: 204
- **エラー**: 404 `TAG_NOT_FOUND`
