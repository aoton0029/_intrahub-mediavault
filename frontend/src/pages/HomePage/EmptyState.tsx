/**
 * TASK-0013: 取得結果0件時の空状態表示（EDGE-101）。
 */
export function EmptyState() {
  return (
    <p role="status">
      該当するアイテムがありません
      <a href="/search-add" className="btn btn-accent">
        ＋ 作品を追加
      </a>
    </p>
  )
}
