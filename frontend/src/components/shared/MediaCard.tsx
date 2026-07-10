import type { ReactNode } from "react";
import { FavoriteToggle } from "./FavoriteToggle";
import { RatingStarsMini } from "./RatingStars";
import { cn } from "@/lib/cn";

export type MediaCardProps = {
  title: string;
  badge: string;
  meta?: ReactNode;
  originalTitle?: string;
  rating?: number;
  favorite?: boolean;
  onFavoriteChange?: (value: boolean) => void;
  variant?: "default" | "compact" | "search-result";
  imported?: boolean;
  actionLabel?: string;
  onAction?: () => void;
};

export function MediaCard({
  title,
  badge,
  meta,
  originalTitle,
  rating,
  favorite = false,
  onFavoriteChange,
  variant = "default",
  imported = false,
  actionLabel = "取り込む",
  onAction,
}: MediaCardProps) {
  return (
    <article className={cn("media-card", variant === "compact" && "is-compact", variant === "search-result" && "search-result is-compact")}>
      <div className="cover">
        <span className="badge">{badge}</span>
        {variant !== "search-result" && onFavoriteChange ? <FavoriteToggle value={favorite} onChange={onFavoriteChange} label="" /> : null}
      </div>
      <div className="body">
        <p className="title">{title}</p>
        {originalTitle ? <div className="doc-original" style={{ marginBottom: 6, fontSize: 11 }}>{originalTitle}</div> : null}
        {meta ? <div className="meta">{meta}</div> : null}
        {typeof rating === "number" && variant !== "search-result" ? <RatingStarsMini value={rating} /> : null}
        {variant === "search-result" ? (
          <button type="button" className={cn("btn btn-sm", imported ? "" : "btn-accent")} disabled={imported} onClick={onAction}>
            {imported ? "取り込み済み" : actionLabel}
          </button>
        ) : null}
      </div>
    </article>
  );
}
