import type { ReactNode } from "react";
import { RatingStarsMini } from "./RatingStars";

export type LiteratureRowProps = {
  id: string;
  title: string;
  authors?: string;
  year?: string | number;
  journal?: string;
  doi?: string;
  rating?: number;
  tags?: ReactNode;
  aside?: ReactNode;
  onClick?: (id: string) => void;
};

export function LiteratureList({ items, onRowClick }: { items: LiteratureRowProps[]; onRowClick?: (id: string) => void }) {
  return (
    <div className="table-view lit-table-view">
      <table>
        <thead>
          <tr>
            <th className="col-lit-title">タイトル</th>
            <th className="col-lit-authors">著者</th>
            <th className="col-lit-year">年</th>
            <th className="col-lit-journal">掲載誌 / DOI</th>
            <th className="col-lit-tags">タグ</th>
            <th className="col-lit-rating">評価</th>
            <th className="col-actions" />
          </tr>
        </thead>
        <tbody>
          {items.map(({ id, title, authors, year, journal, doi, rating, tags, aside, onClick: rowOnClick }) => {
            const onClick = rowOnClick ?? onRowClick;
            return (
              <tr key={id} onClick={onClick ? () => onClick(id) : undefined} className={onClick ? "is-clickable" : undefined}>
                <td>
                  <p className="lit-title" title={title}>
                    {title}
                  </p>
                </td>
                <td>
                  <span className="lit-authors" title={authors}>
                    {authors || <span className="meta">—</span>}
                  </span>
                </td>
                <td>
                  <span className="lit-year">{year ?? <span className="meta">—</span>}</span>
                </td>
                <td>
                  <div className="lit-journal-cell">
                    {journal ? <span className="lit-journal">{journal}</span> : null}
                    {doi ? <span className="doi">{doi}</span> : null}
                    {!journal && !doi ? <span className="meta">—</span> : null}
                  </div>
                </td>
                <td>{tags ?? <span className="meta">—</span>}</td>
                <td>{typeof rating === "number" ? <RatingStarsMini value={rating} /> : <span className="meta">未評価</span>}</td>
                <td>
                  <div className="lit-aside">{aside}</div>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
