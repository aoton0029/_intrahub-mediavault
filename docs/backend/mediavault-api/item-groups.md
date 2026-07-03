← [index](./index.md)

# Item Groups API（season / volume / chapter）

## POST /items/{id}/groups
- **リクエストボディ** (`CreateItemGroupRequest`): `group_type` (必須), `group_name` (必須), `number` (optional), `display_order` (デフォルト 0), `parent_item_id` (optional, ネスト用)
- **成功レスポンス** (201): `ApiOk<ItemGroup>`
- **エラー**: 404 `ITEM_NOT_FOUND`

## GET /items/{id}/groups
- **成功レスポンス** (200): `ApiOk<ItemGroup[]>`
- **エラー**: 404 `ITEM_NOT_FOUND`
