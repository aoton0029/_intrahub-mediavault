import { FiFilm, FiLink2, FiPaperclip, FiTrash2 } from "react-icons/fi";
import { Tabs } from "./Tabs";

export type ResourceTabKey = "links" | "files" | "trailers";

export type ResourceEntry = { id: string; label: string; detail: string; onRemove?: (id: string) => void };

export const resourceTabLabels: Record<ResourceTabKey, string> = { links: "リンク", files: "ファイル", trailers: "トレーラー" };
export const resourceTabIcons = { links: FiLink2, files: FiPaperclip, trailers: FiFilm } satisfies Record<ResourceTabKey, typeof FiLink2>;

export function ExternalLinkText({ text }: { text: string }) {
  if (/^https?:\/\//.test(text)) {
    return (
      <a className="sub" href={text} target="_blank" rel="noopener noreferrer">
        {text}
      </a>
    );
  }
  return <span className="sub">{text}</span>;
}

export function ResourceEntryList({ entries }: { entries: ResourceEntry[] }) {
  return (
    <>
      {entries.map((entry) => (
        <div key={entry.id} className="prop-list-item">
          <span className="label">{entry.label}</span>
          <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <ExternalLinkText text={entry.detail} />
            {entry.onRemove ? (
              <button type="button" className="btn btn-danger btn-sm" onClick={() => entry.onRemove?.(entry.id)}>
                <FiTrash2 className="icon" />
                削除
              </button>
            ) : null}
          </div>
        </div>
      ))}
    </>
  );
}

export function ResourceTabs({ tabs }: { tabs: Partial<Record<ResourceTabKey, ResourceEntry[]>> }) {
  const availableTabs = (Object.keys(tabs) as ResourceTabKey[]).filter((key) => tabs[key]?.length);

  return (
    <Tabs
      items={availableTabs.map((tab) => {
        const Icon = resourceTabIcons[tab];
        return {
          key: tab,
          label: resourceTabLabels[tab],
          icon: <Icon className="icon" />,
          content: <ResourceEntryList entries={tabs[tab] ?? []} />,
        };
      })}
    />
  );
}
