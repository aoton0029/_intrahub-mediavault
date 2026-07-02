import { describe, expect, it, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import GeneralListPage from './GeneralListPage'
import type { Item } from '@/types'

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

const mockSetFilters = vi.fn()

function renderPage() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter initialEntries={['/collections/general']}>
        <GeneralListPage />
      </MemoryRouter>
    </QueryClientProvider>
  )
}

beforeEach(() => {
  vi.mocked(useSearchParamsFilter).mockReturnValue({ filters: {}, setFilters: mockSetFilters })
  vi.mocked(useItemsQuery).mockReturnValue({
    data: { data: [], pagination: undefined },
    isLoading: false,
    isError: false,
  } as unknown as ReturnType<typeof useItemsQuery>)
  mockNavigate.mockReset()
  mockSetFilters.mockReset()
})

describe('GeneralListPage - media_typeフィルタ', () => {
  it('TC-GL-01: media_typeセレクトにanime/movie/drama/manga/novel/gameの6種が表示される', () => {
    renderPage()

    const select = screen.getByRole('combobox', { name: /メディアタイプ/i })
    const options = Array.from(select.querySelectorAll('option')).map(o => o.value).filter(Boolean)

    expect(options).toContain('anime')
    expect(options).toContain('movie')
    expect(options).toContain('drama')
    expect(options).toContain('manga')
    expect(options).toContain('novel')
    expect(options).toContain('game')
  })

  it('TC-GL-02: academic_bookとpaperは選択肢に含まれない', () => {
    renderPage()

    const select = screen.getByRole('combobox', { name: /メディアタイプ/i })
    const options = Array.from(select.querySelectorAll('option')).map(o => o.value)

    expect(options).not.toContain('academic_book')
    expect(options).not.toContain('paper')
  })

  it('TC-GL-03: mangaを選択するとsetFiltersにmediaType:mangaが渡される', () => {
    renderPage()

    const select = screen.getByRole('combobox', { name: /メディアタイプ/i })
    fireEvent.change(select, { target: { value: 'manga' } })

    expect(mockSetFilters).toHaveBeenCalledWith(expect.objectContaining({ mediaType: 'manga' }))
  })
})

describe('GeneralListPage - 一覧表示', () => {
  it('アイテムが3件あれば3枚のMediaCardが表示される', () => {
    vi.mocked(useItemsQuery).mockReturnValue({
      data: { data: [makeItem('1'), makeItem('2'), makeItem('3')], pagination: undefined },
      isLoading: false,
      isError: false,
    } as unknown as ReturnType<typeof useItemsQuery>)

    renderPage()

    expect(screen.getAllByTestId('media-card')).toHaveLength(3)
  })

  it('アイテムが0件のときEmptyStateが表示される', () => {
    renderPage()

    expect(screen.getByTestId('empty-state')).toBeInTheDocument()
  })

  it('isLoading=trueのときスケルトンが表示される', () => {
    vi.mocked(useItemsQuery).mockReturnValue({
      data: undefined,
      isLoading: true,
      isError: false,
    } as unknown as ReturnType<typeof useItemsQuery>)

    renderPage()

    expect(screen.getByTestId('skeleton-grid')).toBeInTheDocument()
  })

  it('isError=trueのときリトライボタンが表示される', () => {
    vi.mocked(useItemsQuery).mockReturnValue({
      data: undefined,
      isLoading: false,
      isError: true,
      refetch: vi.fn(),
    } as unknown as ReturnType<typeof useItemsQuery>)

    renderPage()

    expect(screen.getByRole('button', { name: /リトライ/i })).toBeInTheDocument()
  })
})
