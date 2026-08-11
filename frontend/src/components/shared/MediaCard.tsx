import type { MouseEvent, ReactNode } from "react";
import { Link } from "react-router-dom";
import { FiEye, FiExternalLink, FiHeart, FiMoreHorizontal, FiPlus } from "react-icons/fi";
import { FavoriteToggle } from "./FavoriteToggle";
import { RatingStarsMini } from "./RatingStars";
import { cn } from "@/lib/cn";

export type MediaCardProps = {
  id?: string;
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
  thumbnailOrientation?: "vertical" | "horizontal";
  /** Applies only to non-search-result variants; search-result/picker cards always show the title. */
  showTitle?: boolean;
  /** Applies only to non-search-result variants. */
  showRating?: boolean;
  /** Renders the hover overlay (quick view / add-to-collection / open-detail / kebab menu) when provided. */
  onQuickView?: () => void;
  onAddToCollection?: () => void;
  onOpenMenu?: (anchor: HTMLElement) => void;
};

export function MediaCard({
  id,
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
  thumbnailOrientation = "horizontal",
  showTitle = true,
  showRating = true,
  onQuickView,
  onAddToCollection,
  onOpenMenu,
}: MediaCardProps) {
  const isSearchResult = variant === "search-result" || variant === "picker";
  const hasOverlay = !isSearchResult && Boolean(onQuickView || onOpenMenu || onAddToCollection);
  const hasBodyContent =
    isSearchResult ||
    showTitle ||
    Boolean(originalTitle) ||
    Boolean(meta) ||
    (typeof rating === "number" && showRating);
  const className = cn(
    "media-card",
    variant === "compact" && "is-compact",
    variant === "search-result" && "search-result is-compact",
    variant === "picker" && "search-result is-dense",
    variant === "horizontal" && "is-horizontal",
    thumbnailOrientation === "vertical" && "cover-portrait",
    !hasBodyContent && "no-body",
  );

  function stop(event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
  }

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
        {hasOverlay ? (
          <div className="card-overlay">
            <div className="card-overlay-top">
              {onQuickView ? (
                <button type="button" title="クイックビュー" aria-label="クイックビュー" onClick={(event) => { stop(event); onQuickView(); }}>
                  <FiEye className="icon" />
                </button>
              ) : null}
              {onAddToCollection ? (
                <button type="button" title="コレクションに追加" aria-label="コレクションに追加" onClick={(event) => { stop(event); onAddToCollection(); }}>
                  <FiPlus className="icon" />
                </button>
              ) : null}
            </div>
            <div className="card-overlay-center">
              {href ? (
                <Link to={href} title="詳細ページを開く" aria-label="詳細ページを開く" onClick={(event) => event.stopPropagation()}>
                  <FiExternalLink className="icon" />
                </Link>
              ) : null}
            </div>
            <div className="card-overlay-bottom">
              <p className="card-title">{title}</p>
              {onOpenMenu ? (
                <button
                  type="button"
                  className="kebab-btn"
                  title="メニュー"
                  aria-label="メニュー"
                  onClick={(event) => { stop(event); onOpenMenu(event.currentTarget); }}
                >
                  <FiMoreHorizontal className="icon" />
                </button>
              ) : null}
            </div>
          </div>
        ) : null}
      </div>
      {hasBodyContent ? (
        <div className="body">
          {isSearchResult || showTitle ? <p className="title">{isSearchResult && meta ? `${title}(${meta})` : title}</p> : null}
          {originalTitle ? <div className="doc-original" style={{ marginBottom: 6, fontSize: 11 }}>{originalTitle}</div> : null}
          {!isSearchResult && meta ? <div className="meta">{meta}</div> : null}
          {typeof rating === "number" && !isSearchResult && showRating ? <RatingStarsMini value={rating} /> : null}
          {isSearchResult ? (
            <button type="button" className={cn("btn btn-sm", imported ? "" : "btn-accent")} disabled={imported} onClick={onAction}>
              {imported ? importedLabel : actionLabel}
            </button>
          ) : null}
        </div>
      ) : null}
    </>
  );

  if (href && !isSearchResult && !hasOverlay) {
    return (
      <Link className={className} style={{ color: "inherit", textDecoration: "none" }} to={href}>
        {body}
      </Link>
    );
  }

  return (
    <article className={className} data-id={id}>
      {body}
    </article>
  );
}
