← [index](./index.md)

# Tags API

## GET /tags
全タグを、付与アイテム件数(`item_count`)付きで一覧取得する。ページネーションなし。

- **認証**: 不要
- **成功レスポンス** (200): `ApiOk<TagWithCount[]>`（`TagWithCount = { id: UUID, name: string, item_count: number }`）

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
