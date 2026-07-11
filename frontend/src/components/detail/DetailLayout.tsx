import type { ReactNode } from "react";
import { FiBookmark, FiFileText, FiFilm, FiFolder, FiGitBranch, FiLayers, FiPaperclip, FiTag, FiTv, FiUsers } from "react-icons/fi";
import { PropertyList, type PropertyItem, RelatedWorksList, ResourceTabs, TagList, type TagListItem, type RelatedWork, type ResourceTabKey } from "@/components/shared";

export type RailSectionListItem = { id: string; label: string; actionLabel?: string };
export type Episode = { id: string; number: string; title: string };
export type Group = { id: string; label: string; episodes: Episode[] };
export type StaffMember = { id: string; label: string; sub: string; actionLabel?: string; onAction?: (id: string) => void };
export type StreamingLinkItem = { id: string; label: string; sub: string; actionLabel?: string; onAction?: (id: string) => void };

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
          <span className="label">{link.label}</span>
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

export function DetailMain({
  overview,
  propertyList,
  groups,
  staffList,
  relatedWorks,
  streaming,
  resourceTabs,
  groupTitle = "構成",
  groupActions,
  groupFooter,
  staffFooter,
  relatedWorksFooter,
  streamingFooter,
  resourceFooter,
}: {
  overview: ReactNode;
  propertyList?: PropertyItem[];
  groups?: Group[];
  staffList?: StaffMember[];
  relatedWorks?: RelatedWork[];
  streaming?: StreamingLinkItem[];
  resourceTabs?: Partial<Record<ResourceTabKey, { id: string; label: string; detail: string; onRemove?: (id: string) => void }[]>>;
  groupTitle?: string;
  groupActions?: (group: Group) => ReactNode;
  groupFooter?: ReactNode;
  staffFooter?: ReactNode;
  relatedWorksFooter?: ReactNode;
  streamingFooter?: ReactNode;
  resourceFooter?: ReactNode;
}) {
  return (
    <div className="detail-main">
      <DetailSection icon={<FiFileText className="icon" />} title="概要"><p>{overview}</p></DetailSection>
      {propertyList ? <DetailSection icon={<FiFilm className="icon" />} title="種別固有情報"><PropertyList items={propertyList} /></DetailSection> : null}
      {groups ? <DetailSection icon={<FiLayers className="icon" />} title={groupTitle}><GroupList groups={groups} renderGroupActions={groupActions} footerAction={groupFooter} /></DetailSection> : null}
      {staffList ? <DetailSection icon={<FiUsers className="icon" />} title="スタッフ"><StaffList members={staffList} footerAction={staffFooter} /></DetailSection> : null}
      {relatedWorks ? <DetailSection icon={<FiGitBranch className="icon" />} title="関連作品"><RelatedWorksList items={relatedWorks} />{relatedWorksFooter}</DetailSection> : null}
      {streaming ? <DetailSection icon={<FiTv className="icon" />} title="配信"><StreamingLinks links={streaming} footerAction={streamingFooter} /></DetailSection> : null}
      {resourceTabs ? <DetailSection icon={<FiPaperclip className="icon" />} title="リソース"><ResourceTabs tabs={resourceTabs} />{resourceFooter}</DetailSection> : null}
    </div>
  );
}

export function DetailLayout({ rail, main }: { rail: ReactNode; main: ReactNode }) {
  return <div className="detail-layout">{rail}{main}</div>;
}
