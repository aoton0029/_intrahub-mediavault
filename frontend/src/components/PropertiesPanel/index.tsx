import { StatusDropdown } from '@/components/StatusDropdown'
import type { ItemDetail, ItemStatus } from '@/features/items/types'

export interface PropertiesPanelProps {
  item: ItemDetail
  onStatusChange: (next: { status: ItemStatus; consumedDate?: string | null }) => void
  isStatusUpdating?: boolean
}

function ratingText(rating: number): string {
  const filled = Math.round(rating)
  const stars = '★'.repeat(filled) + '☆'.repeat(5 - filled)
  return `${stars} (${rating.toFixed(1)})`
}

/**
 * 詳細画面プロパティパネル（.properties）
 *
 * TASK-0015: 02_item_detail.htmlの `.prop-group`/`.prop-row`/`.key`/`.val`/`.muted`
 * 構造に準拠する。status行はStatusDropdown(TASK-0010)経由で変更可能にし、
 * onStatusChangeコールバックのみを発火する（API呼び出しはTASK-0021の責務）。
 */
export function PropertiesPanel({ item, onStatusChange, isStatusUpdating = false }: PropertiesPanelProps) {
  const handleStatusChange = (newStatus: ItemStatus) => {
    onStatusChange({ status: newStatus, consumedDate: item.consumed_date ?? null })
  }

  return (
    <aside className="properties">
      <div className="prop-group">
        <div className="prop-row">
          <span className="key">media_type</span>
          <span className="val muted">{item.media_type}</span>
        </div>

        <div className="prop-row">
          <span className="key">status</span>
          <span className="val">
            <StatusDropdown
              status={item.status}
              onStatusChange={handleStatusChange}
              isPending={isStatusUpdating}
            />
          </span>
        </div>

        <div className="prop-row">
          <span className="key">consumed_date</span>
          <span className="val muted">{item.consumed_date ?? '—'}</span>
        </div>

        {item.rating != null && (
          <div className="prop-row">
            <span className="key">rating</span>
            <span className="val">{ratingText(item.rating)}</span>
          </div>
        )}

        {item.is_favorite && (
          <div className="prop-row">
            <span className="key">favorite</span>
            <span className="val">★ お気に入り</span>
          </div>
        )}

        <div className="prop-row">
          <span className="key">source</span>
          <span className="val muted">{item.source}</span>
        </div>

        <div className="prop-row">
          <span className="key">release_date</span>
          <span className="val muted">{item.release_date ?? '—'}</span>
        </div>

        {item.studio && (
          <div className="prop-row">
            <span className="key">studio</span>
            <span className="val">{item.studio}</span>
          </div>
        )}
      </div>
    </aside>
  )
}
