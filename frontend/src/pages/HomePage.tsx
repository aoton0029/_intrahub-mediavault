import { useNavigate } from 'react-router-dom'
import { useItemsQuery } from '@/api/items'
import { useSearchParamsFilter } from '@/hooks/useSearchParamsFilter'
import { FilterBar } from '@/components/common/FilterBar'
import { MediaCard } from '@/components/common/MediaCard'
import { EmptyState } from '@/components/common/EmptyState'
import { Button } from '@/components/ui/button'

export default function HomePage() {
  const navigate = useNavigate()
  const { filters, setFilters } = useSearchParamsFilter()
  const { data, isLoading, isError, refetch } = useItemsQuery(filters)

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

  const currentPage = pagination?.page ?? filters.page ?? 1
  const totalPages = pagination ? Math.ceil(pagination.total / pagination.limit) : 1

  return (
    <div className="flex flex-col gap-4">
      <div className="titlebar">
        <h1>全体一覧</h1>
        <Button type="button" variant="accent" onClick={() => navigate('/search/general')}>
          ＋ 作品を追加
        </Button>
      </div>

      <div className="flex flex-col gap-4 px-4 pb-4">
        <FilterBar
          filters={filters}
          onChange={next => setFilters(next)}
          tagOptions={[]}
          categoryOptions={[]}
        />

        {items.length === 0 ? (
          <EmptyState
            message="コレクションがありません"
            actionLabel="+ 追加する"
            onAction={() => navigate('/search/general')}
          />
        ) : (
          <>
            <div className="card-grid">
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
    </div>
  )
}
