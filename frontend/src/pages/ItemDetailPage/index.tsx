import { useParams } from 'react-router-dom';
import { useSetTitlebar } from '@/components/layout/useTitlebar';
import { useItemDetailQuery } from '@/features/items/api-detail';
import { mediaTypeLabels } from '@/features/items/types';
import { DetailRail } from '@/features/items/components/detail/DetailRail';
import { GroupEpisodeList } from '@/features/items/components/detail/GroupEpisodeList';
import { StaffList } from '@/features/items/components/detail/StaffList';
import { RelatedItemsList } from '@/features/items/components/detail/RelatedItemsList';
import { ResourceTabs } from '@/features/items/components/detail/ResourceTabs';
import { ErrorState } from '@/features/items/components/ErrorState';

export function ItemDetailPage() {
  const { id } = useParams<{ id: string }>();
  const itemId = id ?? '';
  const query = useItemDetailQuery(itemId);
  const item = query.data;

  useSetTitlebar({
    title: item?.title ?? '',
    breadcrumb: item ? mediaTypeLabels[item.media_type] : undefined,
    action: item ? (
      <a
        href={`/items/${item.id}/edit`}
        className="rounded-app border border-accent bg-accent px-3.5 py-1.5 text-sm font-medium text-white hover:bg-accent-strong"
      >
        編集する
      </a>
    ) : undefined,
  });

  if (query.isLoading) {
    return <div className="py-16 text-center text-text-faint">読み込み中…</div>;
  }

  if (query.isError || !item) {
    return <ErrorState onRetry={() => query.refetch()} />;
  }

  return (
    <div className="grid grid-cols-[280px_1fr] items-start gap-8">
      <DetailRail item={item} />
      <div className="min-w-0">
        {item.description && (
          <div className="mb-7 max-w-[680px]">
            <h3 className="mb-2.5 border-b border-border-soft pb-1.5 text-xs uppercase tracking-[0.05em] text-text-faint">
              概要
            </h3>
            <p className="leading-[1.7] text-text-primary">{item.description}</p>
          </div>
        )}
        <GroupEpisodeList itemId={item.id} />
        <StaffList itemId={item.id} />
        <RelatedItemsList itemId={item.id} />
        <ResourceTabs itemId={item.id} />
      </div>
    </div>
  );
}
