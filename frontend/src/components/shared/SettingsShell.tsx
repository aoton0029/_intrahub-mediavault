import { useState } from "react";
import type { ReactNode } from "react";
import { cn } from "@/lib/cn";

type SettingsTab = {
  key: string;
  label: string;
  description?: string;
  content: ReactNode;
};

export function SettingsShell({ tabs }: { tabs: SettingsTab[] }) {
  const [activeTab, setActiveTab] = useState(tabs[0]?.key ?? "");

  return (
    <div className="settings-shell">
      <div className="settings-tabs">
        {tabs.map((tab) => (
          <button key={tab.key} type="button" className={cn("settings-tab", activeTab === tab.key && "active")} onClick={() => setActiveTab(tab.key)}>
            {tab.label}
          </button>
        ))}
      </div>
      <div className="settings-panel">
        {tabs.map((tab) =>
          activeTab === tab.key ? (
            <section key={tab.key}>
              <h2>{tab.label}</h2>
              {tab.description ? <p className="desc">{tab.description}</p> : null}
              {tab.content}
            </section>
          ) : null,
        )}
      </div>
    </div>
  );
}
