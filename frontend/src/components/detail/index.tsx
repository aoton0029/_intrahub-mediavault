import type { ReactNode } from 'react';
import { Link } from 'react-router-dom';
import { FiBookmark, FiFolder, FiTag, FiX } from 'react-icons/fi';
import {
  FavoriteToggle,
  RatingStars,
  StatusSwitcher,
  TagList,
} from '@/components/shared';
import type {
  DetailMylistItem,
  GroupItem,
  ItemStatus,
  PropertyItem,
  RailMetaItem,
  RelatedWorkItem,
  StaffMember,
  StreamingLinkItem,
  TagLikeItem,
} from '@/types/ui';

export function DetailLayout({ rail, main }: { rail: ReactNode; main: ReactNode }) {
  return (
    <div className="detail-layout">
      <aside className="detail-rail">{rail}</aside>
      <div className="detail-main">{main}</div>
    </div>
  );
}

export function DetailSection({
  icon,
  title,
  children,
}: {
  icon: ReactNode;
  title: string;
  children: ReactNode;
}) {
  return (
    <section className="doc-section">
      <h3>
        {icon}
        <span>{title}</span>
      </h3>
      {children}
    </section>
  );
}

export function PropertyList({ items }: { items: PropertyItem[] }) {
  return (
    <div className="prop-group">
      {items.map((item) => (
        <div className="prop-row" key={item.key}>
          <div className="key">{item.label}</div>
          <div className={item.muted ? 'val muted' : 'val'}>{item.value}</div>
        </div>
      ))}
    </div>
  );
}

export function EpisodeRow({ number, title, meta }: { number: string; title: string; meta?: string }) {
  return (
    <div className="episode-row">
      <div className="num">{number}</div>
      <div>
        <div>{title}</div>
        {meta ? <div className="sub">{meta}</div> : null}
      </div>
    </div>
  );
}

export function GroupList({ groups }: { groups: GroupItem[] }) {
  return (
    <div>
      {groups.map((group) => (
        <div className="group-block" key={group.id}>
          <div className="group-header">
            <span>{group.title}</span>
            {group.subtitle ? <span className="sub">{group.subtitle}</span> : null}
          </div>
          {group.episodes.map((episode) => (
            <EpisodeRow key={episode.id} number={episode.number} title={episode.title} meta={episode.meta} />
          ))}
        </div>
      ))}
    </div>
  );
}

export function StaffList({ members }: { members: StaffMember[] }) {
  return (
    <div className="prop-group">
      {members.map((member) => (
        <div className="prop-list-item" key={member.id}>
          <span className="label">{member.label}</span>
          {member.subLabel ? <span className="sub">{member.subLabel}</span> : null}
        </div>
      ))}
    </div>
  );
}

export function StreamingLinks({ links }: { links: StreamingLinkItem[] }) {
  return (
    <div className="prop-group">
      {links.map((link) => (
        <div className="prop-list-item" key={link.id}>
          <span className="label">{link.label}</span>
          {link.subLabel ? <span className="sub">{link.subLabel}</span> : null}
        </div>
      ))}
    </div>
  );
}

export function RelatedWorksList({ items }: { items: RelatedWorkItem[] }) {
  return (
    <div>
      {items.map((item) => (
        <div className="result-row" key={item.id}>
          <div className="thumb" />
          <div className="info">
            <div className="title">{item.title}</div>
            <div className="sub">{item.meta}</div>
          </div>
          {item.to ? (
            <Link className="btn btn-ghost btn-sm" to={item.to}>
              開く
            </Link>
          ) : null}
        </div>
      ))}
    </div>
  );
}

export function DetailRail({
  cover,
  title,
  originalTitle,
  status,
  rating,
  favorite,
  metaItems,
  tags,
  categories,
  mylists,
  onStatusChange,
  onRatingChange,
  onFavoriteChange,
  onAddTag,
  onRemoveTag,
  onAddCategory,
  onRemoveCategory,
  onRemoveMylist,
}: {
  cover?: string;
  title: string;
  originalTitle?: string;
  status: ItemStatus;
  rating: number;
  favorite: boolean;
  metaItems: RailMetaItem[];
  tags: TagLikeItem[];
  categories: TagLikeItem[];
  mylists: DetailMylistItem[];
  onStatusChange: (value: ItemStatus) => void;
  onRatingChange: (value: number) => void;
  onFavoriteChange: (value: boolean) => void;
  onAddTag: (label: string) => void;
  onRemoveTag: (id: string) => void;
  onAddCategory: (label: string) => void;
  onRemoveCategory: (id: string) => void;
  onRemoveMylist: (id: string) => void;
}) {
  return (
    <>
      <div className="doc-cover" style={cover ? { backgroundImage: `url(${cover})`, backgroundSize: 'cover' } : undefined} />
      <h2 className="doc-title">{title}</h2>
      {originalTitle ? <div className="doc-original">{originalTitle}</div> : null}

      <div className="rail-facts">
        <StatusSwitcher value={status} onChange={onStatusChange} />
        <RatingStars value={rating} onChange={onRatingChange} />
        <FavoriteToggle value={favorite} onChange={onFavoriteChange} />
        {metaItems.map((item) => (
          <div className={item.muted ? 'meta-item muted' : 'meta-item'} key={item.id}>
            <item.icon className="icon" />
            <span>{item.label}</span>
            <span>{item.value}</span>
          </div>
        ))}
      </div>

      <hr className="rail-divider" />

      <div className="rail-section">
        <h3>
          <FiTag className="icon" />
          タグ
        </h3>
        <TagList kind="tag" items={tags} onAdd={onAddTag} onRemove={onRemoveTag} />
      </div>

      <div className="rail-section">
        <h3>
          <FiFolder className="icon" />
          カテゴリ
        </h3>
        <TagList kind="category" items={categories} onAdd={onAddCategory} onRemove={onRemoveCategory} />
      </div>

      <div className="rail-section">
        <h3>
          <FiBookmark className="icon" />
          マイリスト
        </h3>
        <div className="prop-group">
          {mylists.map((mylist) => (
            <div className="prop-list-item" key={mylist.id}>
              <span className="label">{mylist.label}</span>
              <button type="button" className="btn btn-ghost btn-sm" onClick={() => onRemoveMylist(mylist.id)}>
                <FiX className="icon" />
                解除
              </button>
            </div>
          ))}
          <Link to="/mylists" className="btn btn-ghost btn-sm" style={{ marginTop: 6 }}>
            追加先を開く
          </Link>
        </div>
      </div>
    </>
  );
}
