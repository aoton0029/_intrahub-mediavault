import { useMemo } from 'react'
import { useItemsInfiniteQuery } from '@/features/items/hooks'
import { useItemFilters } from '@/features/filters/useItemFilters'
import { FilterBar } from '@/features/filters/FilterBar'
import { MediaCard } from '@/components/MediaCard'
import { useInfiniteScrollSentinel } from './useInfiniteScrollSentinel'
import { ErrorState } from './ErrorState'
import { EmptyState } from './EmptyState'
import type { Item } from '@/features/items/types'

/**
 * 全体一覧画面（HomePage）
 *
 * TASK-0011: useItemFilters(TASK-0007)とuseItemsInfiniteQuery(TASK-0006)を統合し、
 * FilterBar(TASK-0009)・MediaCard(TASK-0008)で構成するカードグリッドを表示する。
 * 01_home.htmlの`.app-shell`/`.main`/`.titlebar`/`.content`/`.card-grid`構造に準拠する。
 *
 * ローディング/エラー/空状態の具体的なUIはTASK-0013で実装するため、本タスクでは
 * isLoading/isError/data.length === 0 の分岐箇所のみ用意する。
 * 無限スクロールはTASK-0012で実装するため、本タスクでは取得済み全ページのフラット化のみ行う。
 */
export default function HomePage() {
  const { filters } = useItemFilters()
  const { data, isLoading, isError, hasNextPage, isFetchingNextPage, fetchNextPage, refetch } =
    useItemsInfiniteQuery(filters)

  const items: Item[] = useMemo(
    () => data?.pages.flatMap((page) => page.data) ?? [],
    [data],
  )

  const sentinelRef = useInfiniteScrollSentinel({
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage,
  })

  return (
    <div className="app-shell">
      <aside className="sidebar" />
      <main className="main">
        <div className="titlebar">
          <h1>全体一覧</h1>
          <a href="/search-add" className="btn btn-accent">
            ＋ 作品を追加
          </a>
        </div>

        <div className="content">
          <FilterBar />

          {isLoading && <p role="status">読み込み中...</p>}
          {isError && <ErrorState onRetry={() => void refetch()} />}
          {!isLoading && !isError && items.length === 0 && <EmptyState />}

          {!isLoading && !isError && items.length > 0 && (
            <div className="card-grid">
              {items.map((item) => (
                <MediaCard key={item.id} item={item} />
              ))}
              <div ref={sentinelRef} aria-hidden="true" />
              {isFetchingNextPage && <p role="status" aria-live="polite">追加読み込み中...</p>}
            </div>
          )}
        </div>
      </main>
    </div>
  )
}
