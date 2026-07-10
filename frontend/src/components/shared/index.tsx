import { useEffect, useRef, useState, type ReactNode } from 'react';
import { Link } from 'react-router-dom';
import {
  FiArrowDown,
  FiCheckCircle,
  FiChevronDown,
  FiCircle,
  FiHeart,
  FiPaperclip,
  FiPlayCircle,
  FiPlus,
  FiSearch,
  FiStar,
  FiTrash2,
} from 'react-icons/fi';
import { cn } from '@/lib/cn';
import type {
  FilterChip,
  ItemStatus,
  LiteratureItem,
  MediaCardItem,
  ResourceTabItem,
  SelectOption,
  TagLikeItem,
} from '@/types/ui';

function Cover({ coverUrl, badge }: { coverUrl?: string; badge?: string }) {
  return (
    <div className="cover" style={coverUrl ? { backgroundImage: `url(${coverUrl})`, backgroundSize: 'cover' } : undefined}>
      {badge ? <span className="badge">{badge}</span> : null}
    </div>
  );
}

function MiniStars({ value = 0 }: { value?: number }) {
  return (
    <div className="rating-stars-mini" aria-label={`rating ${value}`}>
      {Array.from({ length: 5 }, (_, index) => (
        <FiStar key={index} className={cn('icon', index < Math.round(value) && 'is-full')} />
      ))}
    </div>
  );
}

export function MediaCard({
  item,
  variant = 'default',
}: {
  item: MediaCardItem;
  variant?: 'default' | 'compact' | 'search-result';
}) {
  const className = cn('media-card', variant === 'compact' && 'is-compact', variant === 'search-result' && 'search-result');
  const body = (
    <>
      <Cover coverUrl={item.coverUrl} badge={item.badge} />
      <div className="body">
        <div className="mb-2 flex items-start justify-between gap-2">
          <h3 className="title">{item.title}</h3>
          {item.isFavorite ? <FiHeart className="fav icon" /> : null}
        </div>
        <div className="meta">{item.meta}</div>
        {typeof item.rating === 'number' ? (
          <div className="mt-2 flex items-center gap-2">
            <MiniStars value={item.rating} />
            <span className="rating">
              <span className="val">{item.rating.toFixed(1)}</span>
            </span>
          </div>
        ) : null}
        {variant === 'search-result' && item.actionLabel ? (
          <button type="button" className="btn btn-accent btn-sm mt-3" disabled={item.actionDisabled}>
            {item.actionDisabled ? '取り込み済み' : item.actionLabel}
          </button>
        ) : null}
      </div>
    </>
  );

  if (item.to) {
    return (
      <Link to={item.to} className={className} style={{ display: 'block', color: 'inherit', textDecoration: 'none' }}>
        {body}
      </Link>
    );
  }

  return <div className={className}>{body}</div>;
}

export function MediaGrid({
  items,
  density = 'default',
  variant = 'default',
}: {
  items: MediaCardItem[];
  density?: 'default' | 'compact';
  variant?: 'default' | 'compact' | 'search-result';
}) {
  return (
    <div className={cn('card-grid', density === 'compact' && 'is-compact')}>
      {items.map((item) => (
        <MediaCard key={item.id} item={item} variant={variant === 'default' && density === 'compact' ? 'compact' : variant} />
      ))}
    </div>
  );
}

export function LiteratureRow({ item }: { item: LiteratureItem }) {
  const content = (
    <>
      <div className="thumb" style={item.coverUrl ? { backgroundImage: `url(${item.coverUrl})`, backgroundSize: 'cover' } : undefined} />
      <div className="info">
        <div className="title">{item.title}</div>
        <div className="byline">
          {item.byline}
          {item.meta ? <span className="sep">•</span> : null}
          {item.meta}
        </div>
        {item.doi ? <div className="doi">{item.doi}</div> : null}
        {item.tags?.length ? (
          <div className="prop-taglist">
            {item.tags.map((tag) => (
              <span className="tag-pill" key={tag.id}>
                {tag.label}
              </span>
            ))}
          </div>
        ) : null}
      </div>
      <div className="aside">
        {typeof item.rating === 'number' ? <MiniStars value={item.rating} /> : null}
        <FiHeart className={cn('fav-mark', item.favorite && 'is-active')} />
      </div>
    </>
  );

  if (item.to) {
    return (
      <Link to={item.to} className="lit-row" style={{ textDecoration: 'none' }}>
        {content}
      </Link>
    );
  }

  return <div className="lit-row">{content}</div>;
}

export function LiteratureList({ items }: { items: LiteratureItem[] }) {
  return (
    <div className="lit-list">
      {items.map((item) => (
        <LiteratureRow key={item.id} item={item} />
      ))}
    </div>
  );
}

export function FilterToolbar({
  chips,
  filters = [],
  sortOptions,
  sortValue,
  onSortChange,
  searchValue,
  onSearchChange,
  searchPlaceholder = '検索する',
}: {
  chips: FilterChip[];
  filters?: Array<{ id: string; label: string; value: string; options: SelectOption[]; onChange: (value: string) => void }>;
  sortOptions: SelectOption[];
  sortValue: string;
  onSortChange: (value: string) => void;
  searchValue: string;
  onSearchChange: (value: string) => void;
  searchPlaceholder?: string;
}) {
  return (
    <div className="filter-toolbar">
      <div className="filter-bar">
        {chips.map((chip) => (
          <button
            key={chip.id}
            type="button"
            className={cn('chip', chip.active && 'active', chip.addTrigger && 'chip-add')}
          >
            {chip.addTrigger ? <FiPlus className="icon" /> : null}
            {chip.label}
          </button>
        ))}
        {filters.map((filter) => (
          <label className={cn('filter-select', filter.value && 'active')} key={filter.id}>
            <span>{filter.label}</span>
            <select value={filter.value} onChange={(event) => filter.onChange(event.target.value)} aria-label={filter.label}>
              {filter.options.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </label>
        ))}
      </div>
      <div className="sort-search-group">
        <label className="sort-select">
          <FiArrowDown className="icon" />
          <select value={sortValue} onChange={(event) => onSortChange(event.target.value)} aria-label="並び替え">
            {sortOptions.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        </label>
        <label className="search-box">
          <FiSearch className="icon" />
          <input
            value={searchValue}
            onChange={(event) => onSearchChange(event.target.value)}
            placeholder={searchPlaceholder}
            aria-label="検索"
          />
        </label>
      </div>
    </div>
  );
}

export function useInfiniteScroll({
  onReachEnd,
  enabled = true,
}: {
  onReachEnd: () => void;
  enabled?: boolean;
}) {
  const observerRef = useRef<IntersectionObserver | null>(null);

  return {
    ref: (element: HTMLDivElement | null) => {
      observerRef.current?.disconnect();
      if (!enabled || !element) {
        return;
      }

      observerRef.current = new IntersectionObserver((entries) => {
        if (entries[0]?.isIntersecting) {
          onReachEnd();
        }
      });

      observerRef.current.observe(element);
    },
  };
}

export function LoadMoreSentinel({
  loading = true,
  label = 'さらに読み込み中',
  innerRef,
}: {
  loading?: boolean;
  label?: string;
  innerRef?: (element: HTMLDivElement | null) => void;
}) {
  return (
    <div className="load-more-sentinel" ref={innerRef}>
      {loading ? <span className="spinner" /> : null}
      <span>{label}</span>
    </div>
  );
}

const statusMap = {
  not_started: { label: '未着手', icon: FiCircle },
  in_progress: { label: '進行中', icon: FiPlayCircle },
  done: { label: '完了', icon: FiCheckCircle },
} satisfies Record<ItemStatus, { label: string; icon: typeof FiCircle }>;

export function StatusSwitcher({
  value,
  onChange,
  disabled,
}: {
  value: ItemStatus;
  onChange: (value: ItemStatus) => void;
  disabled?: boolean;
}) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement | null>(null);
  const current = statusMap[value];

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (!ref.current?.contains(event.target as Node)) {
        setOpen(false);
      }
    }

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  return (
    <div className="status-switcher" ref={ref}>
      <button
        type="button"
        className="status-trigger meta-item"
        data-status-trigger
        data-current={value}
        onClick={() => !disabled && setOpen((currentOpen) => !currentOpen)}
        disabled={disabled}
      >
        <current.icon className="icon" />
        <span className="label">{current.label}</span>
        <FiChevronDown className="chevron" />
      </button>
      <div className="status-popover" hidden={!open}>
        {(Object.keys(statusMap) as ItemStatus[]).map((status) => {
          const option = statusMap[status];
          return (
            <button
              type="button"
              className="status-option"
              key={status}
              data-status={status}
              onClick={() => {
                onChange(status);
                setOpen(false);
              }}
            >
              <option.icon className="icon" />
              <span className="label">{option.label}</span>
            </button>
          );
        })}
      </div>
    </div>
  );
}

export function RatingStars({
  value,
  onChange,
  readOnly,
}: {
  value: number;
  onChange: (value: number) => void;
  readOnly?: boolean;
}) {
  const [hoverValue, setHoverValue] = useState<number | null>(null);
  const visibleValue = hoverValue ?? value;

  return (
    <div
      className="rating-stars"
      data-rating={value}
      onMouseLeave={() => setHoverValue(null)}
      aria-label="rating"
    >
      {Array.from({ length: 5 }, (_, index) => {
        const starValue = index + 1;

        return (
          <button
            type="button"
            key={starValue}
            className="star-btn"
            data-rating={starValue}
            onMouseOver={() => !readOnly && setHoverValue(starValue)}
            onClick={() => !readOnly && onChange(starValue)}
            disabled={readOnly}
          >
            <FiStar className={cn('icon', starValue <= visibleValue && 'is-full')} />
          </button>
        );
      })}
      <span className="val">{value.toFixed(1)}</span>
    </div>
  );
}

export function RatingStarsMini({ value }: { value: number }) {
  return <MiniStars value={value} />;
}

export function FavoriteToggle({
  value,
  onChange,
}: {
  value: boolean;
  onChange: (value: boolean) => void;
}) {
  return (
    <button
      type="button"
      className={cn('favorite-toggle meta-item', value && 'is-active')}
      data-favorite-toggle
      onClick={() => onChange(!value)}
    >
      <FiHeart className="icon" />
      <span>{value ? 'お気に入り' : 'お気に入りに追加'}</span>
    </button>
  );
}

export function TagList({
  kind,
  items,
  onAdd,
  onRemove,
  addLabel,
}: {
  kind: 'tag' | 'category';
  items: TagLikeItem[];
  onAdd: (label: string) => void;
  onRemove: (id: string) => void;
  addLabel?: string;
}) {
  const [isAdding, setIsAdding] = useState(false);
  const [draft, setDraft] = useState('');

  const cancel = () => {
    setDraft('');
    setIsAdding(false);
  };

  const confirm = () => {
    const trimmed = draft.trim();
    if (!trimmed) {
      cancel();
      return;
    }

    onAdd(trimmed);
    setDraft('');
  };

  return (
    <div className="prop-taglist">
      {items.map((item) => (
        <span className="tag-pill" key={item.id}>
          {item.label}
          <button
            type="button"
            className="tag-remove"
            aria-label={`${item.label}を削除`}
            onClick={() => onRemove(item.id)}
          >
            ×
          </button>
        </span>
      ))}
      {isAdding ? (
        <input
          autoFocus
          className="tag-add-input"
          value={draft}
          onChange={(event) => setDraft(event.target.value)}
          onBlur={() => window.setTimeout(cancel, 100)}
          onKeyDown={(event) => {
            if (event.key === 'Enter') {
              confirm();
            }
            if (event.key === 'Escape') {
              cancel();
            }
          }}
          placeholder={kind === 'tag' ? 'タグ名を入力してEnter' : 'カテゴリ名を入力してEnter'}
        />
      ) : (
        <button type="button" className="tag-add-trigger" onClick={() => setIsAdding(true)}>
          <FiPlus className="icon" />
          {addLabel ?? (kind === 'tag' ? 'タグ追加' : 'カテゴリ追加')}
        </button>
      )}
    </div>
  );
}

export function MylistCover({ count, covers }: { count: 1 | 2 | 3 | 4; covers: string[] }) {
  return (
    <div className={cn('mylist-cover', `n${count}`)}>
      {Array.from({ length: count }, (_, index) => (
        <div
          key={`${covers[index] ?? 'placeholder'}-${index}`}
          style={covers[index] ? { backgroundImage: `url(${covers[index]})` } : undefined}
        />
      ))}
      <span className="badge">{count} covers</span>
    </div>
  );
}

export function Modal({
  open,
  onClose,
  title,
  children,
}: {
  open: boolean;
  onClose: () => void;
  title: string;
  children: ReactNode;
}) {
  useEffect(() => {
    if (!open) {
      return;
    }

    function handleEscape(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        onClose();
      }
    }

    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }, [open, onClose]);

  if (!open) {
    return null;
  }

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={(event) => event.stopPropagation()}>
        <h2>{title}</h2>
        {children}
      </div>
    </div>
  );
}

export function EmptyState({
  title,
  description,
  action,
}: {
  title: string;
  description: string;
  action?: ReactNode;
}) {
  return (
    <div className="empty-state">
      <h3>{title}</h3>
      <p>{description}</p>
      {action ? <div className="mt-3">{action}</div> : null}
    </div>
  );
}

export function ResourceTabs({
  tabs,
  defaultTab,
}: {
  tabs: ResourceTabItem[];
  defaultTab?: string;
}) {
  const [activeTab, setActiveTab] = useState(defaultTab ?? tabs[0]?.key);
  const visibleTab = tabs.find((tab) => tab.key === activeTab) ?? tabs[0];

  return (
    <div className="resource-tabs">
      <div className="resource-tabbar" role="tablist">
        {tabs.map((tab) => (
          <button
            type="button"
            key={tab.key}
            className={cn('chip', tab.key === visibleTab?.key && 'active')}
            onClick={() => setActiveTab(tab.key)}
            role="tab"
            aria-selected={tab.key === visibleTab?.key}
          >
            <tab.icon className="icon" />
            {tab.label}
          </button>
        ))}
      </div>
      <div className="resource-panels">
        <div className="resource-panel" style={{ display: 'block' }}>
          {visibleTab?.content}
        </div>
      </div>
    </div>
  );
}

export function FormSection({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section>
      <div className="form-section-title">{title}</div>
      {children}
    </section>
  );
}

export function FormGrid({ children }: { children: ReactNode }) {
  return <div className="form-grid">{children}</div>;
}

export function FormField({
  label,
  required,
  error,
  hint,
  full,
  children,
}: {
  label: string;
  required?: boolean;
  error?: string;
  hint?: string;
  full?: boolean;
  children: ReactNode;
}) {
  return (
    <div className={cn('form-field', full && 'full', error && 'error')}>
      <label>
        <span>{label}</span>
        {required ? <span className="required">必須</span> : null}
      </label>
      {children}
      {error ? <div className="field-error">{error}</div> : null}
      {hint ? <div className="field-hint">{hint}</div> : null}
    </div>
  );
}

export function FormActions({ children }: { children: ReactNode }) {
  return <div className="form-actions">{children}</div>;
}

export function ResourceList({ items }: { items: Array<{ id: string; label: string; sub?: string; href?: string }> }) {
  return (
    <div className="prop-group">
      {items.map((item) => (
        <div className="prop-list-item" key={item.id}>
          <div>
            {item.href ? (
              <a href={item.href} target="_blank" rel="noreferrer">
                {item.label}
              </a>
            ) : (
              <span className="label">{item.label}</span>
            )}
          </div>
          {item.sub ? <span className="sub">{item.sub}</span> : <FiPaperclip className="icon" />}
        </div>
      ))}
    </div>
  );
}

export function DangerAction({ label }: { label: string }) {
  return (
    <button type="button" className="btn btn-danger btn-sm">
      <FiTrash2 className="icon" />
      {label}
    </button>
  );
}
