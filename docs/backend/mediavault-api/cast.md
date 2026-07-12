← [index](./index.md)

# Cast API

キャスト（声優＋役名）は`staff`とは別テーブル（`cast_members` / `item_cast`）で管理する。`staff.md`のスタッフ（監督・脚本等のクルー）とは分離されており、`role`列は持たない。

## POST /cast
- **リクエストボディ** (`CreateCastRequest`): `name` (必須), `external_id` / `image_url` (optional)
- **成功レスポンス** (201): `ApiOk<Cast>`

## GET /items/{id}/cast
指定アイテムに紐づくキャスト紐付けを一覧取得する。
- **成功レスポンス** (200): `ApiOk<ItemCast[]>`

## POST /items/{id}/cast
アイテムにキャストを紐付け（役名付き）。
- **リクエストボディ** (`CreateItemCastRequest`): `cast_id` (必須), `character_name` (optional)
- **成功レスポンス** (201): `ApiOk<ItemCast>`
- **エラー**: 404 `CAST_NOT_FOUND`

## DELETE /items/{id}/cast/{item_cast_id}
- **成功レスポンス**: 204
- **エラー**: 404 `ITEM_NOT_FOUND`
