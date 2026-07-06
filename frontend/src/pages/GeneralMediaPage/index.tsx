import { useMemo, useState } from 'react';
import { useSetTitlebar } from '@/components/layout/useTitlebar';
import { useInfiniteItemsQuery } from '@/features/items/api';
import { useItemFilters } from '@/features/items/hooks/useItemFilters';
import { useInfiniteScrollSentinel } from '@/features/items/hooks/useInfiniteScrollSentinel';
import { MediaCard } from '@/features/items/components/MediaCard';
import { FilterBar } from '@/features/items/components/FilterBar';
import { SearchBox } from '@/features/items/components/SearchBox';
import { EmptyState } from '@/features/items/components/EmptyState';
import { ErrorState } from '@/features/items/components/ErrorState';
import type { Item } from '@/features/items/types';

type SortKey = 'created_at' | 'title' | 'rating';

export function GeneralMediaPage() {
  const { filters, toggleFavorite, setMediaType, toggleTag, toggleCategory, setTitle } =
    useItemFilters();
  const [sortKey, setSortKey] = useState<SortKey>('created_at');

  const query = useInfiniteItemsQuery(filters);

  const items: Item[] = useMemo(
    () => query.data?.pages.flatMap((page) => page.data) ?? [],
    [query.data],
  );

  // 【注意】: バックエンドにsortクエリパラメータが無いため、この並び替えは
  // 「現在ロード済みのページ」のみを対象としたクライアント側ソートであり、
  // 全件を対象とした並び替えではない（無限スクロールで未取得のページには適用されない）。
  const sortedItems = useMemo(() => {
    const copy = [...items];
    switch (sortKey) {
      case 'title':
        return copy.sort((a, b) => a.title.localeCompare(b.title, 'ja'));
      case 'rating':
        return copy.sort((a, b) => (b.rating ?? 0) - (a.rating ?? 0));
      default:
        return copy;
    }
  }, [items, sortKey]);

  const sentinelRef = useInfiniteScrollSentinel(() => {
    if (query.hasNextPage && !query.isFetchingNextPage) {
      query.fetchNextPage();
    }
  }, !query.isLoading && !query.isError);

  useSetTitlebar({
    title: '一般メディア',
    action: (
      <a
        href="/search-add"
        className="rounded-app border border-accent bg-accent px-3.5 py-1.5 text-sm font-medium text-white hover:bg-accent-strong"
      >
        ＋ 作品を追加
      </a>
    ),
  });

  return (
    <>
      <div className="mb-5 flex flex-wrap items-center justify-between gap-3">
        <FilterBar
          filters={filters}
          onToggleFavorite={toggleFavorite}
          onSetMediaType={setMediaType}
          onToggleTag={toggleTag}
          onToggleCategory={toggleCategory}
        />
        <div className="flex items-center gap-2">
          <select
            value={sortKey}
            onChange={(e) => setSortKey(e.target.value as SortKey)}
            className="rounded-app border border-border bg-bg-input px-2.5 py-1.5 text-sm text-text-muted"
          >
            <option value="created_at">追加日順</option>
            <option value="title">タイトル順</option>
            <option value="rating">評価順</option>
          </select>
          <SearchBox value={filters.title ?? ''} onChange={setTitle} />
        </div>
      </div>

      {query.isError && <ErrorState onRetry={() => query.refetch()} />}

      {!query.isError && !query.isLoading && sortedItems.length === 0 && <EmptyState />}

      {!query.isError && sortedItems.length > 0 && (
        <>
          <div className="grid grid-cols-[repeat(auto-fill,minmax(168px,1fr))] gap-4">
            {sortedItems.map((item) => (
              <MediaCard key={item.id} item={item} />
            ))}
          </div>
          <div ref={sentinelRef} className="flex justify-center py-6 text-xs text-text-faint">
            {query.isFetchingNextPage && '読み込み中…'}
          </div>
        </>
      )}

      {query.isLoading && (
        <div className="py-16 text-center text-text-faint">読み込み中…</div>
      )}
    </>
  );
}
