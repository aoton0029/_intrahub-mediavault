import { useSearchParams } from 'react-router-dom'

/**
 * フィルタ状態とURLクエリの同期フックのプレースホルダー。
 * 本実装（media_type/タグ/カテゴリ/favorite/status等の複合フィルタ）は TASK-0007 で行う。
 */
export interface ItemFilterState {
  [key: string]: string | string[] | undefined
}

export function useSearchParamsFilter() {
  const [searchParams, setSearchParams] = useSearchParams()

  const filters: ItemFilterState = Object.fromEntries(searchParams.entries())

  const setFilters = (next: ItemFilterState) => {
    const params = new URLSearchParams()
    for (const [key, value] of Object.entries(next)) {
      if (value === undefined) continue
      if (Array.isArray(value)) {
        value.forEach((v) => params.append(key, v))
      } else {
        params.set(key, value)
      }
    }
    setSearchParams(params)
  }

  return { filters, setFilters }
}
