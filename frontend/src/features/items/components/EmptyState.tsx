export function EmptyState() {
  return (
    <div role="status" className="py-16 text-center text-text-faint">
      <h3 className="mb-1.5 font-display font-semibold text-text-muted">
        該当するアイテムがありません
      </h3>
      <a href="/search-add" className="text-accent-strong">
        ＋ 作品を追加
      </a>
    </div>
  );
}
