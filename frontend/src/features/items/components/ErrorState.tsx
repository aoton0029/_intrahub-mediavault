export function ErrorState({ onRetry }: { onRetry: () => void }) {
  return (
    <div className="py-16 text-center text-text-faint">
      <p className="mb-3 text-text-muted">一覧の取得に失敗しました。再試行してください。</p>
      <button
        type="button"
        onClick={onRetry}
        className="rounded-app border border-border bg-bg-surface px-3.5 py-1.5 text-sm text-text-primary hover:bg-bg-surface-hover"
      >
        再試行
      </button>
    </div>
  );
}
