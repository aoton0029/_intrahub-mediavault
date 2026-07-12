import { useState, type ReactNode } from "react";
import { cn } from "@/lib/cn";

export type TabItem = { key: string; label: string; icon?: ReactNode; content: ReactNode };

export function Tabs({ items, defaultKey }: { items: TabItem[]; defaultKey?: string }) {
  const [activeKey, setActiveKey] = useState(defaultKey ?? items[0]?.key);

  if (!items.length) {
    return null;
  }

  return (
    <div className="resource-tabs">
      <div className="resource-tabbar">
        {items.map((item) => (
          <button key={item.key} type="button" className={cn(activeKey === item.key && "active")} onClick={() => setActiveKey(item.key)}>
            {item.icon}
            {item.label}
          </button>
        ))}
      </div>
      {items.map((item) => (
        <div key={item.key} className="resource-panel" hidden={activeKey !== item.key}>
          {item.content}
        </div>
      ))}
    </div>
  );
}
