← [index](./index.md)

# Staff API

## POST /staff
- **リクエストボディ** (`CreateStaffRequest`): `name` (必須), `external_id` / `image_url` (optional)
- **成功レスポンス** (201): `ApiOk<Staff>`

## GET /items/{id}/staff
指定アイテムに紐づくスタッフ紐付けを一覧取得する。
- **成功レスポンス** (200): `ApiOk<ItemStaff[]>`

## POST /items/{id}/staff
アイテムにスタッフを紐付け（役割・キャラ名付き）。
- **リクエストボディ** (`CreateItemStaffRequest`): `staff_id` (必須), `role` (必須), `character_name` (optional)
- **成功レスポンス** (201): `ApiOk<ItemStaff>`
- **エラー**: 404 `STAFF_NOT_FOUND`

## DELETE /items/{id}/staff/{item_staff_id}
- **成功レスポンス**: 204
- **エラー**: 404 `ITEM_NOT_FOUND`
