import type { ItemStatus } from '@/features/items/types'
import { cn } from '@/lib/utils'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'

/**
 * ItemStatus -> status-dot クラス名・表示ラベル
 * 🔵 信頼性: TASK-0010要件（未着手/視聴中/視聴済）より
 */
const STATUS_DOT_CLASS: Record<ItemStatus, string> = {
  not_started: 'none',
  in_progress: 'progress',
  completed: 'done',
}

const STATUS_LABEL: Record<ItemStatus, string> = {
  not_started: '未着手',
  in_progress: '視聴中',
  completed: '視聴済',
}

const STATUS_OPTIONS: ItemStatus[] = ['not_started', 'in_progress', 'completed']

export interface StatusDropdownProps {
  /** 現在のstatus値 */
  status: ItemStatus
  /** status選択時に呼び出されるコールバック。API呼び出し自体は呼び出し元の責務とする */
  onStatusChange: (newStatus: ItemStatus) => void
  /** status更新失敗時などにトリガーへエラー視覚状態を適用する */
  isError?: boolean
  /** 更新中は半透明化しトリガーを無効化する */
  isPending?: boolean
}

export function StatusDropdown({
  status,
  onStatusChange,
  isError = false,
  isPending = false,
}: StatusDropdownProps) {
  const label = STATUS_LABEL[status]

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button
          type="button"
          disabled={isPending}
          className={cn(
            'status-dropdown-trigger inline-flex min-h-11 items-center gap-1.5 rounded-md px-2 py-1 text-sm',
            isError && 'status-dropdown-trigger--error text-destructive',
            isPending && 'opacity-50',
          )}
          aria-label={`ステータス: ${label}。変更する`}
          aria-invalid={isError || undefined}
        >
          <span className={`status-dot ${STATUS_DOT_CLASS[status]}`} aria-hidden="true" />
          {label}
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent>
        {STATUS_OPTIONS.map((option) => (
          <DropdownMenuItem key={option} onSelect={() => onStatusChange(option)}>
            <span className={`status-dot ${STATUS_DOT_CLASS[option]}`} aria-hidden="true" />
            {STATUS_LABEL[option]}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
