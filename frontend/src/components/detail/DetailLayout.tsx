import { useEffect, useState, type ReactNode } from "react";
import { FiBookmark, FiCheck, FiEdit2, FiFileText, FiFilm, FiFolder, FiGitBranch, FiImage, FiLayers, FiTag, FiTv, FiUsers, FiX } from "react-icons/fi";
import { FaAmazon } from "react-icons/fa";
import { SiAppletv, SiDmm, SiNetflix } from "react-icons/si";
import { PropertyList, type PropertyItem, RelatedWorksList, ResourceEntryList, resourceTabIcons, resourceTabLabels, Tabs, type TabItem, TagList, type TagListItem, type RelatedWork, type ResourceTabKey } from "@/components/shared";

export type RailSectionListItem = { id: string; label: string; actionLabel?: string };
export type Episode = { id: string; number: string; title: string };
export type Group = { id: string; label: string; episodes: Episode[] };
export type StaffMember = { id: string; label: string; sub: string; actionLabel?: string; onAction?: (id: string) => void };
export type StreamingPlatform = "netflix" | "amazon_prime" | "disney_plus" | "dmm_tv" | "apple_tv";
export type StreamingLinkItem = { id: string; label: string; sub: string; platform?: StreamingPlatform; actionLabel?: string; onAction?: (id: string) => void };
export type ImageItem = { id: string; url: string; isCover?: boolean; onSetCover?: (url: string) => void; onRemove?: (id: string) => void };

const STREAMING_PLATFORM_ICONS: Record<StreamingPlatform, { icon: ReactNode; color: string }> = {
  netflix: { icon: <SiNetflix />, color: "#E50914" },
  amazon_prime: { icon: <FaAmazon />, color: "#00A8E1" },
  disney_plus: { icon: <FiTv />, color: "#113CCF" },
  dmm_tv: { icon: <SiDmm />, color: "#000000" },
  apple_tv: { icon: <SiAppletv />, color: "#000000" },
};

export function StreamingPlatformIcon({ platform }: { platform?: StreamingPlatform }) {
  if (!platform) return null;
  const entry = STREAMING_PLATFORM_ICONS[platform];
  if (!entry) return null;
  return (
    <span className="streaming-platform-icon" style={{ color: entry.color }}>
      {entry.icon}
    </span>
  );
}

export function RailSection({
  title,
  icon,
  children,
  items,
  onRemoveItem,
  footerAction,
}: {
  title: string;
  icon: ReactNode;
  children?: ReactNode;
  items?: RailSectionListItem[];
  onRemoveItem?: (id: string) => void;
  footerAction?: ReactNode;
}) {
  return (
    <section className="rail-section">
      <h3>{icon}{title}</h3>
      {children}
      {items?.map((item) => (
        <div key={item.id} className="prop-list-item">
          <span className="label">{item.label}</span>
          {item.actionLabel ? (
            <button type="button" className="btn btn-ghost btn-sm" onClick={() => onRemoveItem?.(item.id)}>
              {item.actionLabel}
            </button>
          ) : null}
        </div>
      ))}
      {footerAction}
    </section>
  );
}

export function DetailRail({
  cover,
  title,
  originalTitle,
  facts,
  tags = [],
  categories = [],
  mylists = [],
  onAddTag,
  onRemoveTag,
  onAddCategory,
  onRemoveCategory,
  onRemoveMylist,
  mylistsFooter,
}: {
  cover?: ReactNode;
  title: string;
  originalTitle?: string;
  facts: ReactNode[];
  tags?: TagListItem[];
  categories?: TagListItem[];
  mylists?: RailSectionListItem[];
  onAddTag?: (name: string) => void;
  onRemoveTag?: (id: string) => void;
  onAddCategory?: (name: string) => void;
  onRemoveCategory?: (id: string) => void;
  onRemoveMylist?: (id: string) => void;
  mylistsFooter?: ReactNode;
}) {
  return (
    <aside className="detail-rail">
      {cover ?? <div className="doc-cover" />}
      <h1 className="doc-title">{title}</h1>
      {originalTitle ? <div className="doc-original">{originalTitle}</div> : null}
      <div className="rail-facts">{facts}</div>
      <hr className="rail-divider" />
      <RailSection title="タグ" icon={<FiTag className="icon" />}><TagList kind="tag" items={tags} onAdd={onAddTag} onRemove={onRemoveTag} /></RailSection>
      <RailSection title="カテゴリ" icon={<FiFolder className="icon" />}><TagList kind="category" items={categories} onAdd={onAddCategory} onRemove={onRemoveCategory} /></RailSection>
      <RailSection title="マイリスト" icon={<FiBookmark className="icon" />} items={mylists} onRemoveItem={onRemoveMylist} footerAction={mylistsFooter} />
    </aside>
  );
}

export function DetailSection({ icon, title, children }: { icon: ReactNode; title: string; children: ReactNode }) {
  return (
    <section className="doc-section">
      <h3>{icon}{title}</h3>
      {children}
    </section>
  );
}

export function EpisodeRow({ episode }: { episode: Episode }) {
  return (
    <div className="episode-row">
      <span className="num">{episode.number}</span>
      <span>{episode.title}</span>
    </div>
  );
}

export function GroupList({
  groups,
  renderGroupActions,
  footerAction,
}: {
  groups: Group[];
  renderGroupActions?: (group: Group) => ReactNode;
  footerAction?: ReactNode;
}) {
  return (
    <div>
      {groups.map((group) => (
        <div key={group.id} className="group-block">
          <div className="group-header">
            <span>{group.label}</span>
            {renderGroupActions ? <div>{renderGroupActions(group)}</div> : null}
          </div>
          {group.episodes.map((episode) => (
            <EpisodeRow key={episode.id} episode={episode} />
          ))}
        </div>
      ))}
      {footerAction}
    </div>
  );
}

export function StaffList({
  members,
  footerAction,
}: {
  members: StaffMember[];
  footerAction?: ReactNode;
}) {
  return (
    <div>
      {members.map((member) => (
        <div key={member.id} className="prop-list-item">
          <span className="label">{member.label}</span>
          <div className="detail-section-actions">
            <span className="sub">{member.sub}</span>
            {member.actionLabel ? (
              <button type="button" className="btn btn-danger btn-sm" onClick={() => member.onAction?.(member.id)}>
                {member.actionLabel}
              </button>
            ) : null}
          </div>
        </div>
      ))}
      {footerAction}
    </div>
  );
}

export function StreamingLinks({
  links,
  footerAction,
}: {
  links: StreamingLinkItem[];
  footerAction?: ReactNode;
}) {
  return (
    <div>
      {links.map((link) => (
        <div key={link.id} className="prop-list-item">
          <span className="label">
            <StreamingPlatformIcon platform={link.platform} />
            {link.label}
          </span>
          <div className="detail-section-actions">
            <span className="sub">{link.sub}</span>
            {link.actionLabel ? (
              <button type="button" className="btn btn-danger btn-sm" onClick={() => link.onAction?.(link.id)}>
                {link.actionLabel}
              </button>
            ) : null}
          </div>
        </div>
      ))}
      {footerAction}
    </div>
  );
}

export function ImageGrid({
  items,
  footerAction,
}: {
  items: ImageItem[];
  footerAction?: ReactNode;
}) {
  return (
    <div>
      <div className="image-grid" style={{ display: "flex", flexWrap: "wrap", gap: 12 }}>
        {items.map((image) => (
          <div key={image.id} className="image-grid-item" style={{ width: 160 }}>
            <img
              src={image.url}
              alt=""
              style={{ width: "100%", height: 160, objectFit: "cover", borderRadius: 4 }}
            />
            <div className="detail-section-actions" style={{ marginTop: 4 }}>
              <button
                type="button"
                className="btn btn-ghost btn-sm"
                disabled={image.isCover}
                onClick={() => image.onSetCover?.(image.url)}
              >
                {image.isCover ? "サムネイル設定済み" : "サムネイルに設定"}
              </button>
              <button type="button" className="btn btn-danger btn-sm" onClick={() => image.onRemove?.(image.id)}>
                削除
              </button>
            </div>
          </div>
        ))}
      </div>
      {footerAction}
    </div>
  );
}

export function OverviewSection({
  overview,
  onSave,
}: {
  overview: string;
  onSave?: (value: string) => void;
}) {
  const [isEditing, setEditing] = useState(false);
  const [draft, setDraft] = useState(overview);

  useEffect(() => setDraft(overview), [overview]);

  if (!onSave) {
    return <p>{overview || "概要はまだ登録されていません。"}</p>;
  }

  if (!isEditing) {
    return (
      <div className="overview-view">
        <p>{overview || "概要はまだ登録されていません。"}</p>
        <button type="button" className="btn btn-ghost btn-sm" onClick={() => setEditing(true)}>
          <FiEdit2 className="icon" />
          編集
        </button>
      </div>
    );
  }

  return (
    <div className="overview-editor">
      <textarea
        className="overview-textarea"
        rows={5}
        value={draft}
        onChange={(event) => setDraft(event.target.value)}
      />
      <div className="detail-section-actions">
        <button
          type="button"
          className="btn btn-ghost btn-sm"
          onClick={() => {
            setDraft(overview);
            setEditing(false);
          }}
        >
          <FiX className="icon" />
          キャンセル
        </button>
        <button
          type="button"
          className="btn btn-accent btn-sm"
          onClick={() => {
            onSave(draft);
            setEditing(false);
          }}
        >
          <FiCheck className="icon" />
          保存
        </button>
      </div>
    </div>
  );
}

export function DetailMain({
  overview,
  onUpdateOverview,
  propertyList,
  groups,
  staffList,
  castList,
  relatedWorks,
  streaming,
  images,
  resourceTabs,
  groupTitle = "構成",
  groupActions,
  groupFooter,
  staffFooter,
  castFooter,
  relatedWorksFooter,
  streamingFooter,
  imagesFooter,
  linksFooter,
  filesFooter,
  trailersFooter,
}: {
  overview: string;
  onUpdateOverview?: (value: string) => void;
  propertyList?: PropertyItem[];
  groups?: Group[];
  staffList?: StaffMember[];
  castList?: StaffMember[];
  relatedWorks?: RelatedWork[];
  streaming?: StreamingLinkItem[];
  images?: ImageItem[];
  resourceTabs?: Partial<Record<ResourceTabKey, { id: string; label: string; detail: string; onRemove?: (id: string) => void }[]>>;
  groupTitle?: string;
  groupActions?: (group: Group) => ReactNode;
  groupFooter?: ReactNode;
  staffFooter?: ReactNode;
  castFooter?: ReactNode;
  relatedWorksFooter?: ReactNode;
  streamingFooter?: ReactNode;
  imagesFooter?: ReactNode;
  linksFooter?: ReactNode;
  filesFooter?: ReactNode;
  trailersFooter?: ReactNode;
}) {
  const tabs: TabItem[] = [];

  if (propertyList) {
    tabs.push({
      key: "property",
      label: "プロパティ",
      icon: <FiFilm className="icon" />,
      content: <DetailSection icon={<FiFilm className="icon" />} title="プロパティ"><PropertyList items={propertyList} /></DetailSection>,
    });
  }

  if (groups) {
    tabs.push({
      key: "groups",
      label: groupTitle,
      icon: <FiLayers className="icon" />,
      content: <DetailSection icon={<FiLayers className="icon" />} title={groupTitle}><GroupList groups={groups} renderGroupActions={groupActions} footerAction={groupFooter} /></DetailSection>,
    });
  }

  if (staffList || castList) {
    tabs.push({
      key: "people",
      label: "スタッフ・キャスト",
      icon: <FiUsers className="icon" />,
      content: (
        <>
          {staffList ? <DetailSection icon={<FiUsers className="icon" />} title="スタッフ"><StaffList members={staffList} footerAction={staffFooter} /></DetailSection> : null}
          {castList ? <DetailSection icon={<FiUsers className="icon" />} title="キャスト"><StaffList members={castList} footerAction={castFooter} /></DetailSection> : null}
        </>
      ),
    });
  }

  if (relatedWorks) {
    tabs.push({
      key: "related",
      label: "関連作品",
      icon: <FiGitBranch className="icon" />,
      content: <DetailSection icon={<FiGitBranch className="icon" />} title="関連作品"><RelatedWorksList items={relatedWorks} />{relatedWorksFooter}</DetailSection>,
    });
  }

  if (streaming) {
    tabs.push({
      key: "streaming",
      label: "配信",
      icon: <FiTv className="icon" />,
      content: <DetailSection icon={<FiTv className="icon" />} title="配信"><StreamingLinks links={streaming} footerAction={streamingFooter} /></DetailSection>,
    });
  }

  if (images) {
    tabs.push({
      key: "images",
      label: "画像",
      icon: <FiImage className="icon" />,
      content: <DetailSection icon={<FiImage className="icon" />} title="画像"><ImageGrid items={images} footerAction={imagesFooter} /></DetailSection>,
    });
  }

  const resourceFooters: Partial<Record<ResourceTabKey, ReactNode>> = { links: linksFooter, files: filesFooter, trailers: trailersFooter };

  (["links", "files", "trailers"] as ResourceTabKey[]).forEach((key) => {
    const entries = resourceTabs?.[key];
    if (!entries) return;
    const Icon = resourceTabIcons[key];
    tabs.push({
      key,
      label: resourceTabLabels[key],
      icon: <Icon className="icon" />,
      content: (
        <DetailSection icon={<Icon className="icon" />} title={resourceTabLabels[key]}>
          <ResourceEntryList entries={entries} />
          {resourceFooters[key]}
        </DetailSection>
      ),
    });
  });

  return (
    <div className="detail-main">
      <DetailSection icon={<FiFileText className="icon" />} title="概要">
        <OverviewSection overview={overview} onSave={onUpdateOverview} />
      </DetailSection>
      {tabs.length ? <Tabs items={tabs} /> : null}
    </div>
  );
}

export function DetailLayout({ rail, main }: { rail: ReactNode; main: ReactNode }) {
  return <div className="detail-layout">{rail}{main}</div>;
}
