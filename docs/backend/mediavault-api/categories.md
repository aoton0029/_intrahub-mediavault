← [index](./index.md)

# Categories API
タグと構造は同一。

## GET /categories
全カテゴリを、付与アイテム件数(`item_count`)付きで一覧取得する。ページネーションなし。

- **認証**: 不要
- **成功レスポンス** (200): `ApiOk<CategoryWithCount[]>`（`CategoryWithCount = { id: UUID, name: string, item_count: number }`）

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
