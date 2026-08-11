import type { CSSProperties } from "react";
import { MediaCard, type MediaCardProps } from "./MediaCard";
import { cn } from "@/lib/cn";

export function MediaGrid({
  items,
  density = "default",
  thumbnailOrientation,
  columns,
  showTitle,
  showRating,
  onQuickView,
  onAddToCollection,
  onOpenMenu,
}: {
  items: MediaCardProps[];
  density?: "default" | "compact" | "horizontal";
  thumbnailOrientation?: "vertical" | "horizontal";
  /** Fixed number of columns; omit for auto-fill behavior. */
  columns?: number;
  showTitle?: boolean;
  showRating?: boolean;
  onQuickView?: (item: MediaCardProps) => void;
  onAddToCollection?: (item: MediaCardProps) => void;
  onOpenMenu?: (item: MediaCardProps, anchor: HTMLElement) => void;
}) {
  return (
    <div
      className={cn(
        "card-grid",
        density === "compact" && "is-compact",
        density === "horizontal" && "is-horizontal",
        columns != null && "is-fixed-cols",
      )}
      style={columns != null ? ({ "--grid-cols": columns } as CSSProperties) : undefined}
    >
      {items.map((item) => (
        <MediaCard
          key={item.id ?? `${item.badge}-${item.title}`}
          {...item}
          variant={item.variant ?? (density === "horizontal" ? "horizontal" : density === "compact" ? "compact" : "default")}
          thumbnailOrientation={item.thumbnailOrientation ?? thumbnailOrientation}
          showTitle={item.showTitle ?? showTitle}
          showRating={item.showRating ?? showRating}
          onQuickView={onQuickView ? () => onQuickView(item) : undefined}
          onAddToCollection={onAddToCollection ? () => onAddToCollection(item) : undefined}
          onOpenMenu={onOpenMenu ? (anchor) => onOpenMenu(item, anchor) : undefined}
        />
      ))}
    </div>
  );
}
