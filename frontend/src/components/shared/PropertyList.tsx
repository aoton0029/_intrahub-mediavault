import type { ReactNode } from "react";
import { cn } from "@/lib/cn";

export type PropertyItem = { key: string; label: string; value: ReactNode; muted?: boolean };

export function PropertyList({ items }: { items: PropertyItem[] }) {
  return (
    <div className="prop-group">
      {items.map((item) => (
        <div key={item.key} className="prop-row">
          <span className="key">{item.label}</span>
          <span className={cn("val", item.muted && "muted")}>{item.value}</span>
        </div>
      ))}
    </div>
  );
}
