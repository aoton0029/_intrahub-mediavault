export function LoadMoreSentinel({ loading = true, text = "読み込み中..." }: { loading?: boolean; text?: string }) {
  return (
    <div className="load-more-sentinel">
      {loading ? <span className="spinner" aria-hidden="true" /> : null}
      <span>{text}</span>
    </div>
  );
}
