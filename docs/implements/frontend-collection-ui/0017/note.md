# TASK-0017: ItemDetailPage基本実装 - コンテキストノート

## 1. 技術スタック

- **フレームワーク**: React 18.3+ / TypeScript 5.7+ / Vite 6
- **サーバー状態**: TanStack Query v5 (`useQuery`, `useMutation`)
- **ルーティング**: React Router v7 (`useParams`, `useNavigate`)
- **UIライブラリ**: Tailwind CSS v4 + shadcn/ui
- **テスト**: Vitest + Testing Library (jsdom環境)
- **パッケージマネージャ**: yarn (pnpm記載あるが実際はyarn)

参照元: `docs/spec/frontend-collection-ui/note.md`, `frontend/CLAUDE.md`

## 2. 開発ルール

- TDDサイクル: Red → Green → Refactor
- コメントはWHYが非自明な場合のみ記載
- 型定義は `@/types` から import
- APIフックは `@/api/items` から import
- 共通コンポーネントは `@/components/common/` から import
- shadcn/ui基底コンポーネントは `@/components/ui/` から import

## 3. 関連実装

### 既実装APIフック（`frontend/src/api/items.ts`）

- `useItemQuery(id: string)` - GET /items/:id、queryKey=['items','detail',id]
- `useDeleteItemMutation()` - DELETE /items/:id、成功時に['items']を invalidate
- `useUpdateItemStatusMutation()` - PATCH /items/:id/status、UpdateItemStatusRequest送信

### 既実装共通コンポーネント

- `ConfirmDialog` (`frontend/src/components/common/ConfirmDialog.tsx`)
  - props: `open`, `title`, `description?`, `onConfirm`, `onCancel`, `confirmLabel?`, `cancelLabel?`
  - data-testid="confirm-dialog"
- `MediaTypeBadge` (`frontend/src/components/common/MediaTypeBadge.tsx`)
- `MediaCard` (`frontend/src/components/common/MediaCard.tsx`)
- `EmptyState` (`frontend/src/components/common/EmptyState.tsx`)
- `FilterBar` (`frontend/src/components/common/FilterBar.tsx`)

### 既実装フック

- `useConfirmDialog` (`frontend/src/hooks/useConfirmDialog.ts`)
- `useSearchParamsFilter` (`frontend/src/hooks/useSearchParamsFilter.ts`)

### 実装パターン例（`frontend/src/pages/HomePage.tsx`）

- isLoading → スケルトン表示（data-testid="skeleton-grid"）
- isError → エラーメッセージ＋リトライボタン
- items.length===0 → EmptyState
- QueryClient, MemoryRouter でラップしてテスト

参照元:
- `frontend/src/api/items.ts`
- `frontend/src/components/common/ConfirmDialog.tsx`
- `frontend/src/pages/HomePage.tsx`
- `frontend/src/hooks/useConfirmDialog.ts`

## 4. 設計文書

### 対象ファイル（新規作成）

- `frontend/src/pages/ItemDetailPage.tsx`
- `frontend/src/features/items/ItemDetailHeader.tsx`（任意分割）
- `frontend/src/features/status/StatusUpdateControl.tsx`

### Item 判別共用体（`frontend/src/types/index.ts`）

mediaType別 details フィールド:
| mediaType | detailsの主なフィールド |
|---|---|
| anime | episodeCount, seasonCount, studio, genreList, sourceType, jikanId |
| movie | runtimeMinutes, director, genreList, tmdbId |
| drama | episodeCount, seasonCount, network, genreList, tmdbId |
| manga | volumeCount, chapterCount, author, illustrator, magazine, jikanId |
| novel | volumeCount, author, publisher, isbn, openlibraryId, googleBooksId |
| game | platformList, developer, publisher, steamAppid, igdbId |
| academic_book | author, publisher, isbn, ndlId, googleBooksId |
| paper | doi, journalName, volumeIssue, pageRange, authorList, ndlId |

### エラーコード

- `ITEM_NOT_FOUND`（404）→ エラートーストを表示し一覧へリダイレクト

### UpdateItemStatusRequest

```ts
interface UpdateItemStatusRequest {
  status: ItemStatus;  // 'not_started' | 'in_progress' | 'completed'
  consumedDate?: string;
}
```

参照元: `docs/tasks/frontend-collection-ui/TASK-0017.md`, `frontend/src/types/index.ts`

## 5. テスト関連情報

### テストフレームワーク設定

- 設定ファイル: `frontend/vitest.config.ts`
- 環境: jsdom
- globals: true
- setupFiles: `frontend/src/test/setup.ts`（`@testing-library/jest-dom/vitest` をimport）
- エイリアス: `@` → `frontend/src`
- excludes: `node_modules/**`, `tests/e2e/**`

### テストファイル配置パターン

- ページテスト: `frontend/src/pages/*.test.tsx`（例: `HomePage.test.tsx`）
- コンポーネントテスト: `frontend/src/components/common/*.test.tsx`
- フックテスト: `frontend/src/hooks/*.test.ts`

### テストのモックパターン

```ts
// APIフックのモック
vi.mock('@/api/items', () => ({
  useItemQuery: vi.fn(),
  useDeleteItemMutation: vi.fn(),
  useUpdateItemStatusMutation: vi.fn(),
}))

// react-router-domのuseNavigate/useParamsモック
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual<typeof import('react-router-dom')>('react-router-dom')
  return {
    ...actual,
    useNavigate: () => mockNavigate,
    useParams: () => ({ id: 'test-id' }),
  }
})
```

### テストラッパー

```tsx
function renderPage(initialEntries = ['/items/test-id']) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter initialEntries={initialEntries}>
        <ItemDetailPage />
      </MemoryRouter>
    </QueryClientProvider>
  )
}
```

参照元: `frontend/vitest.config.ts`, `frontend/src/test/setup.ts`, `frontend/src/pages/HomePage.test.tsx`

## 6. 注意事項

- `ApiClientError` には `code` プロパティがある（`ITEM_NOT_FOUND` 判別に使用）
- `useItemQuery` は `enabled: !!id` でidが空の場合スキップ
- 削除成功後は `navigate(-1)` または `/` へ遷移（一覧画面へ）
- TASK-0018〜0020のサブセクションはプレースホルダとして空のセクション枠のみ実装
- ルーティングの前提: `/items/:id` パス（`useParams<{ id: string }>()`）

参照元: `frontend/src/api/items.ts`, `docs/tasks/frontend-collection-ui/TASK-0017.md`
