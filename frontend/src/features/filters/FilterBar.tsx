/**
 * フィルタチップバー（.filter-bar）
 *
 * TASK-0009: media_type・タグ・カテゴリ・お気に入り・statusの各フィルタチップと
 * タイトル検索ボックスを表示する。01_home.htmlの `.filter-bar`/`.chip`/`.search-box`
 * 構造・クラス名に準拠する。
 *
 * 既存の useItemFilters（TASK-0007）フックの実際の公開API
 * ({ filters, setMediaTypes, toggleTag, toggleCategory, setIsFavorite, toggleStatus, setTitle }）
 * に合わせて実装する。タスクドキュメント記載の `toggleMediaType`/`toggleFavorite` という名称は
 * 実際のフックには存在しないため、実装済みの `setMediaTypes`/`setIsFavorite` を利用する
 * （is_favoriteは真偽値なのでトグルは `setIsFavorite(!filters.isFavorite)` で表現する）。
 */
import type { ItemStatus } from '../items/types';
import { useItemFilters } from './useItemFilters';
import { SearchBox } from './SearchBox';

/**
 * タグ一覧: 本タスクの範囲にはタグ取得APIを呼ぶ実装は含まれない（TASK-0009はチップUIのみが対象）。
 * 01_home.htmlのモック表示（#SF・#泣ける）に準拠したプレースホルダーの固定タグ一覧を用いる。
 * 将来タグ取得APIと連携する際は props で差し替え可能な設計とする。
 */
const PLACEHOLDER_TAGS: { id: string; name: string }[] = [
  { id: 'SF', name: 'SF' },
  { id: '泣ける', name: '泣ける' },
];

const STATUS_CHIPS: { status: ItemStatus; label: string }[] = [
  { status: 'in_progress', label: '視聴中' },
  { status: 'completed', label: '読了/視聴済' },
  { status: 'not_started', label: '未着手' },
];

interface FilterBarProps {
  tags?: { id: string; name: string }[];
}

export function FilterBar({ tags = PLACEHOLDER_TAGS }: FilterBarProps) {
  const { filters, setMediaTypes, toggleTag, setIsFavorite, toggleStatus, setTitle } =
    useItemFilters();

  const isAllActive =
    filters.mediaTypes.length === 0 &&
    filters.tagIds.length === 0 &&
    filters.categoryIds.length === 0 &&
    !filters.isFavorite &&
    filters.statuses.length === 0;

  const handleClearAll = () => {
    setMediaTypes([]);
    setIsFavorite(false);
    filters.statuses.forEach((status) => toggleStatus(status));
    filters.tagIds.forEach((tagId) => toggleTag(tagId));
  };

  return (
    <div className="filter-bar" data-testid="filter-bar">
      <div
        className={`chip${isAllActive ? ' active' : ''}`}
        role="button"
        tabIndex={0}
        aria-pressed={isAllActive}
        onClick={handleClearAll}
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') handleClearAll();
        }}
      >
        すべて
      </div>

      <div
        className={`chip${filters.isFavorite ? ' active' : ''}`}
        role="button"
        tabIndex={0}
        aria-pressed={filters.isFavorite}
        onClick={() => setIsFavorite(!filters.isFavorite)}
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') setIsFavorite(!filters.isFavorite);
        }}
      >
        ⭐ お気に入り
      </div>

      {STATUS_CHIPS.map(({ status, label }) => {
        const active = filters.statuses.includes(status);
        return (
          <div
            key={status}
            className={`chip${active ? ' active' : ''}`}
            role="button"
            tabIndex={0}
            aria-pressed={active}
            onClick={() => toggleStatus(status)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') toggleStatus(status);
            }}
          >
            {label}
          </div>
        );
      })}

      {tags.map((tag) => {
        const active = filters.tagIds.includes(tag.id);
        return (
          <div
            key={tag.id}
            className={`chip${active ? ' active' : ''}`}
            role="button"
            tabIndex={0}
            aria-pressed={active}
            onClick={() => toggleTag(tag.id)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') toggleTag(tag.id);
            }}
          >
            #{tag.name}
          </div>
        );
      })}

      <SearchBox value={filters.title} onChange={setTitle} />
    </div>
  );
}
