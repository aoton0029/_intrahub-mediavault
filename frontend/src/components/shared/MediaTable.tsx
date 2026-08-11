import { FiMoreHorizontal } from "react-icons/fi";
import { cn } from "@/lib/cn";
import { RatingStarsMini } from "./RatingStars";

export type MediaTableRow = {
  id: string;
  title: string;
  badge: string;
  imageUrl?: string | null;
  status: "not_started" | "in_progress" | "completed";
  statusLabel: string;
  rating?: number;
  updatedLabel: string;
};

export function MediaTable({
  rows,
  onRowClick,
  onOpenMenu,
}: {
  rows: MediaTableRow[];
  onRowClick: (id: string) => void;
  onOpenMenu: (id: string, anchor: HTMLElement) => void;
}) {
  return (
    <div className="table-view">
      <table>
        <thead>
          <tr>
            <th className="col-cover" />
            <th>タイトル</th>
            <th>ステータス</th>
            <th>評価</th>
            <th>更新日時</th>
            <th className="col-actions" />
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={row.id} onClick={() => onRowClick(row.id)}>
              <td>
                <div className="table-row-cover">{row.imageUrl ? <img alt="" loading="lazy" src={row.imageUrl} /> : null}</div>
              </td>
              <td>
                <div className="table-row-title">
                  {row.title}
                  <span className="table-row-badge">{row.badge}</span>
                </div>
              </td>
              <td>
                <span className={cn("table-row-status", row.status)}>{row.statusLabel}</span>
              </td>
              <td>{typeof row.rating === "number" ? <RatingStarsMini value={row.rating} /> : <span className="meta">未評価</span>}</td>
              <td>
                <span className="table-row-updated">{row.updatedLabel}</span>
              </td>
              <td>
                <button
                  type="button"
                  className="table-row-menu-btn"
                  title="メニュー"
                  aria-label="メニュー"
                  onClick={(event) => {
                    event.preventDefault();
                    event.stopPropagation();
                    onOpenMenu(row.id, event.currentTarget);
                  }}
                >
                  <FiMoreHorizontal className="icon" />
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
