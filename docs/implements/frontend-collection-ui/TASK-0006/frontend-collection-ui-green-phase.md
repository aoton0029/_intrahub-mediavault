# TASK-0006: 共通UIコンポーネント実装 - Greenフェーズ記録

**機能名**: 共通UIコンポーネント（MediaCard / MediaTypeBadge / FilterBar / EmptyState / ConfirmDialog）
**タスクID**: TASK-0006
**要件名**: frontend-collection-ui
**フェーズ**: Green（最小実装でテストを通す）
**作成日**: 2026-06-30

---

## 1. 実装方針

- shadcn/ui CLI（`npx shadcn@latest add badge dialog`）によるコンポーネント追加は実行せず、テスト契約（data-testid・role・テキスト内容）を満たす最小限の独自実装を手書きした。Button のみ既存の `frontend/src/components/ui/button.tsx` を再利用。
- 各コンポーネントは表示専用の Pure Component として実装し、モック・スタブ・インメモリーストレージは一切含まない。
- 信頼性レベル（🔵🟡🔴）はコンポーネント単位で red-phase / requirements の記載に準拠。

---

## 2. 実装ファイル一覧

| コンポーネント | ファイル | 行数 |
| --- | --- | --- |
| MediaTypeBadge | `frontend/src/components/common/MediaTypeBadge.tsx` | 45 |
| MediaCard | `frontend/src/components/common/MediaCard.tsx` | 50 |
| FilterBar | `frontend/src/components/common/FilterBar.tsx` | 19 |
| EmptyState | `frontend/src/components/common/EmptyState.tsx` | 31 |
| ConfirmDialog | `frontend/src/components/common/ConfirmDialog.tsx` | 58 |

合計203行。800行制限内のため分割不要。

### 2.1 MediaTypeBadge.tsx

```typescript
import { cn } from '@/lib/utils'
import { getMediaTypeAccentClass } from '@/lib/media-type-accent'
import type { MediaType } from '@/types'

interface MediaTypeBadgeProps {
  mediaType: MediaType
}

const MEDIA_TYPE_LABEL: Record<MediaType, string> = {
  anime: 'アニメ',
  movie: '映画',
  drama: 'ドラマ',
  manga: '漫画',
  novel: '小説',
  game: 'ゲーム',
  academic_book: '専門書',
  paper: '論文',
}

export function MediaTypeBadge({ mediaType }: MediaTypeBadgeProps) {
  const accentClass = getMediaTypeAccentClass(mediaType) ?? ''
  const label = MEDIA_TYPE_LABEL[mediaType] ?? String(mediaType)

  return (
    <span
      data-slot="badge"
      data-testid="media-type-badge"
      className={cn(
        'inline-flex items-center rounded-md border px-2 py-0.5 text-xs font-medium',
        accentClass
      )}
    >
      {label}
    </span>
  )
}
```

### 2.2 MediaCard.tsx

```typescript
import { cn } from '@/lib/utils'
import { MediaTypeBadge } from './MediaTypeBadge'
import type { Item } from '@/types'

interface MediaCardProps {
  item: Item
  onClick?: (item: Item) => void
}

export function MediaCard({ item, onClick }: MediaCardProps) {
  const handleClick = () => {
    onClick?.(item)
  }

  return (
    <div
      data-testid="media-card"
      onClick={handleClick}
      className={cn(
        'flex cursor-pointer flex-col overflow-hidden rounded-lg border border-border bg-background'
      )}
    >
      <img
        src={item.coverImageUrl ?? ''}
        alt={item.title}
        className="aspect-[2/3] w-full object-cover"
      />

      <div className="flex flex-col gap-1 p-2">
        <span className="text-sm font-medium">{item.title}</span>

        <MediaTypeBadge mediaType={item.mediaType} />

        <span data-testid="media-card-favorite" data-favorite={String(item.isFavorite)} />

        <span data-testid="media-card-status" data-status={item.status} />
      </div>
    </div>
  )
}
```

### 2.3 FilterBar.tsx

```typescript
import type { ReactNode } from 'react'

interface FilterBarProps {
  children?: ReactNode
}

export function FilterBar({ children }: FilterBarProps) {
  return (
    <div data-testid="filter-bar" className="flex flex-wrap items-center gap-2">
      {children}
    </div>
  )
}
```

### 2.4 EmptyState.tsx

```typescript
import { Button } from '@/components/ui/button'

interface EmptyStateProps {
  message: string
  actionLabel?: string
  onAction?: () => void
}

export function EmptyState({ message, actionLabel, onAction }: EmptyStateProps) {
  return (
    <div
      data-testid="empty-state"
      className="flex flex-col items-center justify-center gap-3 py-12 text-center"
    >
      <p className="text-sm text-muted-foreground">{message}</p>

      {actionLabel ? (
        <Button type="button" onClick={() => onAction?.()}>
          {actionLabel}
        </Button>
      ) : null}
    </div>
  )
}
```

### 2.5 ConfirmDialog.tsx

```typescript
import { Button } from '@/components/ui/button'

interface ConfirmDialogProps {
  open: boolean
  title: string
  description?: string
  onConfirm: () => void
  onCancel: () => void
  confirmLabel?: string
  cancelLabel?: string
}

export function ConfirmDialog({
  open,
  title,
  description,
  onConfirm,
  onCancel,
  confirmLabel,
  cancelLabel,
}: ConfirmDialogProps) {
  if (!open) {
    return null
  }

  return (
    <div
      data-testid="confirm-dialog"
      role="dialog"
      aria-modal="true"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
    >
      <div className="flex min-w-[280px] flex-col gap-3 rounded-lg bg-background p-4">
        <h2 className="text-base font-semibold">{title}</h2>

        {description ? <p className="text-sm text-muted-foreground">{description}</p> : null}

        <div className="flex justify-end gap-2">
          <Button type="button" variant="outline" onClick={onCancel}>
            {cancelLabel ?? 'キャンセル'}
          </Button>
          <Button type="button" onClick={onConfirm}>
            {confirmLabel ?? 'OK'}
          </Button>
        </div>
      </div>
    </div>
  )
}
```

---

## 3. テスト実行結果

実行コマンド:
```bash
yarn test -- src/components/common/MediaCard.test.tsx src/components/common/MediaTypeBadge.test.tsx src/components/common/FilterBar.test.tsx src/components/common/EmptyState.test.tsx src/components/common/ConfirmDialog.test.tsx
```

結果:
```
Test Files  5 passed (5)
     Tests  44 passed (44)
```

`yarn lint` も実行し、エラー・警告なし。

---

## 4. 品質判定

```
✅ 高品質:
- テスト結果: 全44件成功（5ファイルすべてパス）
- 実装品質: シンプルかつ動作する（Pure Component、状態管理なし）
- リファクタ箇所: 明確に特定可能（shadcn/ui Badge/Dialog CLI導入、画像プレースホルダ改善）
- 機能的問題: なし
- コンパイルエラー: なし（lint クリーン）
- ファイルサイズ: 合計203行、800行制限内
- モック使用: 実装コードにモック・スタブなし
```

---

## 5. 課題・改善点（Refactorフェーズで対応）

- ConfirmDialog: shadcn/ui の Dialog（Radix UI ベース）未導入。フォーカストラップ・Escキー・ポータル化等のアクセシビリティ機能が不足。Radix Dialog Primitive への置き換えを検討。
- MediaTypeBadge: shadcn/ui の Badge 未導入。cva ベースの variant 管理への統一を検討。
- MediaCard: `coverImageUrl` 未設定時の `src=""` の扱い改善余地あり。
- EmptyState: アイコン表示等のビジュアル強化はテスト要件外のため未実装。

---

## 6. 次のステップ

次のお勧めステップ: `/tsumiki:tdd-refactor frontend-collection-ui TASK-0006` でRefactorフェーズ（品質改善）を開始します。
