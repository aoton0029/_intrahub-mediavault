import { useMemo } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useSetTitlebar } from '@/components/layout/useTitlebar';
import {
  fetchItems,
  useCategoriesQuery,
  useItemStatusCountsQuery,
  useMediaTypeCountsQuery,
  useTagsQuery,
} from '@/features/items/api';
import { MediaCard } from '@/features/items/components/MediaCard';
import { mediaTypeLabels, statusLabels } from '@/features/items/types';

const TOP_TAGS_LIMIT = 8;
const TOP_CATEGORIES_LIMIT = 6;

function StatCard({
  value,
  label,
  isFavorite,
}: {
  value: number | undefined;
  label: string;
  isFavorite?: boolean;
}) {
  return (
    <div className="rounded-app border border-border-soft bg-bg-surface p-3.5 px-4 transition-[border-color,transform] hover:-translate-y-0.5 hover:border-accent">
      <div
        className={`font-mono text-[26px] leading-tight font-semibold ${isFavorite ? 'text-favorite' : 'text-text-primary'}`}
      >
        {value ?? '—'}
      </div>
      <div className="mt-1 text-xs text-text-faint">{label}</div>
    </div>
  );
}

function SectionHeading({ children, seeAllHref }: { children: React.ReactNode; seeAllHref?: string }) {
  return (
    <div className="mt-8 mb-3 flex items-center justify-between border-b border-border-soft pb-1.5 text-xs tracking-wide text-text-faint uppercase first:mt-0">
      {children}
      {seeAllHref && (
        <a href={seeAllHref} className="text-xs text-accent-strong normal-case">
          すべて見る →
        </a>
      )}
    </div>
  );
}

export function HomePage() {
  useSetTitlebar({ title: 'ホーム' });

  const mediaTypeCountsQuery = useMediaTypeCountsQuery();
  const statusCountsQuery = useItemStatusCountsQuery();
  const tagsQuery = useTagsQuery();
  const categoriesQuery = useCategoriesQuery();

  const recentQuery = useQuery({
    queryKey: ['items', 'recent'],
    queryFn: () => fetchItems({ limit: 6 }),
  });

  const inProgressQuery = useQuery({
    queryKey: ['items', 'in-progress'],
    queryFn: () => fetchItems({ status: 'in_progress', limit: 6 }),
  });

  const topTags = useMemo(
    () => [...(tagsQuery.data ?? [])].sort((a, b) => b.item_count - a.item_count).slice(0, TOP_TAGS_LIMIT),
    [tagsQuery.data],
  );

  const topCategories = useMemo(
    () =>
      [...(categoriesQuery.data ?? [])]
        .sort((a, b) => b.item_count - a.item_count)
        .slice(0, TOP_CATEGORIES_LIMIT),
    [categoriesQuery.data],
  );

  return (
    <>
      <div className="grid grid-cols-[repeat(auto-fit,minmax(150px,1fr))] gap-3">
        <StatCard value={mediaTypeCountsQuery.data?.total} label="総件数" />
        <StatCard value={statusCountsQuery.data?.in_progress} label="進行中" />
        <StatCard value={statusCountsQuery.data?.completed} label="読了・視聴済み" />
        <StatCard value={statusCountsQuery.data?.favorite} label="❤ お気に入り" isFavorite />
      </div>

      <SectionHeading seeAllHref="/general-media">最近追加した作品</SectionHeading>
      {recentQuery.isLoading && <div className="py-8 text-center text-text-faint">読み込み中…</div>}
      {recentQuery.isError && (
        <div className="py-8 text-center text-text-faint">最近追加した作品の取得に失敗しました。</div>
      )}
      {recentQuery.data && recentQuery.data.data.length > 0 && (
        <div className="grid grid-cols-[repeat(auto-fill,minmax(168px,1fr))] gap-4">
          {recentQuery.data.data.map((item) => (
            <MediaCard key={item.id} item={item} />
          ))}
        </div>
      )}

      <SectionHeading seeAllHref="/general-media">進行中</SectionHeading>
      {inProgressQuery.isLoading && <div className="py-8 text-center text-text-faint">読み込み中…</div>}
      {inProgressQuery.isError && (
        <div className="py-8 text-center text-text-faint">進行中の作品の取得に失敗しました。</div>
      )}
      {inProgressQuery.data && inProgressQuery.data.data.length > 0 && (
        <div className="grid grid-cols-[repeat(auto-fill,minmax(168px,1fr))] gap-4">
          {inProgressQuery.data.data.map((item) => (
            <div
              key={item.id}
              className="overflow-hidden rounded-app border border-border-soft bg-bg-surface transition-[border-color,transform] hover:-translate-y-0.5 hover:border-accent"
            >
              <div className="relative flex aspect-[2/3] items-end justify-end bg-[linear-gradient(160deg,#33304a,#232323)] p-1.5">
                <span className="absolute top-1.5 left-1.5 rounded bg-black/55 px-1.5 py-0.5 font-mono text-[10px] tracking-wide text-text-muted">
                  {mediaTypeLabels[item.media_type]}
                </span>
              </div>
              <div className="p-2.5 pt-2.5 pb-3">
                <p className="m-0 mb-1 line-clamp-2 font-display text-[13.5px] font-semibold text-text-primary">
                  {item.title}
                </p>
                {item.status && (
                  <div className="mt-1.5 flex flex-wrap gap-1.5">
                    <span className="inline-flex items-center rounded bg-accent-soft px-1.5 py-0.5 font-mono text-[11px] text-status-progress">
                      #{statusLabels[item.status]}
                    </span>
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>
      )}

      <SectionHeading>タグ・カテゴリ</SectionHeading>
      <div className="grid grid-cols-2 gap-6">
        <div>
          <div className="mb-2.5 text-xs text-text-faint">よく使うタグ</div>
          <div className="flex flex-wrap gap-1.5">
            {topTags.map((tag) => (
              <a
                key={tag.id}
                href="/tags"
                className="inline-flex items-center rounded bg-accent-soft px-1.5 py-0.5 font-mono text-[11px] text-accent-strong"
              >
                #{tag.name}
                <span className="ml-1 opacity-60">{tag.item_count}</span>
              </a>
            ))}
          </div>
        </div>
        <div>
          <div className="mb-2.5 text-xs text-text-faint">カテゴリ</div>
          <div className="flex flex-wrap gap-1.5">
            {topCategories.map((category) => (
              <a
                key={category.id}
                href="/categories"
                className="inline-flex items-center rounded bg-accent-soft px-1.5 py-0.5 font-mono text-[11px] text-accent-strong"
              >
                #{category.name}
                <span className="ml-1 opacity-60">{category.item_count}</span>
              </a>
            ))}
          </div>
        </div>
      </div>
    </>
  );
}
