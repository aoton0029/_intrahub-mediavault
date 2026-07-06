← [index](./index.md)

# Mylists API

## GET /mylists
全マイリストを作成日時昇順で一覧取得する。
- **成功レスポンス** (200): `ApiOk<Mylist[]>`

## POST /mylists
- **リクエストボディ** (`CreateMylistRequest`): `name` (必須)
- **成功レスポンス** (201): `ApiOk<Mylist>`

## GET /items/{id}/mylists
指定アイテムが所属する全マイリストを一覧取得する（`mylist_items`からの逆引き）。
- **成功レスポンス** (200): `ApiOk<Mylist[]>`
- **エラー**: 400 `VALIDATION_ERROR`（idがUUID形式でない場合）

## POST /mylists/{id}/items
- **リクエストボディ** (`AddMylistItemRequest`): `item_id` (必須)
- **成功レスポンス**: 201
- **エラー**: 404 `MYLIST_NOT_FOUND`

## DELETE /mylists/{id}/items/{item_id}
- **成功レスポンス**: 204
- **エラー**: 404 `ITEM_NOT_FOUND`
