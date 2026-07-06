← [index](./index.md)

# Item Links API

## GET /items/{id}/links
指定アイテムに紐づく参考リンクを作成日時昇順で一覧取得する。
- **成功レスポンス** (200): `ApiOk<ItemLink[]>`

## POST /items/{id}/links
- **リクエストボディ** (`CreateItemLinkRequest`): `url` (必須), `label` (必須)
- **成功レスポンス** (201): `ApiOk<ItemLink>`
- **エラー**: 404 `ITEM_NOT_FOUND`

## DELETE /items/{id}/links/{link_id}
- **成功レスポンス**: 204
- **エラー**: 404 `ITEM_NOT_FOUND`
