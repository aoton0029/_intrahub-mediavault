# items APIフック要件定義書

**機能名**: items APIフック実装
**タスクID**: TASK-0009
**要件名**: frontend-collection-ui
**作成日**: 2026-07-01

---

## 1. 機能の概要

🔵 **システム内での位置づけ**: `frontend/src/api/items.ts` に実装するAPIレイヤー。TanStack Query v5のフックをラップし、コンポーネントからのデータアクセスを抽象化する。

🔵 **提供するフックと関数**:
- `fetchItems(filters)` — GET /items 呼び出し関数
- `fetchItem(id)` — GET /items/:id 呼び出し関数
- `deleteItem(id)` — DELETE /items/:id 呼び出し関数
- `updateItemStatus(id, body)` — PATCH /items/:id/status 呼び出し関数
- `useItemsQuery(filters)` — 一覧取得クエリフック
- `useItemQuery(id)` — 詳細取得クエリフック
- `useDeleteItemMutation()` — 削除ミューテーションフック
- `useUpdateItemStatusMutation()` — ステータス更新ミューテーションフック

🔵 **想定ユーザー**: TASK-0011（HomePage）、TASK-0012〜0014（各グループ別一覧ページ）の実装者

- **参照したEARS要件**: REQ-002, REQ-007, REQ-013
- **参照した設計文書**: `docs/design/frontend-collection-ui/architecture.md`「リソース別フック」、`docs/design/frontend-collection-ui/dataflow.md`

---

## 2. 入力・出力の仕様

### 2-1. fetchItems / useItemsQuery

🔵 **入力**:
```ts
filters: ItemListFilters
// {
//   mediaType?: MediaType
//   tagId?: string
//   categoryId?: string
//   isFavorite?: boolean
//   status?: ItemStatus
//   page?: number
//   limit?: number
// }
```

🔵 **URLクエリパラメータ変換ルール**:
| TypeScriptプロパティ | クエリパラメータ | 型 |
|---|---|---|
| `mediaType` | `media_type` | string |
| `tagId` | `tag_id` | string |
| `categoryId` | `category_id` | string |
| `isFavorite` | `is_favorite` | boolean→string |
| `status` | `status` | string |
| `page` | `page` | number→string |
| `limit` | `limit` | number→string |

🔵 **undefinedフィールドはクエリパラメータに含めない**（URLSearchParamsに追加しない）

🔵 **出力**:
```ts
// fetchItems の戻り値
{ data: Item[]; pagination?: Pagination }

// useItemsQuery の戻り値（TanStack Query UseQueryResult）
{
  data: { data: Item[]; pagination?: Pagination } | undefined
  isLoading: boolean
  error: ApiClientError | null
  // ...
}
```

🔵 **queryKey**: `['items', filters]`

### 2-2. fetchItem / useItemQuery

🔵 **入力**: `id: string`

🔵 **出力**:
```ts
// fetchItem の戻り値
{ data: Item; pagination?: Pagination }

// useItemQuery の戻り値
{
  data: { data: Item; pagination?: Pagination } | undefined
  isLoading: boolean
  error: ApiClientError | null
}
```

🔵 **queryKey**: `['items', 'detail', id]`

🔵 **enabled**: `!!id`（idが空文字・undefinedの場合は実行しない）

### 2-3. deleteItem / useDeleteItemMutation

🔵 **入力**: `id: string`

🔵 **出力**: `void`（204 No Content想定）

🔵 **成功時の副作用**: `queryClient.invalidateQueries({ queryKey: ['items'] })`

### 2-4. updateItemStatus / useUpdateItemStatusMutation

🔵 **入力**:
```ts
{ id: string; body: UpdateItemStatusRequest }
// UpdateItemStatusRequest = { status: ItemStatus; consumedDate?: string }
```

🔵 **出力**: `{ data: Item; pagination?: Pagination }`

🔵 **成功時の副作用**:
1. `queryClient.invalidateQueries({ queryKey: ['items'] })` — 一覧キャッシュを無効化
2. `queryClient.invalidateQueries({ queryKey: ['items', 'detail', id] })` — 詳細キャッシュを無効化

- **参照したEARS要件**: REQ-002, REQ-013
- **参照した設計文書**: `docs/design/frontend-collection-ui/interfaces.ts` の `ItemListFilters`, `UpdateItemStatusRequest`, `Item`

---

## 3. 制約条件

🔵 **APIクライアント**: `frontend/src/api/client.ts` の `apiClient` 関数を使用する（TASK-0005で実装済み）

🔵 **エラー伝播**: APIエラー時は `ApiClientError` がそのまま各フックの `error` に伝播する。フック側でcatchしない。

🔵 **TanStack Query v5制約**:
- `useQuery` の `onError` コールバックは廃止済み。エラーはクエリ結果の `error` フィールドで取得
- `useMutation` の変数型は `mutate(variables)` の引数型として定義
- `retry`: デフォルト設定に従う（テスト時は `retry: false` で上書き可能）

🟡 **ベースURL**: `VITE_API_BASE_URL` 環境変数または `http://localhost:8080/api/v1`（apiClientが解決済み）

🔵 **import制約**: `@/` エイリアスを使用（`@/types` → `frontend/src/types/index.ts`）

- **参照したEARS要件**: REQ-402（`/internal/*` はフロントから呼ばない）
- **参照した設計文書**: `docs/design/frontend-collection-ui/architecture.md`

---

## 4. 想定される使用例

### 4-1. 基本的な一覧取得フロー
🔵
```ts
// HomePageでの使用例
const { data, isLoading, error } = useItemsQuery({ mediaType: 'anime', page: 1, limit: 20 })
// → GET /items?media_type=anime&page=1&limit=20
```

### 4-2. フィルタ変更で自動再取得
🔵
```ts
// filtersオブジェクトが変わると queryKey が変化し、TanStack Queryが自動で再取得
const [filters, setFilters] = useState<ItemListFilters>({})
const { data } = useItemsQuery(filters)
// filters変更時に ['items', newFilters] キーで新しいクエリが発行される
```

### 4-3. 詳細取得（idなし時はスキップ）
🔵
```ts
const { data } = useItemQuery(itemId) // itemIdが空文字の場合はfetchしない
```

### 4-4. 削除後の一覧更新
🔵
```ts
const { mutate: deleteItem } = useDeleteItemMutation()
deleteItem('item-id-123')
// 成功後: ['items'] 配下の全クエリが無効化 → 自動再取得
```

### 4-5. ステータス更新後のキャッシュ更新
🔵
```ts
const { mutate: updateStatus } = useUpdateItemStatusMutation()
updateStatus({ id: 'item-id-123', body: { status: 'completed' } })
// 成功後: 一覧キャッシュ + ['items', 'detail', 'item-id-123'] の両方が無効化
```

### 4-6. エラーケース
🔵
```ts
const { error } = useItemsQuery(filters)
// APIエラー時: error instanceof ApiClientError === true
// error.code: 'VALIDATION_ERROR' | 'NETWORK_ERROR' | 'PARSE_ERROR' | etc.
```

🟡 **バリデーションエラー**: 不正なstatusを送信した場合、`ApiClientError(code='VALIDATION_ERROR')` が返る

- **参照したEARS要件**: REQ-002（フィルタ・ページング）, REQ-007（お気に入り）, REQ-013（ステータス更新）
- **参照した設計文書**: `docs/design/frontend-collection-ui/dataflow.md`「機能1: 全体一覧の閲覧・絞り込み」「データ整合性の保証 キャッシュ無効化方針」

---

## 5. EARS要件・設計文書との対応関係

- **参照した機能要件**: REQ-002（フィルタ・ページング）, REQ-007（お気に入りフィルタ）, REQ-013（ステータス更新）
- **参照したEdgeケース**: EDGE（削除成功後のキャッシュ無効化）
- **参照した設計文書**:
  - **アーキテクチャ**: `docs/design/frontend-collection-ui/architecture.md`「リソース別フック」
  - **データフロー**: `docs/design/frontend-collection-ui/dataflow.md`「機能1」「データ整合性の保証」
  - **型定義**: `docs/design/frontend-collection-ui/interfaces.ts` — `ItemListFilters`, `UpdateItemStatusRequest`, `Item`, `Pagination`
  - **API仕様**: `docs/design/mediavault-backend/api-endpoints.md` — `GET /items`, `GET /items/:id`, `DELETE /items/:id`, `PATCH /items/:id/status`

---

## 品質判定

✅ **高品質**
- 要件の曖昧さ: なし（型定義・エンドポイント・queryKey形式が明確）
- 入出力定義: 完全（全フック・全関数の入出力を定義）
- 制約条件: 明確（apiClient依存、TanStack Query v5制約、エラー伝播方針）
- 実装可能性: 確実
- 信頼性レベル: 🔵が大部分（TASK-0009.mdの完了条件に完全対応）
