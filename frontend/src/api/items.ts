import { useQuery } from '@tanstack/react-query'

/**
 * items系APIクライアント関数・フックのプレースホルダー。
 * 本実装（一覧取得・フィルタ・ページネーション）は TASK-0003 で行う。
 * TASK-0001 時点では App.test.tsx がモックする対象としての最小限の型・シグネチャのみ定義する。
 */
export interface ItemListItem {
  id: string
  title: string
}

export interface ItemsQueryResult {
  data: {
    data: ItemListItem[]
    pagination?: unknown
  }
  isLoading: boolean
  isError: boolean
}

export function useItemsQuery(): ItemsQueryResult {
  const query = useQuery({
    queryKey: ['items'],
    queryFn: async () => ({ data: [] as ItemListItem[], pagination: undefined }),
  })

  return {
    data: query.data ?? { data: [], pagination: undefined },
    isLoading: query.isLoading,
    isError: query.isError,
  }
}
