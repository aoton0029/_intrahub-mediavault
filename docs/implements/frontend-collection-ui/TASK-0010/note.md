# TASK-0010 コンテキストノート: FilterBarコンポーネント詳細実装

## 1. 技術スタック

- **フレームワーク**: React 18.3+ / TypeScript 5.7+ / Vite 6
- **スタイリング**: Tailwind CSS v4 + shadcn/ui（button/badge/dialogがインストール済み）
- **UIプリミティブ**: `radix-ui` v1.6.0（unified package）
- **テストフレームワーク**: Vitest + @testing-library/react（jsdom環境）
- **パッケージマネージャ**: yarn（frontendディレクトリ内）
- **エイリアス**: `@/` → `frontend/src/`
- 参照元: `frontend/vitest.config.ts`, `frontend/CLAUDE.md`, `docs/spec/frontend-collection-ui/note.md`

## 2. 開発ルール

- テストファイルは実装ファイルと同ディレクトリに `*.test.tsx` として配置する
- テストケースIDは `TC-{略称}-{カテゴリ}-{連番}` 形式（例: `TC-FB-N-01`）
- テスト記述パターン: `describe` + `it` の二段構成
- Reactコンポーネントテストには `render` + `screen` + `userEvent` を使用
- **shadcn/ui SelectはjsdomでRadixのPortalが正常動作しないため、テスト可能性を考慮して native HTML `<select>` or Radixを使わないカスタムSelect実装にする**
- controlledコンポーネントパターン: FilterBarは`filters`と`onChange`をpropsで受け取り、内部stateを持たない
- 参照元: `frontend/src/hooks/useSearchParamsFilter.test.ts`, `frontend/src/components/common/MediaCard.test.tsx`

## 3. 関連実装

### 汎用フック（TASK-0008で実装済み）
- **ファイル**: `frontend/src/hooks/useSearchParamsFilter.ts`
- **役割**: URLクエリパラメータと`ItemListFilters`を双方向同期
- **呼び出し元（HomePage等）がフックを保持してFilterBarに`filters`と`onChange`をpropsで渡す設計**

### 共通コンポーネント（TASK-0006で実装済み）
- `frontend/src/components/common/MediaCard.tsx` — カードレイアウトパターンの参考
- `frontend/src/components/common/MediaTypeBadge.tsx` — MediaType→ラベル変換の参考
- `frontend/src/components/ui/button.tsx` — クリアボタンに使用

### 型定義（TASK-0004で配置済み）
- **ファイル**: `frontend/src/types/index.ts`
- **使用する主な型**: `ItemListFilters`, `MediaType`, `ItemStatus`, `Tag`, `Category`
- **MediaType**: `'anime' | 'movie' | 'drama' | 'manga' | 'novel' | 'game' | 'academic_book' | 'paper'`
- **ItemStatus**: `'not_started' | 'in_progress' | 'completed'`

### shadcn/uiインストール済みコンポーネント
- `frontend/src/components/ui/button.tsx`
- `frontend/src/components/ui/badge.tsx`
- `frontend/src/components/ui/dialog.tsx`
- **未インストール**: `select`, `switch`, `checkbox`（必要に応じてnative HTMLで代替）

## 4. 設計文書

- **タスク定義**: `docs/tasks/frontend-collection-ui/TASK-0010.md`
- **型定義設計**: `docs/design/frontend-collection-ui/interfaces.ts`
- **アーキテクチャ**: `docs/design/frontend-collection-ui/architecture.md`
- **データフロー**: `docs/design/frontend-collection-ui/dataflow.md`

### FilterBarのprops設計

```ts
interface FilterBarProps {
  filters: ItemListFilters
  onChange: (filters: ItemListFilters) => void
  tagOptions: Tag[]
  categoryOptions: Category[]
  mediaTypeOptions?: MediaType[]   // 省略時は全MediaTypeを表示
  disabled?: boolean               // ローディング中は全フィルタをdisabledに
}
```

### フィルタ種類と対応するUIコントロール

| フィルタ | UIコントロール | propsキー |
|---|---|---|
| media_type | `<select>` | `filters.mediaType` |
| tag | `<select>` | `filters.tagId` |
| category | `<select>` | `filters.categoryId` |
| お気に入り | `<input type="checkbox">` | `filters.isFavorite` |
| status | `<select>` | `filters.status` |

### クリアボタン仕様
- `onChange`に`{}`（空のItemListFilters）を渡す
- `page`はリセット対象に含める

## 5. テスト関連情報

- **テスト設定**: `frontend/vitest.config.ts`（jsdom環境, globals: true）
- **setupファイル**: `frontend/src/test/setup.ts`（`@testing-library/jest-dom/vitest` をインポート）
- **テストファイル配置先**: `frontend/src/components/common/FilterBar.test.tsx`（既存を上書き）
- **既存テスト（Phase 1のもの）**: 既存3テストケースはcontainerパターン用のため削除してTASK-0010仕様に差し替え

### テストパターン

```ts
import { describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { FilterBar } from './FilterBar'
import type { ItemListFilters, Tag, Category, MediaType } from '@/types'
```

### userEvent注意点
- `userEvent.selectOptions(element, value)` でselect操作
- `userEvent.click(element)` でcheckbox/button操作
- `await userEvent.setup()...` を使う（v14以降のAPI）

### jsdom制約
- Radix UI の `Select`/`Switch` は Portal使用のためjsdomで動作が不安定
- **ネイティブ `<select>` と `<input type="checkbox">` を使ってテスト可能にする**

## 6. 注意事項

### 実装ファイルパス
- **実装先**: `frontend/src/components/common/FilterBar.tsx`（既存ファイルを差し替え）
- **テスト先**: `frontend/src/components/common/FilterBar.test.tsx`（既存ファイルを差し替え）

### Phase 1との互換性
- Phase 1では `FilterBar` が `children` propsのみの最小コンテナとして実装されていた
- Phase 2（本タスク）でフル実装に差し替える
- `children` propsは削除し、`filters`/`onChange`/`tagOptions`/`categoryOptions`/`mediaTypeOptions?`/`disabled?` の設計に変更する
- 後続タスク（TASK-0011等）でFilterBarを使う際はこの新しいpropsを使用する

### MediaType全選択肢のラベル対応
```ts
const MEDIA_TYPE_LABELS: Record<MediaType, string> = {
  anime: 'アニメ',
  movie: '映画',
  drama: 'ドラマ',
  manga: '漫画',
  novel: '小説',
  game: 'ゲーム',
  academic_book: '学術書',
  paper: '論文',
}
```

### ItemStatus全選択肢のラベル対応
```ts
const STATUS_LABELS: Record<ItemStatus, string> = {
  not_started: '未開始',
  in_progress: '進行中',
  completed: '完了',
}
```
