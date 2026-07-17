import type { ReactNode } from "react";
import { Link } from "react-router-dom";
import { FiHeart } from "react-icons/fi";
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
  variant?: "default" | "compact" | "search-result" | "picker" | "horizontal";
  imported?: boolean;
  importedLabel?: string;
  actionLabel?: string;
  onAction?: () => void;
  href?: string;
  imageUrl?: string | null;
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
  importedLabel = "取り込み済み",
  actionLabel = "取り込む",
  onAction,
  href,
  imageUrl,
}: MediaCardProps) {
  const isSearchResult = variant === "search-result" || variant === "picker";
  const className = cn(
    "media-card",
    variant === "compact" && "is-compact",
    variant === "search-result" && "search-result is-compact",
    variant === "picker" && "search-result is-dense",
    variant === "horizontal" && "is-horizontal",
  );

  const body = (
    <>
      <div className="cover">
        {imageUrl ? <img src={imageUrl} alt="" loading="lazy" /> : null}
        <span className="badge">{badge}</span>
        {!isSearchResult && onFavoriteChange ? <FavoriteToggle value={favorite} onChange={onFavoriteChange} label="" /> : null}
        {!isSearchResult && !onFavoriteChange && favorite ? (
          <span className="cover-favorite" aria-label="お気に入り">
            <FiHeart className="icon" />
          </span>
        ) : null}
      </div>
      <div className="body">
        <p className="title">{isSearchResult && meta ? `${title}(${meta})` : title}</p>
        {originalTitle ? <div className="doc-original" style={{ marginBottom: 6, fontSize: 11 }}>{originalTitle}</div> : null}
        {!isSearchResult && meta ? <div className="meta">{meta}</div> : null}
        {typeof rating === "number" && !isSearchResult ? <RatingStarsMini value={rating} /> : null}
        {isSearchResult ? (
          <button type="button" className={cn("btn btn-sm", imported ? "" : "btn-accent")} disabled={imported} onClick={onAction}>
            {imported ? importedLabel : actionLabel}
          </button>
        ) : null}
      </div>
    </>
  );

  if (href && !isSearchResult) {
    return (
      <Link className={className} style={{ display: "block", color: "inherit", textDecoration: "none" }} to={href}>
        {body}
      </Link>
    );
  }

  return (
    <article className={className}>
      {body}
    </article>
  );
}
