import { cn } from "@/lib/cn";

export type MediaFilterOption = { id: string; label: string };

export function MediaFilterPanel({
  hidden,
  tags,
  categories,
  activeTagId,
  activeCategoryId,
  onSelectTag,
  onSelectCategory,
}: {
  hidden: boolean;
  tags: MediaFilterOption[];
  categories: MediaFilterOption[];
  activeTagId?: string;
  activeCategoryId?: string;
  onSelectTag: (id?: string) => void;
  onSelectCategory: (id?: string) => void;
}) {
  if (hidden) {
    return null;
  }

  return (
    <div className="filter-panel" id="filter-panel">
      {tags.length ? (
        <div className="filter-panel-group">
          <p className="filter-panel-label">タグ</p>
          <div className="filter-panel-chips" role="group" aria-label="タグで絞り込み">
            {tags.map((tag) => (
              <button
                key={tag.id}
                type="button"
                className={cn("chip filter-chip", activeTagId === tag.id && "is-active")}
                onClick={() => onSelectTag(activeTagId === tag.id ? undefined : tag.id)}
              >
                {tag.label}
              </button>
            ))}
          </div>
        </div>
      ) : null}
      {categories.length ? (
        <div className="filter-panel-group">
          <p className="filter-panel-label">カテゴリ</p>
          <div className="filter-panel-chips" role="group" aria-label="カテゴリで絞り込み">
            {categories.map((category) => (
              <button
                key={category.id}
                type="button"
                className={cn("chip filter-chip", activeCategoryId === category.id && "is-active")}
                onClick={() => onSelectCategory(activeCategoryId === category.id ? undefined : category.id)}
              >
                {category.label}
              </button>
            ))}
          </div>
        </div>
      ) : null}
    </div>
  );
}
