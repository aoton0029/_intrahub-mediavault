# TASK-0009 コンテキストノート: items APIフック実装

## 1. 技術スタック

- **フレームワーク**: React 18.3+ / TypeScript 5.7+ / Vite 6
- **サーバー状態管理**: TanStack Query v5（`useQuery` / `useMutation` / `useQueryClient`）
- **テストフレームワーク**: Vitest + @testing-library/react（jsdom環境）
- **パッケージマネージャ**: yarn（frontendディレクトリ内）
- **エイリアス**: `@/` → `frontend/src/`
- 参照元: `frontend/vitest.config.ts`, `frontend/CLAUDE.md`, `docs/spec/frontend-collection-ui/note.md`

## 2. 開発ルール

- テストファイルは実装ファイルと同ディレクトリに `*.test.ts` として配置する（例: `client.test.ts`）
- テストケースIDは `TC-{略称}-{カテゴリ}-{連番}` 形式（例: `TC-IQ-N-01`）
- テスト記述パターン: `describe` + `it` の二段構成、`vi.stubGlobal` でfetchをモック
- `afterEach` で `vi.unstubAllGlobals()` を呼んでモックをリセット
- TanStack QueryフックのテストにはQueryClientProviderラッパーが必要
- 参照元: `frontend/src/api/client.test.ts`, `frontend/src/hooks/useSearchParamsFilter.test.ts`

## 3. 関連実装

### apiClient（TASK-0005で実装済み）
- **ファイル**: `frontend/src/api/client.ts`
- **シグネチャ**: `apiClient<T>(path: string, options?: RequestOptions): Promise<{ data: T; pagination?: Pagination }>`
- **エラー**: `ApiClientError(code, message)` をthrow
- **メソッド対応**: GET/POST/PATCH/DELETE/PUT + FormData対応

### 汎用フック（TASK-0008で実装済み）
- `frontend/src/hooks/useSearchParamsFilter.ts` — URLクエリパラメータとItemListFiltersの変換
- `frontend/src/hooks/useConfirmDialog.ts` — 確認ダイアログ状態管理

### 型定義（TASK-0004で配置済み）
- **ファイル**: `frontend/src/types/index.ts`
- 使用する主な型: `Item`, `ItemListFilters`, `UpdateItemStatusRequest`, `ApiClientError`, `Pagination`

## 4. 設計文書

- **タスク定義**: `docs/tasks/frontend-collection-ui/TASK-0009.md`
- **型定義設計**: `docs/design/frontend-collection-ui/interfaces.ts`
- **アーキテクチャ**: `docs/design/frontend-collection-ui/architecture.md`
- **データフロー**: `docs/design/frontend-collection-ui/dataflow.md`
- **バックエンドAPI**: `docs/design/mediavault-backend/api-endpoints.md`

### APIエンドポイント（対象）
| メソッド | パス | 用途 |
|---|---|---|
| GET | `/items` | 一覧取得（フィルタ・ページング） |
| GET | `/items/:id` | 詳細取得 |
| DELETE | `/items/:id` | 削除 |
| PATCH | `/items/:id/status` | ステータス更新 |

### フィルタ→URLSearchParams変換ルール
`ItemListFilters` のプロパティ名とクエリパラメータ名の対応:
| TypeScriptプロパティ | クエリパラメータ |
|---|---|
| `mediaType` | `media_type` |
| `tagId` | `tag_id` |
| `categoryId` | `category_id` |
| `isFavorite` | `is_favorite` |
| `status` | `status` |
| `page` | `page` |
| `limit` | `limit` |
undefinedフィールドはクエリパラメータに含めない。

## 5. テスト関連情報

- **テスト設定**: `frontend/vitest.config.ts`（jsdom環境, globals: true）
- **setupファイル**: `frontend/src/test/setup.ts`（`@testing-library/jest-dom/vitest` をインポート）
- **既存テストのパターン**:
  - `vi.stubGlobal('fetch', vi.fn().mockResolvedValue(...))` でfetchをモック
  - TanStack Queryフックには `QueryClient` + `QueryClientProvider` でラップが必要
  - `renderHook` を使用し、`waitFor` でデータ取得を待機
- **TanStack Query テストパターン**:
  ```ts
  import { renderHook, waitFor } from '@testing-library/react'
  import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
  function createWrapper() {
    const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
    return ({ children }) => createElement(QueryClientProvider, { client: queryClient }, children)
  }
  ```
- **Mutationテスト**: `result.current.mutate(...)` → `waitFor(() => expect(...).toBe('success'))`
- 参照元: `frontend/src/api/client.test.ts`, `frontend/vitest.config.ts`

## 6. 注意事項

### 実装ファイルパス
- **実装先**: `frontend/src/api/items.ts`

### キャッシュ戦略
- `useItemsQuery` のqueryKey: `['items', filters]`（filtersオブジェクト全体をキーに含める）
- `useItemQuery` のqueryKey: `['items', 'detail', id]`
- `useDeleteItemMutation` のonSuccess: `invalidateQueries({ queryKey: ['items'] })`
- `useUpdateItemStatusMutation` のonSuccess: 一覧 + 詳細の両方を無効化

### テストでの注意
- TanStack Query v5では `useQuery` の `onError` コールバックが廃止済み。エラーは `result.current.error` で確認する
- `useMutation` の変数型は `mutate(variables)` の引数型。`useDeleteItemMutation` は `mutate(id: string)` 形式
- `queryClient.invalidateQueries` の呼び出し確認には `vi.spyOn(queryClient, 'invalidateQueries')` を使う
- `retry: false` を QueryClient に設定してテスト時の再試行を無効化する
