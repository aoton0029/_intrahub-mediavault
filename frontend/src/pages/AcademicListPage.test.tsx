import { describe, expect, it, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import AcademicListPage from './AcademicListPage'
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
    mediaType: 'academic_book',
    status: 'not_started',
    isFavorite: false,
    source: 'manual',
    createdAt: '2026-01-01T00:00:00Z',
    updatedAt: '2026-01-01T00:00:00Z',
    details: {},
  }
}

const mockSetFilters = vi.fn()

function renderPage() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter initialEntries={['/collections/academic']}>
        <AcademicListPage />
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

describe('AcademicListPage - media_type固定フィルタ', () => {
  it('TC-AL-01: useItemsQueryがmediaType=academic_bookで呼ばれる', () => {
    renderPage()

    expect(vi.mocked(useItemsQuery)).toHaveBeenCalledWith(
      expect.objectContaining({ mediaType: 'academic_book' })
    )
  })

  it('TC-AL-02: media_typeセレクトにacademic_book以外の選択肢が含まれない', () => {
    renderPage()

    const select = screen.getByRole('combobox', { name: /メディアタイプ/i })
    const options = Array.from(select.querySelectorAll('option')).map(o => o.value).filter(Boolean)

    expect(options).not.toContain('anime')
    expect(options).not.toContain('movie')
    expect(options).not.toContain('paper')
    expect(options.every(v => v === 'academic_book')).toBe(true)
  })

  it('TC-AL-03: お気に入りフィルタONでacademic_bookが維持されたままsetFiltersが呼ばれる', () => {
    renderPage()

    const checkbox = screen.getByRole('checkbox', { name: /お気に入り/i })
    fireEvent.click(checkbox)

    expect(mockSetFilters).toHaveBeenCalledWith(
      expect.objectContaining({ mediaType: 'academic_book', isFavorite: true })
    )
  })
})

describe('AcademicListPage - 一覧表示', () => {
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
