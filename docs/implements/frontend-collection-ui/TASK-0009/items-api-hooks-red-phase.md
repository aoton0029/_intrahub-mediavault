# Redフェーズ記録: items APIフック

**作成日**: 2026-07-01
**テストファイル**: `frontend/src/api/items.test.ts`

## 作成したテストケース一覧

| ID | テスト名 | 信頼性 |
|---|---|---|
| TC-IQ-N-01 | フィルタなしでGET /itemsを呼び出す | 🔵 |
| TC-IQ-N-02 | mediaType/page/limitフィルタ付きでGET /itemsを呼び出す | 🔵 |
| TC-IQ-N-03 | isFavorite/status/tagId/categoryIdフィルタ付きで呼び出す | 🔵 |
| TC-IQ-N-06 | queryKeyが['items', filters]形式である | 🔵 |
| TC-IQ-N-11 | undefinedフィールドはクエリパラメータに含まれない | 🔵 |
| TC-IQ-E-01 | APIエラー時にApiClientErrorを返す（useItemsQuery） | 🔵 |
| TC-IQ-N-05 | 存在するidでGET /items/:idを呼び出しItemを返す | 🔵 |
| TC-IQ-N-07 | queryKeyが['items', 'detail', id]形式である | 🔵 |
| TC-IQ-B-01 | idが空文字のときfetchしない（enabled=false） | 🔵 |
| TC-IQ-E-02 | APIエラー時にApiClientErrorを返す（useItemQuery） | 🔵 |
| TC-IQ-E-05 | ネットワークエラー時にNETWORK_ERRORが伝播する | 🔵 |
| TC-IQ-N-08 | 削除成功時にinvalidateQueriesが['items']で呼ばれる | 🔵 |
| TC-IQ-B-04 | 削除成功後にinvalidateQueriesが1回のみ呼ばれる | 🔵 |
| TC-IQ-E-04 | APIエラー時にApiClientErrorを返す（useDeleteItemMutation） | 🔵 |
| TC-IQ-N-09 | 成功時に一覧と詳細の両方のキャッシュが無効化される | 🔵 |
| TC-IQ-N-10 | 成功時に更新後のItemを返す | 🔵 |
| TC-IQ-E-03 | バリデーションエラー時にApiClientErrorを伝播する | 🟡 |

**合計**: 17ケース（🔵 16件、🟡 1件）

## 期待される失敗内容

```
FAIL src/api/items.test.ts
Error: Failed to resolve import "./items" from "src/api/items.test.ts". Does the file exist?
```

`frontend/src/api/items.ts` が存在しないため、インポートエラーでテストが失敗する。

## Greenフェーズで実装すべき内容

`frontend/src/api/items.ts` に以下を実装する:

1. **fetch関数群**:
   - `fetchItems(filters: ItemListFilters)` — `ItemListFilters` → URLSearchParams変換後に `GET /items` を呼ぶ
   - `fetchItem(id: string)` — `GET /items/:id`
   - `deleteItem(id: string)` — `DELETE /items/:id`
   - `updateItemStatus(id: string, body: UpdateItemStatusRequest)` — `PATCH /items/:id/status`

2. **TanStack Queryフック群**:
   - `useItemsQuery(filters)` — `queryKey: ['items', filters]`
   - `useItemQuery(id)` — `queryKey: ['items', 'detail', id]`、`enabled: !!id`
   - `useDeleteItemMutation()` — `onSuccess: invalidateQueries({ queryKey: ['items'] })`
   - `useUpdateItemStatusMutation()` — `onSuccess: invalidateQueries(['items']) + invalidateQueries(['items','detail',id])`
