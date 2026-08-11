import type { ReactNode } from "react";
import { RatingStarsMini } from "./RatingStars";

export type LiteratureRowProps = {
  id: string;
  title: string;
  byline: string;
  doi?: string;
  rating?: number;
  tags?: ReactNode;
  aside?: ReactNode;
  onClick?: (id: string) => void;
};

export function LiteratureRow({ id, title, byline, doi, rating, tags, aside, onClick }: LiteratureRowProps) {
  return (
    <div className="lit-row" onClick={onClick ? () => onClick(id) : undefined} role={onClick ? "button" : undefined} tabIndex={onClick ? 0 : undefined}>
      <div className="thumb" />
      <div className="info">
        <p className="title">{title}</p>
        <div className="byline">
          {byline}
          {typeof rating === "number" ? <RatingStarsMini value={rating} /> : null}
        </div>
        {doi ? <div className="doi">{doi}</div> : null}
        {tags}
      </div>
      {aside ? <div className="aside">{aside}</div> : null}
    </div>
  );
}

export function LiteratureList({ items, onRowClick }: { items: LiteratureRowProps[]; onRowClick?: (id: string) => void }) {
  return (
    <div className="lit-list">
      {items.map((item) => (
        <LiteratureRow key={item.id} {...item} onClick={item.onClick ?? onRowClick} />
      ))}
    </div>
  );
}
