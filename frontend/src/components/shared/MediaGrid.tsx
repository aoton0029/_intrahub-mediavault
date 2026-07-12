import { MediaCard, type MediaCardProps } from "./MediaCard";
import { cn } from "@/lib/cn";

export function MediaGrid({ items, density = "default" }: { items: MediaCardProps[]; density?: "default" | "compact" | "horizontal" }) {
  return (
    <div className={cn("card-grid", density === "compact" && "is-compact", density === "horizontal" && "is-horizontal")}>
      {items.map((item) => (
        <MediaCard
          key={`${item.badge}-${item.title}`}
          {...item}
          variant={item.variant ?? (density === "horizontal" ? "horizontal" : density === "compact" ? "compact" : "default")}
        />
      ))}
    </div>
  );
}
