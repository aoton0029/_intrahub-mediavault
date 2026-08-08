← [index](./index.md)

# Item Relations API

## relation_type と向きの意味

`item_id` を起点、`related_item_id` を終点とする有向の関連。種別ごとに向きの意味が異なる。

| 値 | `item_id`（起点） | `related_item_id`（終点） |
|---|---|---|
| `adaptation` | 原作 | 映像化・翻案作品 |
| `sequel` | 前作 | 続編 |
| `prequel` | 後の作品 | 前日譚 |
| `spinoff` | 本編 | スピンオフ |
| `dlc` | 本編 | DLC・追加コンテンツ |
| `reference` | 引用元 | 引用先 |

上記6値以外は 400 `VALIDATION_ERROR` で拒否される。

## GET /items/{id}/relations
指定アイテムを起点とする関連付けを作成日時昇順で一覧取得する。
- **成功レスポンス** (200): `ApiOk<ItemRelation[]>`

## POST /item-relations
アイテム間の関連を作成。

- **リクエストボディ** (`CreateItemRelationRequest`): `item_id`, `related_item_id`, `relation_type` (すべて必須)
- **成功レスポンス** (201): `ApiOk<ItemRelation>`
- **エラー**: 400 `VALIDATION_ERROR`（自己参照・不正な `relation_type`）, 409 `DUPLICATE_RELATION`（同一の `item_id` / `related_item_id` / `relation_type` の組み合わせ）

## DELETE /item-relations/{id}
- **成功レスポンス**: 204
- **エラー**: 404 `ITEM_NOT_FOUND`
