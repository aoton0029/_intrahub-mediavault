export interface ErrorStateProps {
  onRetry: () => void
}

/**
 * TASK-0013: 一覧取得エラー時の表示（EDGE-001）。
 */
export function ErrorState({ onRetry }: ErrorStateProps) {
  return (
    <p role="status">
      一覧の取得に失敗しました。再試行してください。
      <button type="button" onClick={onRetry}>
        再試行
      </button>
    </p>
  )
}
