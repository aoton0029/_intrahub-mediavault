import type { ReactNode } from "react";
import { FiBookmark, FiFileText, FiFilm, FiFolder, FiGitBranch, FiLayers, FiPaperclip, FiTag, FiTv, FiUsers } from "react-icons/fi";
import { PropertyList, type PropertyItem, RelatedWorksList, ResourceTabs, TagList, type TagListItem, type RelatedWork, type ResourceTabKey } from "@/components/shared";

export type RailSectionListItem = { id: string; label: string; actionLabel?: string };
export type Episode = { id: string; number: string; title: string };
export type Group = { id: string; label: string; episodes: Episode[] };

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
  onRemoveMylist,
}: {
  cover?: ReactNode;
  title: string;
  originalTitle?: string;
  facts: ReactNode[];
  tags?: TagListItem[];
  categories?: TagListItem[];
  mylists?: RailSectionListItem[];
  onRemoveMylist?: (id: string) => void;
}) {
  return (
    <aside className="detail-rail">
      {cover ?? <div className="doc-cover" />}
      <h1 className="doc-title">{title}</h1>
      {originalTitle ? <div className="doc-original">{originalTitle}</div> : null}
      <div className="rail-facts">{facts}</div>
      <hr className="rail-divider" />
      <RailSection title="タグ" icon={<FiTag className="icon" />}><TagList kind="tag" items={tags} /></RailSection>
      <RailSection title="カテゴリ" icon={<FiFolder className="icon" />}><TagList kind="category" items={categories} /></RailSection>
      <RailSection title="マイリスト" icon={<FiBookmark className="icon" />} items={mylists} onRemoveItem={onRemoveMylist} />
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

export function GroupList({ groups }: { groups: Group[] }) {
  return (
    <div>
      {groups.map((group) => (
        <div key={group.id} className="group-block">
          <div className="group-header"><span>{group.label}</span></div>
          {group.episodes.map((episode) => (
            <EpisodeRow key={episode.id} episode={episode} />
          ))}
        </div>
      ))}
    </div>
  );
}

export function StaffList({ members }: { members: { id: string; label: string; sub: string }[] }) {
  return (
    <div>
      {members.map((member) => (
        <div key={member.id} className="prop-list-item">
          <span className="label">{member.label}</span>
          <span className="sub">{member.sub}</span>
        </div>
      ))}
    </div>
  );
}

export function StreamingLinks({ links }: { links: { id: string; label: string; sub: string }[] }) {
  return (
    <div>
      {links.map((link) => (
        <div key={link.id} className="prop-list-item">
          <span className="label">{link.label}</span>
          <span className="sub">{link.sub}</span>
        </div>
      ))}
    </div>
  );
}

export function DetailMain({
  overview,
  propertyList,
  groups,
  staffList,
  relatedWorks,
  streaming,
  resourceTabs,
}: {
  overview: ReactNode;
  propertyList?: PropertyItem[];
  groups?: Group[];
  staffList?: { id: string; label: string; sub: string }[];
  relatedWorks?: RelatedWork[];
  streaming?: { id: string; label: string; sub: string }[];
  resourceTabs?: Partial<Record<ResourceTabKey, { id: string; label: string; detail: string }[]>>;
}) {
  return (
    <div className="detail-main">
      <DetailSection icon={<FiFileText className="icon" />} title="概要"><p>{overview}</p></DetailSection>
      {propertyList ? <DetailSection icon={<FiFilm className="icon" />} title="種別固有情報"><PropertyList items={propertyList} /></DetailSection> : null}
      {groups ? <DetailSection icon={<FiLayers className="icon" />} title="構成"><GroupList groups={groups} /></DetailSection> : null}
      {staffList ? <DetailSection icon={<FiUsers className="icon" />} title="スタッフ"><StaffList members={staffList} /></DetailSection> : null}
      {relatedWorks ? <DetailSection icon={<FiGitBranch className="icon" />} title="関連作品"><RelatedWorksList items={relatedWorks} /></DetailSection> : null}
      {streaming ? <DetailSection icon={<FiTv className="icon" />} title="配信"><StreamingLinks links={streaming} /></DetailSection> : null}
      {resourceTabs ? <DetailSection icon={<FiPaperclip className="icon" />} title="リソース"><ResourceTabs tabs={resourceTabs} /></DetailSection> : null}
    </div>
  );
}

export function DetailLayout({ rail, main }: { rail: ReactNode; main: ReactNode }) {
  return <div className="detail-layout">{rail}{main}</div>;
}
