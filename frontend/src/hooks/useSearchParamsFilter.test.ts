import { describe, expect, it } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { createElement } from 'react'
import { useSearchParamsFilter } from './useSearchParamsFilter'

function wrapper(initialEntries: string[] = ['/']) {
  return ({ children }: { children: React.ReactNode }) =>
    createElement(MemoryRouter, { initialEntries }, children)
}

describe('useSearchParamsFilter', () => {
  // ===== 正常系 =====

  it('TC-SPF-N-01: 空のURLで全フィールドがundefinedになる', () => {
    const { result } = renderHook(() => useSearchParamsFilter(), {
      wrapper: wrapper(['/'])
    })

    expect(result.current.filters.mediaType).toBeUndefined()
    expect(result.current.filters.tagId).toBeUndefined()
    expect(result.current.filters.categoryId).toBeUndefined()
    expect(result.current.filters.isFavorite).toBeUndefined()
    expect(result.current.filters.status).toBeUndefined()
    expect(result.current.filters.page).toBeUndefined()
    expect(result.current.filters.limit).toBeUndefined()
  })

  it('TC-SPF-N-02: URLクエリ ?media_type=anime&page=2 からfiltersが正しく読み取れる', () => {
    const { result } = renderHook(() => useSearchParamsFilter(), {
      wrapper: wrapper(['/?media_type=anime&page=2'])
    })

    expect(result.current.filters.mediaType).toBe('anime')
    expect(result.current.filters.page).toBe(2)
  })

  it('TC-SPF-N-03: setFilters({ mediaType: "movie" }) でURLが media_type=movie に更新される', () => {
    const { result } = renderHook(() => useSearchParamsFilter(), {
      wrapper: wrapper(['/'])
    })

    act(() => {
      result.current.setFilters({ mediaType: 'movie' })
    })

    expect(result.current.filters.mediaType).toBe('movie')
  })

  it('TC-SPF-N-04: 全フィールドをURLから正しく読み取れる', () => {
    const { result } = renderHook(() => useSearchParamsFilter(), {
      wrapper: wrapper(['/?media_type=manga&tag_id=t1&category_id=c1&favorite=true&status=completed&page=3&limit=20'])
    })

    expect(result.current.filters.mediaType).toBe('manga')
    expect(result.current.filters.tagId).toBe('t1')
    expect(result.current.filters.categoryId).toBe('c1')
    expect(result.current.filters.isFavorite).toBe(true)
    expect(result.current.filters.status).toBe('completed')
    expect(result.current.filters.page).toBe(3)
    expect(result.current.filters.limit).toBe(20)
  })

  it('TC-SPF-N-05: setFilters で値をundefinedにするとURLパラメータが削除される', () => {
    const { result } = renderHook(() => useSearchParamsFilter(), {
      wrapper: wrapper(['/?media_type=anime'])
    })

    act(() => {
      result.current.setFilters({ mediaType: undefined })
    })

    expect(result.current.filters.mediaType).toBeUndefined()
  })

  it('TC-SPF-N-06: setFilters で複数フィールドを一度に更新できる', () => {
    const { result } = renderHook(() => useSearchParamsFilter(), {
      wrapper: wrapper(['/'])
    })

    act(() => {
      result.current.setFilters({ mediaType: 'anime', page: 5, status: 'in_progress' })
    })

    expect(result.current.filters.mediaType).toBe('anime')
    expect(result.current.filters.page).toBe(5)
    expect(result.current.filters.status).toBe('in_progress')
  })

  // ===== 境界値 =====

  it('TC-SPF-B-01: favorite=false はisFavorite=undefinedとして扱われる（falseは未フィルタと同義）', () => {
    const { result } = renderHook(() => useSearchParamsFilter(), {
      wrapper: wrapper(['/?favorite=false'])
    })

    expect(result.current.filters.isFavorite).toBeUndefined()
  })

  it('TC-SPF-B-02: page/limitが数値に正しく変換される', () => {
    const { result } = renderHook(() => useSearchParamsFilter(), {
      wrapper: wrapper(['/?page=10&limit=50'])
    })

    expect(result.current.filters.page).toBe(10)
    expect(typeof result.current.filters.page).toBe('number')
    expect(result.current.filters.limit).toBe(50)
    expect(typeof result.current.filters.limit).toBe('number')
  })

  it('TC-SPF-B-03: 既存のパラメータを保持したままsetFiltersで一部だけ更新できる', () => {
    const { result } = renderHook(() => useSearchParamsFilter(), {
      wrapper: wrapper(['/?media_type=anime&page=1'])
    })

    act(() => {
      result.current.setFilters({ page: 2 })
    })

    expect(result.current.filters.mediaType).toBe('anime')
    expect(result.current.filters.page).toBe(2)
  })
})
