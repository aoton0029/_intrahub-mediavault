← [index](./index.md)

# Categories API
タグと構造は同一。

## POST /categories
- **リクエストボディ** (`CreateCategoryRequest`): `name` (必須)
- **成功レスポンス** (201): `ApiOk<Category>`
- **エラー**: 409 `DUPLICATE_CATEGORY_NAME`

## DELETE /categories/{id}
- **成功レスポンス**: 204
- **エラー**: 404 `CATEGORY_NOT_FOUND`

## POST /items/{id}/categories/{category_id}
- **成功レスポンス**: 201（空ボディ）

## DELETE /items/{id}/categories/{category_id}
- **成功レスポンス**: 204
- **エラー**: 404 `CATEGORY_NOT_FOUND`
