import { Button } from '@/components/ui/button'

interface EmptyStateProps {
  message: string
  actionLabel?: string
  onAction?: () => void
}

/**
 * 【機能概要】: アイテム0件時の空状態表示。メッセージと任意のアクション導線を表示する
 * 【実装方針】: actionLabel が指定された場合のみボタンを描画する
 * 【テスト対応】: TC-ES-N-01〜03, TC-ES-E-01, TC-ES-B-01〜02
 * 🟡 信頼性レベル: requirements.md「2.4 EmptyState」, EDGE-101 より
 */
export function EmptyState({ message, actionLabel, onAction }: EmptyStateProps) {
  return (
    <div
      data-testid="empty-state"
      className="flex flex-col items-center justify-center gap-3 py-12 text-center"
    >
      <p className="text-sm text-muted-foreground">{message}</p>

      {/* 【アクションボタン】: actionLabel 指定時のみ描画する */}
      {actionLabel ? (
        <Button type="button" onClick={() => onAction?.()}>
          {actionLabel}
        </Button>
      ) : null}
    </div>
  )
}
