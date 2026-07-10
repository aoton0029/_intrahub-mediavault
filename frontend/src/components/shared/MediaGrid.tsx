import { MediaCard, type MediaCardProps } from "./MediaCard";
import { cn } from "@/lib/cn";

export function MediaGrid({ items, density = "default" }: { items: MediaCardProps[]; density?: "default" | "compact" }) {
  return (
    <div className={cn("card-grid", density === "compact" && "is-compact")}>
      {items.map((item) => (
        <MediaCard key={`${item.badge}-${item.title}`} {...item} variant={item.variant ?? (density === "compact" ? "compact" : "default")} />
      ))}
    </div>
  );
}
