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
};

export function LiteratureRow({ title, byline, doi, rating, tags, aside }: LiteratureRowProps) {
  return (
    <div className="lit-row">
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

export function LiteratureList({ items }: { items: LiteratureRowProps[] }) {
  return (
    <div className="lit-list">
      {items.map((item) => (
        <LiteratureRow key={item.id} {...item} />
      ))}
    </div>
  );
}
