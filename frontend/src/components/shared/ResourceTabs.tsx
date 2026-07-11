import { useState } from "react";
import { FiFilm, FiLink2, FiPaperclip, FiTrash2 } from "react-icons/fi";
import { cn } from "@/lib/cn";

export type ResourceTabKey = "links" | "files" | "trailers";

type ResourceEntry = { id: string; label: string; detail: string; onRemove?: (id: string) => void };

const labels: Record<ResourceTabKey, string> = { links: "リンク", files: "ファイル", trailers: "トレーラー" };
const icons = { links: FiLink2, files: FiPaperclip, trailers: FiFilm } satisfies Record<ResourceTabKey, typeof FiLink2>;

export function ResourceTabs({ tabs }: { tabs: Partial<Record<ResourceTabKey, ResourceEntry[]>> }) {
  const availableTabs = (Object.keys(tabs) as ResourceTabKey[]).filter((key) => tabs[key]?.length);
  const [activeTab, setActiveTab] = useState<ResourceTabKey>(availableTabs[0] ?? "links");

  return (
    <div className="resource-tabs">
      <div className="resource-tabbar">
        {availableTabs.map((tab) => {
          const Icon = icons[tab];
          return (
            <button key={tab} type="button" className={cn(activeTab === tab && "active")} onClick={() => setActiveTab(tab)}>
              <Icon className="icon" />
              {labels[tab]}
            </button>
          );
        })}
      </div>
      {availableTabs.map((tab) => (
        <div key={tab} className="resource-panel" hidden={activeTab !== tab}>
          {tabs[tab]?.map((entry) => (
            <div key={entry.id} className="prop-list-item">
              <span className="label">{entry.label}</span>
              <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                <span className="sub">{entry.detail}</span>
                {entry.onRemove ? (
                  <button type="button" className="btn btn-danger btn-sm" onClick={() => entry.onRemove?.(entry.id)}>
                    <FiTrash2 className="icon" />
                    削除
                  </button>
                ) : null}
              </div>
            </div>
          ))}
        </div>
      ))}
    </div>
  );
}
