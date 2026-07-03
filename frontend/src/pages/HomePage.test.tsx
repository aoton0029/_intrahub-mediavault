import { describe, expect, it, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, within } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import HomePage from './HomePage'
import type { Item, Pagination } from '@/types'

// ===== モック =====

vi.mock('@/api/items', () => ({
  useItemsQuery: vi.fn(),
}))

vi.mock('@/hooks/useSearchParamsFilter', () => ({
  useSearchParamsFilter: vi.fn(),
}))

const mockNavigate = vi.fn()
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual<typeof import('react-router-dom')>('react-router-dom')
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  }
})

import { useItemsQuery } from '@/api/items'
import { useSearchParamsFilter } from '@/hooks/useSearchParamsFilter'

// ===== テストデータ =====

function makeItem(id: string): Item {
  return {
    id,
    title: `Item ${id}`,
    mediaType: 'anime',
    status: 'not_started',
    isFavorite: false,
    source: 'manual',
    createdAt: '2026-01-01T00:00:00Z',
    updatedAt: '2026-01-01T00:00:00Z',
    details: { episodeCount: undefined, seasonCount: undefined, studio: undefined, genreList: [], sourceType: undefined, jikanId: undefined },
  }
}

const mockFilters = {}
const mockSetFilters = vi.fn()

// ===== セットアップ =====

function renderHomePage(initialEntries = ['/']) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter initialEntries={initialEntries}>
        <HomePage />
      </MemoryRouter>
    </QueryClientProvider>
  )
}

beforeEach(() => {
  vi.mocked(useSearchParamsFilter).mockReturnValue({ filters: mockFilters, setFilters: mockSetFilters })
  mockNavigate.mockReset()
  mockSetFilters.mockReset()
})

// ===== 正常系 =====

describe('HomePage - 正常系', () => {
  it('TC-HP-N-01: useItemsQueryが3件のItemを返す → 3件分のMediaCardが表示される', () => {
    vi.mocked(useItemsQuery).mockReturnValue({
      data: { data: [makeItem('1'), makeItem('2'), makeItem('3')], pagination: undefined },
      isLoading: false,
      isError: false,
    } as unknown as ReturnType<typeof useItemsQuery>)

    renderHomePage()

    const cards = screen.getAllByTestId('media-card')
    expect(cards).toHaveLength(3)
  })

  it('TC-HP-N-02: useItemsQueryが0件を返す → EmptyStateと追加画面への導線が表示される', () => {
    vi.mocked(useItemsQuery).mockReturnValue({
      data: { data: [], pagination: undefined },
      isLoading: false,
      isError: false,
    } as unknown as ReturnType<typeof useItemsQuery>)

    renderHomePage()

    const emptyState = screen.getByTestId('empty-state')
    expect(emptyState).toBeInTheDocument()
    expect(within(emptyState).getByRole('button', { name: /追加/i })).toBeInTheDocument()
  })

  it('TC-HP-N-03: FilterBarでmediaType=animeを選択するとsetFiltersが呼ばれURLクエリが反映される', () => {
    vi.mocked(useItemsQuery).mockReturnValue({
      data: { data: [], pagination: undefined },
      isLoading: false,
      isError: false,
    } as unknown as ReturnType<typeof useItemsQuery>)

    renderHomePage()

    const mediaTypeSelect = screen.getByRole('combobox', { name: /メディアタイプ/i })
    fireEvent.change(mediaTypeSelect, { target: { value: 'anime' } })

    expect(mockSetFilters).toHaveBeenCalledWith(expect.objectContaining({ mediaType: 'anime' }))
  })

  it('TC-HP-N-04: pagination={page:1,total:50}のとき「次のページ」ボタン押下でsetFiltersがpage:2で呼ばれる', () => {
    const pagination: Pagination = { page: 1, limit: 20, total: 50 }
    vi.mocked(useItemsQuery).mockReturnValue({
      data: { data: [makeItem('1')], pagination },
      isLoading: false,
      isError: false,
    } as unknown as ReturnType<typeof useItemsQuery>)

    renderHomePage()

    const nextButton = screen.getByRole('button', { name: /次のページ/i })
    fireEvent.click(nextButton)

    expect(mockSetFilters).toHaveBeenCalledWith(expect.objectContaining({ page: 2 }))
  })

  it('TC-HP-N-06: タイトルバーに「＋ 作品を追加」ボタン（accentバリアント）が表示され、クリックで検索追加画面へ遷移する', () => {
    vi.mocked(useItemsQuery).mockReturnValue({
      data: { data: [], pagination: undefined },
      isLoading: false,
      isError: false,
    } as unknown as ReturnType<typeof useItemsQuery>)

    renderHomePage()

    const addButton = screen.getByRole('button', { name: '＋ 作品を追加' })
    expect(addButton).toHaveAttribute('data-variant', 'accent')

    fireEvent.click(addButton)

    expect(mockNavigate).toHaveBeenCalledWith('/search/general')
  })

  it('TC-HP-N-07: MediaCard一覧がcard-gridクラスのコンテナ内に表示される', () => {
    vi.mocked(useItemsQuery).mockReturnValue({
      data: { data: [makeItem('1'), makeItem('2')], pagination: undefined },
      isLoading: false,
      isError: false,
    } as unknown as ReturnType<typeof useItemsQuery>)

    renderHomePage()

    const cards = screen.getAllByTestId('media-card')
    expect(cards[0].closest('.card-grid')).not.toBeNull()
  })

  it('TC-HP-N-05: カードをクリックすると/items/:idへナビゲーションが発生する', () => {
    vi.mocked(useItemsQuery).mockReturnValue({
      data: { data: [makeItem('item-abc')], pagination: undefined },
      isLoading: false,
      isError: false,
    } as unknown as ReturnType<typeof useItemsQuery>)

    renderHomePage()

    const card = screen.getByTestId('media-card')
    fireEvent.click(card)

    expect(mockNavigate).toHaveBeenCalledWith('/items/item-abc')
  })
})

// ===== ローディング・エラー状態 =====

describe('HomePage - ローディング・エラー', () => {
  it('isLoading=trueのときスケルトンが表示される', () => {
    vi.mocked(useItemsQuery).mockReturnValue({
      data: undefined,
      isLoading: true,
      isError: false,
    } as unknown as ReturnType<typeof useItemsQuery>)

    renderHomePage()

    expect(screen.getByTestId('skeleton-grid')).toBeInTheDocument()
  })

  it('isError=trueのときエラーメッセージとリトライボタンが表示される', () => {
    vi.mocked(useItemsQuery).mockReturnValue({
      data: undefined,
      isLoading: false,
      isError: true,
      refetch: vi.fn(),
    } as unknown as ReturnType<typeof useItemsQuery>)

    renderHomePage()

    expect(screen.getByRole('button', { name: /リトライ/i })).toBeInTheDocument()
  })
})
