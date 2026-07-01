# TASK-0012 コンテキストノート: GeneralListPage実装

## 1. 技術スタック

- **フレームワーク**: React 18.3+ + TypeScript + Vite
- **ルーティング**: React Router v7 (`createBrowserRouter`, `RouterProvider`)
- **状態管理**: TanStack Query v5 (`useQuery`, `useMutation`, `QueryClient`)
- **UIライブラリ**: shadcn/ui + Tailwind CSS v4 + Radix UI
- **テスト**: Vitest (jsdom) + @testing-library/react
- **アーキテクチャ**: Feature-Sliced 寄りレイヤード構成

- 参照元: `docs/design/frontend-collection-ui/architecture.md`

## 2. 開発ルール

- **コンポーネント**: controlled component パターン（props/onChange によるステート管理）
- **FilterBar**: native HTML `<select>` + `<input type="checkbox">` を使用（Radix は jsdom で不安定）
- **テスト構造**: `describe` + `it` + JSDoc（テスト目的・内容・期待動作・データ準備）
- **テストケースID体系**: `TC-{略称}-{N/E/B}-{連番}` (N=正常系, E=異常系, B=境界値)
  - GeneralListPage: `TC-GL-{カテゴリ}-{連番}` を使用
- **ルーティングテスト**: `MemoryRouter` でラップ
- **Query テスト**: `QueryClient` + `QueryClientProvider` でラップ + `vi.mocked()`

- 参照元: `docs/implements/frontend-collection-ui/0010/note.md`, `frontend/src/components/common/FilterBar.tsx`

## 3. 関連実装

### HomePage.tsx (共有ロジックの参照元)
- パス: `frontend/src/pages/HomePage.tsx`
- `useItemsQuery(filters)` でアイテム一覧取得
- `useSearchParamsFilter()` でURLフィルタ同期
- FilterBar に `mediaTypeOptions` なし = 全8種表示
- `grid-cols-2 md:grid-cols-4 lg:grid-cols-6` のレスポンシブグリッド
- EmptyState + ページネーション + スケルトン + エラー再試行

### 既存共有コンポーネント
- `frontend/src/components/common/FilterBar.tsx` - mediaTypeOptions prop でフィルタ選択肢を制御
- `frontend/src/components/common/MediaCard.tsx` - アイテムカード表示
- `frontend/src/components/common/EmptyState.tsx` - 空状態表示
- `frontend/src/hooks/useSearchParamsFilter.ts` - URLSearchParams ⟷ ItemListFilters 同期
- `frontend/src/api/items.ts` - `useItemsQuery(filters)`, `fetchItems(filters)`

### 類似実装（AcademicListPage / PaperListPage）
- パス: `frontend/src/pages/AcademicListPage.tsx`, `frontend/src/pages/PaperListPage.tsx`
- HomePage と同じコンポーネントを使用し、mediaTypeOptions を絞り込む実装パターン

## 4. 設計文書

- `docs/spec/frontend-collection-ui/requirements.md` - REQ-004: メディアグループ別専用一覧画面
- `docs/design/frontend-collection-ui/architecture.md` - コンポーネント構成・ルーティング
- `docs/design/frontend-collection-ui/dataflow.md` - データフロー図
- `docs/tasks/frontend-collection-ui/TASK-0012.md` - タスク詳細
- `docs/tasks/frontend-collection-ui/TASK-0011.md` - 依存タスク（HomePage実装）

### ルーティング
- ファイル: `frontend/src/routes.tsx`
- `/collections/general` → `GeneralListPage` のルートは **既に定義済み**
- 現在 GeneralListPage はスタブ（`<div>GeneralListPage</div>`）のみ

### FilterBar の mediaTypeOptions 仕様
- Props: `mediaTypeOptions?: string[]`
- 未指定時: 全8種表示
- 指定時: 指定された種類のみ表示
- GeneralListPage では `['anime','movie','drama','manga','novel','game']` を渡す

## 5. テスト関連情報

- **テストフレームワーク**: Vitest + @testing-library/react + jsdom
- **設定ファイル**: `frontend/vitest.config.ts`
- **セットアップ**: `frontend/src/test/setup.ts` (`@testing-library/jest-dom/vitest`)
- **テストディレクトリ**: 実装ファイルと同階層（例: `pages/HomePage.test.tsx`）
- **既存テストの参考**: `frontend/src/pages/HomePage.test.tsx`

### テストデータ fixture パターン
```ts
function makeAnimeItem(overrides = {}): Item {
  return { id: '1', title: 'Test Anime', media_type: 'anime', status: 'watching', isFavorite: false, ...overrides }
}
```

### モックパターン
```ts
import { vi } from 'vitest'
import * as itemsApi from '@/api/items'
vi.mock('@/api/items')
vi.mocked(useItemsQuery).mockReturnValue({ data: { items: [...], pagination: {...} }, isLoading: false, isError: false })
```

### テスト実行
```bash
cd frontend && yarn test
cd frontend && yarn test:watch
```

- 参照元: `frontend/vitest.config.ts`, `frontend/src/test/setup.ts`, `frontend/src/pages/HomePage.test.tsx`

## 6. 注意事項

### API の media_type 制約
- バックエンド API (`GET /items`) の `media_type` パラメータは単一値のみ受け付ける可能性あり
- **設計方針**: `mediaType` フィルタ未指定時は全アイテム取得 → クライアント側で6種フィルタ不要（サーバー側が `academic_book`/`paper` を別扱いしない場合）
- または: `media_type` パラメータを送信しない → バックエンドが全種返す → UIで6種のオプションのみ表示
- **実装方針**: FilterBar の選択肢を6種に固定し、ユーザーが個別選択した場合のみ `media_type` パラメータを送信する
- 参照元: `docs/tasks/frontend-collection-ui/TASK-0012.md` § 3. 一般メディア6種の固定フィルタ適用

### GeneralListPage の現状
- `frontend/src/pages/GeneralListPage.tsx` は現在スタブ実装のみ
- ルート `/collections/general` は `routes.tsx` に定義済み（追加不要）
- HomePage の実装パターンをほぼそのまま流用し、`mediaTypeOptions` のみ差分として追加

### 型定義
- `ItemListFilters.mediaType` は `string | undefined`
- `useSearchParamsFilter()` が URL `media_type` クエリパラメータを `filters.mediaType` に変換

- 参照元: `frontend/src/types/index.ts`, `frontend/src/hooks/useSearchParamsFilter.ts`
