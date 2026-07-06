← [index](./index.md)

# Item Relations API

## GET /items/{id}/relations
指定アイテムを起点とする関連付け（参照・DLC）を作成日時昇順で一覧取得する。
- **成功レスポンス** (200): `ApiOk<ItemRelation[]>`

## POST /item-relations
アイテム間の関連（参照・DLCなど）を作成。

- **リクエストボディ** (`CreateItemRelationRequest`): `item_id`, `related_item_id`, `relation_type` (すべて必須)
- **成功レスポンス** (201): `ApiOk<ItemRelation>`
- **エラー**: 400 `VALIDATION_ERROR`（自己参照）, 409 `DUPLICATE_RELATION`

## DELETE /item-relations/{id}
- **成功レスポンス**: 204
- **エラー**: 404 `ITEM_NOT_FOUND`
