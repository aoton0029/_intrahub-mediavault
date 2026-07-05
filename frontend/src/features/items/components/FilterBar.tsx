import { useCategoriesQuery, useTagsQuery } from '../api';
import type { ItemFilters } from '../api';
import type { MediaType } from '../types';
import { mediaTypeLabels } from '../types';

/**
 * バックエンドにタグ/カテゴリの一覧取得エンドポイントが無かった頃のモックUIに合わせた
 * 静的な候補チップ。GET /tags が利用できない環境（未接続・エラー時）でも
 * 最低限のクイックフィルタを提供するためのフォールバックとして残す。
 */
const FALLBACK_TAG_CHIPS = [
  { id: 'SF', name: 'SF' },
  { id: 'zokuhen-machi', name: '続編待ち' },
];

interface FilterBarProps {
  filters: ItemFilters;
  onToggleFavorite: () => void;
  onSetMediaType: (value: MediaType | '') => void;
  onToggleTag: (tagId: string) => void;
  onToggleCategory: (categoryId: string) => void;
}

export function FilterBar({
  filters,
  onToggleFavorite,
  onSetMediaType,
  onToggleTag,
  onToggleCategory,
}: FilterBarProps) {
  const tagsQuery = useTagsQuery();
  const categoriesQuery = useCategoriesQuery();

  const tagChips = tagsQuery.data && tagsQuery.data.length > 0 ? tagsQuery.data : FALLBACK_TAG_CHIPS;
  const categoryChips = categoriesQuery.data ?? [];

  return (
    <div
      data-testid="filter-bar"
      className="mb-5 flex flex-wrap items-center gap-2"
    >
      <button
        type="button"
        aria-pressed={filters.is_favorite === true}
        onClick={onToggleFavorite}
        className={`inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs ${
          filters.is_favorite
            ? 'border-accent bg-accent-soft text-accent-strong'
            : 'border-border bg-bg-surface text-text-muted'
        }`}
      >
        ⭐ お気に入り
      </button>

      {tagChips.map((tag) => (
        <button
          key={tag.id}
          type="button"
          aria-pressed={filters.tag_id === tag.id}
          onClick={() => onToggleTag(tag.id)}
          className={`inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs ${
            filters.tag_id === tag.id
              ? 'border-accent bg-accent-soft text-accent-strong'
              : 'border-border bg-bg-surface text-text-muted'
          }`}
        >
          #{tag.name}
        </button>
      ))}

      {categoryChips.map((category) => (
        <button
          key={category.id}
          type="button"
          aria-pressed={filters.category_id === category.id}
          onClick={() => onToggleCategory(category.id)}
          className={`inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs ${
            filters.category_id === category.id
              ? 'border-accent bg-accent-soft text-accent-strong'
              : 'border-border bg-bg-surface text-text-muted'
          }`}
        >
          {category.name}
        </button>
      ))}

      <div className="inline-flex items-center gap-1.5 rounded-app border border-border bg-bg-input px-2.5 py-1 text-xs text-text-muted">
        種別
        <select
          value={filters.media_type ?? ''}
          onChange={(e) => onSetMediaType(e.target.value as MediaType | '')}
          className="bg-transparent text-inherit outline-none"
        >
          <option value="">すべて</option>
          {(Object.keys(mediaTypeLabels) as MediaType[]).map((value) => (
            <option key={value} value={value}>
              {mediaTypeLabels[value]}
            </option>
          ))}
        </select>
      </div>
    </div>
  );
}
