import type { CSSProperties } from "react";
import { MediaCard, type MediaCardProps } from "./MediaCard";
import { cn } from "@/lib/cn";

export function MediaGrid({
  items,
  density = "default",
  thumbnailOrientation,
  columns,
}: {
  items: MediaCardProps[];
  density?: "default" | "compact" | "horizontal";
  thumbnailOrientation?: "vertical" | "horizontal";
  /** Fixed number of columns; omit for auto-fill behavior. */
  columns?: number;
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
          key={`${item.badge}-${item.title}`}
          {...item}
          variant={item.variant ?? (density === "horizontal" ? "horizontal" : density === "compact" ? "compact" : "default")}
          thumbnailOrientation={item.thumbnailOrientation ?? thumbnailOrientation}
        />
      ))}
    </div>
  );
}
