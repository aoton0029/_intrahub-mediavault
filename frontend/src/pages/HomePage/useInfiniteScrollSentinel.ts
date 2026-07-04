import { useCallback, useEffect, useRef } from 'react'

export interface UseInfiniteScrollSentinelOptions {
  hasNextPage: boolean | undefined
  isFetchingNextPage: boolean
  fetchNextPage: () => void
}

/**
 * IntersectionObserverでsentinel要素の可視化を検知し、fetchNextPage()を呼び出すフック。
 *
 * TASK-0012: `.card-grid`末尾のsentinelを監視する。hasNextPage:falseの場合は呼び出さない(EDGE-102)。
 * isFetchingNextPage中は再フェッチ完了までUI側でも二重発火を防止する。
 */
export function useInfiniteScrollSentinel({
  hasNextPage,
  isFetchingNextPage,
  fetchNextPage,
}: UseInfiniteScrollSentinelOptions): (node: Element | null) => void {
  const observerRef = useRef<IntersectionObserver | null>(null)
  const hasNextPageRef = useRef(hasNextPage)
  const isFetchingNextPageRef = useRef(isFetchingNextPage)
  const fetchNextPageRef = useRef(fetchNextPage)

  hasNextPageRef.current = hasNextPage
  isFetchingNextPageRef.current = isFetchingNextPage
  fetchNextPageRef.current = fetchNextPage

  const setSentinel = useCallback((node: Element | null) => {
    observerRef.current?.disconnect()
    observerRef.current = null

    if (!node) return

    const observer = new IntersectionObserver(([entry]) => {
      if (
        entry.isIntersecting &&
        hasNextPageRef.current &&
        !isFetchingNextPageRef.current
      ) {
        fetchNextPageRef.current()
      }
    })

    observer.observe(node)
    observerRef.current = observer
  }, [])

  useEffect(() => {
    return () => {
      observerRef.current?.disconnect()
    }
  }, [])

  return setSentinel
}
