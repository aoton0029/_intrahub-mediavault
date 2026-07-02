import { describe, expect, it, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import PaperListPage from './PaperListPage'
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
    mediaType: 'paper',
    status: 'not_started',
    isFavorite: false,
    source: 'manual',
    createdAt: '2026-01-01T00:00:00Z',
    updatedAt: '2026-01-01T00:00:00Z',
    details: { authorList: [] },
  }
}

const mockSetFilters = vi.fn()

function renderPage() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter initialEntries={['/collections/paper']}>
        <PaperListPage />
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

describe('PaperListPage - media_type固定フィルタ', () => {
  it('TC-PL-01: useItemsQueryがmediaType=paperで呼ばれる', () => {
    renderPage()

    expect(vi.mocked(useItemsQuery)).toHaveBeenCalledWith(
      expect.objectContaining({ mediaType: 'paper' })
    )
  })

  it('TC-PL-02: media_typeセレクトにpaper以外の選択肢が含まれない', () => {
    renderPage()

    const select = screen.getByRole('combobox', { name: /メディアタイプ/i })
    const options = Array.from(select.querySelectorAll('option')).map(o => o.value).filter(Boolean)

    expect(options).not.toContain('anime')
    expect(options).not.toContain('movie')
    expect(options).not.toContain('academic_book')
    expect(options.every(v => v === 'paper')).toBe(true)
  })

  it('TC-PL-03: お気に入りフィルタONでpaperが維持されたままsetFiltersが呼ばれる', () => {
    renderPage()

    const checkbox = screen.getByRole('checkbox', { name: /お気に入り/i })
    fireEvent.click(checkbox)

    expect(mockSetFilters).toHaveBeenCalledWith(
      expect.objectContaining({ mediaType: 'paper', isFavorite: true })
    )
  })
})

describe('PaperListPage - 一覧表示', () => {
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
