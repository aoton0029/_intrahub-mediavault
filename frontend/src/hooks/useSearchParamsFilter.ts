import { useSearchParams } from 'react-router-dom'
import type { ItemListFilters, MediaType, ItemStatus } from '@/types'

const KEY_MAP: Record<keyof ItemListFilters, string> = {
  mediaType: 'media_type',
  tagId: 'tag_id',
  categoryId: 'category_id',
  isFavorite: 'favorite',
  status: 'status',
  page: 'page',
  limit: 'limit',
}

function toQueryKey(field: keyof ItemListFilters): string {
  return KEY_MAP[field]
}

export function useSearchParamsFilter() {
  const [searchParams, setSearchParams] = useSearchParams()

  const filters: ItemListFilters = {
    mediaType: (searchParams.get('media_type') as MediaType) ?? undefined,
    tagId: searchParams.get('tag_id') ?? undefined,
    categoryId: searchParams.get('category_id') ?? undefined,
    isFavorite: searchParams.get('favorite') === 'true' ? true : undefined,
    status: (searchParams.get('status') as ItemStatus) ?? undefined,
    page: searchParams.get('page') ? Number(searchParams.get('page')) : undefined,
    limit: searchParams.get('limit') ? Number(searchParams.get('limit')) : undefined,
  }

  const setFilters = (next: Partial<ItemListFilters>) => {
    setSearchParams(prev => {
      const updated = new URLSearchParams(prev)
      for (const [key, value] of Object.entries(next)) {
        const paramKey = toQueryKey(key as keyof ItemListFilters)
        if (value === undefined || value === null || value === '') {
          updated.delete(paramKey)
        } else {
          updated.set(paramKey, String(value))
        }
      }
      return updated
    })
  }

  return { filters, setFilters }
}
