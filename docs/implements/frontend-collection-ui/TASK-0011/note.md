# TASK-0011: HomePage（全体一覧）実装 - 開発ノート

## 1. 技術スタック

- **フレームワーク**: React 18.3+ / TypeScript 5.7+ / Vite 6
- **状態管理**: TanStack Query v5（サーバー状態）、React built-in（UI状態）
- **ルーティング**: React Router v7
- **スタイリング**: Tailwind CSS 4 + shadcn/ui
- **テスト**: Vitest + Testing Library + jsdom

- 参照元: docs/spec/frontend-collection-ui/note.md

## 2. 開発ルール

- **アーキテクチャ**: Feature-Sliced layered（pages → features → components → api/hooks/types/lib）
- **エラーハンドリング**: apiClient が `{success:false, error:{code,message}}` 形式をチェックして ApiClientError をスロー
- **テスト注意**: shadcn/ui の Select はPortal/jsdom問題があるためネイティブ `<select>` を使うこと
- **userEvent**: v14+ API (`await userEvent.setup()` → `userEvent.selectOptions()`, `userEvent.click()`)
- **TanStack Query v5**: `onError` コールバック廃止 → `result.current.error` を使う
- **ファイルパスはプロジェクトルートからの相対パスで記載**

- 参照元: docs/design/frontend-collection-ui/architecture.md, docs/implements/frontend-collection-ui/TASK-0010/note.md

## 3. 関連実装

### 使用するフック・コンポーネント
- `useItemsQuery(filters)` — `frontend/src/api/items.ts` — queryKey=['items', filters]
- `useSearchParamsFilter()` — `frontend/src/hooks/useSearchParamsFilter.ts` — URL ↔ ItemListFilters同期
- `FilterBar` — `frontend/src/components/common/FilterBar.tsx` — controlled component
- `EmptyState` — `frontend/src/components/common/EmptyState.tsx` — 0件時表示
- `MediaCard` — `frontend/src/components/common/MediaCard.tsx` — アイテムカード表示

### FilterBar Props
```ts
interface FilterBarProps {
  filters: ItemListFilters
  onChange: (filters: ItemListFilters) => void
  disabled?: boolean
  tagOptions: Tag[]
  categoryOptions: Category[]
  mediaTypeOptions?: MediaType[]
}
```

### useItemsQuery の戻り値
- `data.items`: Item[]
- `data.pagination`: { page, limit, total }
- `isLoading`, `isError`, `error`

- 参照元: frontend/src/api/items.ts, frontend/src/hooks/useSearchParamsFilter.ts, frontend/src/components/common/FilterBar.tsx

## 4. 設計文書

### ページルーティング
- HomePage: `/` (全体一覧)
- 詳細画面: `/items/:id`
- 追加画面: `/search/general` 等

### データフロー（機能1: 全体一覧の閲覧・絞り込み）
1. HomePage マウント → `useSearchParamsFilter` でURLクエリ読み込み
2. `useItemsQuery(filters)` でGET `/items?filters` 実行
3. レスポンス → MediaCard グリッド表示
4. FilterBar操作 → `setFilters` → URL更新 → queryKey変化 → 再fetch

### グリッドレイアウト（モバイル対応）
```
grid-cols-2 md:grid-cols-4 lg:grid-cols-6
```

- 参照元: docs/design/frontend-collection-ui/dataflow.md, docs/design/frontend-collection-ui/architecture.md

## 5. テスト関連情報

### テストフレームワーク
- **設定ファイル**: `frontend/vitest.config.ts`
  - environment: 'jsdom', globals: true
  - setupFiles: ['./src/test/setup.ts']
  - exclude: ['node_modules/**', 'tests/e2e/**']
  - Alias: `@`: `./src`
- **セットアップ**: `frontend/src/test/setup.ts` → `@testing-library/jest-dom/vitest` インポート

### テストファイルのディレクトリ構成・命名パターン
- `frontend/src/` 直下にテストファイルを配置 (`*.test.tsx`, `*.test.ts`)
- コンポーネントテスト例: `frontend/src/components/common/FilterBar.test.tsx`
- フックテスト例: `frontend/src/hooks/useSearchParamsFilter.test.ts`
- **HomePgeテスト配置先**: `frontend/src/pages/HomePage.test.tsx`

### テストIDの命名規則（TASK-0009,0010より）
- 正常系: `TC-HP-N-001`, `TC-HP-N-002`, ...
- 境界値: `TC-HP-B-001`, ...
- 異常系: `TC-HP-E-001`, ...

### TanStack Query のテストパターン（TASK-0009より）
```ts
const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
const wrapper = ({ children }) =>
  createElement(QueryClientProvider, { client: queryClient }, children)
// renderHook + waitFor
```

### React Router のテストパターン
```ts
import { MemoryRouter } from 'react-router'
// render(<MemoryRouter initialEntries={['/']}><HomePage /></MemoryRouter>)
```

### モック方針
- `useItemsQuery` → `vi.mock('@/api/items')` でモック
- `useSearchParamsFilter` → `vi.mock('@/hooks/useSearchParamsFilter')` でモック
- `useNavigate` → `vi.mock('react-router')` でモック可能

- 参照元: frontend/vitest.config.ts, frontend/src/test/setup.ts, docs/implements/frontend-collection-ui/TASK-0009/note.md, docs/implements/frontend-collection-ui/TASK-0010/note.md

## 6. 注意事項

- **TASK-0005 (apiClient)未完了**: overview.md によると TASK-0005 は未完了だが、TASK-0009 ではすでに `useItemsQuery` 等が実装済み。テストでは直接モックするため影響なし。
- **カード/リスト切り替え**: PRDに詳細記載なし → シンプルなトグルボタンで実装
- **useTagsQuery / useCategoriesQuery**: 別タスクで実装予定。未実装時は FilterBar の select を `disabled` にする（空配列を渡す）
- **ページング**: URLの `page` クエリパラメータを更新する方式
- **TASK-0012〜0014への共有**: `useItemListPage` カスタムフックまたは `ItemListView` 共通コンポーネントへの抽出を検討すること

- 参照元: docs/tasks/frontend-collection-ui/TASK-0011.md, docs/tasks/frontend-collection-ui/overview.md
