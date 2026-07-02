import { useNavigate } from 'react-router-dom'
import { useItemsQuery } from '@/api/items'
import { useSearchParamsFilter } from '@/hooks/useSearchParamsFilter'
import { FilterBar } from '@/components/common/FilterBar'
import { MediaCard } from '@/components/common/MediaCard'
import { EmptyState } from '@/components/common/EmptyState'
import { Button } from '@/components/ui/button'
import type { ItemListFilters } from '@/types'

export default function PaperListPage() {
  const navigate = useNavigate()
  const { filters, setFilters } = useSearchParamsFilter()

  const fixedFilters: ItemListFilters = { ...filters, mediaType: 'paper' }
  const { data, isLoading, isError, refetch } = useItemsQuery(fixedFilters)

  const items = data?.data ?? []
  const pagination = data?.pagination

  if (isLoading) {
    return (
      <div data-testid="skeleton-grid" className="grid grid-cols-2 gap-4 p-4 md:grid-cols-4 lg:grid-cols-6">
        {Array.from({ length: 12 }).map((_, i) => (
          <div key={i} className="aspect-[2/3] animate-pulse rounded-lg bg-muted" />
        ))}
      </div>
    )
  }

  if (isError) {
    return (
      <div className="flex flex-col items-center gap-4 p-8">
        <p className="text-sm text-destructive">データの取得に失敗しました</p>
        <Button type="button" onClick={() => refetch?.()}>リトライ</Button>
      </div>
    )
  }

  const currentPage = pagination?.page ?? fixedFilters.page ?? 1
  const totalPages = pagination ? Math.ceil(pagination.total / pagination.limit) : 1

  return (
    <div className="flex flex-col gap-4 p-4">
      <div className="flex items-center justify-end">
        <Button type="button" size="sm" onClick={() => navigate('/search/paper')}>
          + 追加する
        </Button>
      </div>

      <FilterBar
        filters={fixedFilters}
        onChange={next => setFilters({ ...next, mediaType: 'paper' })}
        tagOptions={[]}
        categoryOptions={[]}
        mediaTypeOptions={['paper']}
      />

      {items.length === 0 ? (
        <EmptyState
          message="コレクションがありません"
          actionLabel="+ 追加する"
          onAction={() => navigate('/search/paper')}
        />
      ) : (
        <>
          <div className="grid grid-cols-2 gap-4 md:grid-cols-4 lg:grid-cols-6">
            {items.map(item => (
              <MediaCard
                key={item.id}
                item={item}
                onClick={clicked => navigate(`/items/${clicked.id}`)}
              />
            ))}
          </div>

          {pagination && (
            <div className="flex items-center justify-center gap-2">
              <Button
                type="button"
                disabled={currentPage <= 1}
                onClick={() => setFilters({ page: currentPage - 1 })}
                aria-label="前のページ"
              >
                前のページ
              </Button>
              <span className="text-sm">{currentPage} / {totalPages}</span>
              <Button
                type="button"
                disabled={currentPage >= totalPages}
                onClick={() => setFilters({ page: currentPage + 1 })}
                aria-label="次のページ"
              >
                次のページ
              </Button>
            </div>
          )}
        </>
      )}
    </div>
  )
}
