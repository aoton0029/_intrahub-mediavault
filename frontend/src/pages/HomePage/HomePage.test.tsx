import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { MemoryRouter } from 'react-router-dom'
import HomePage from './index'
import { listItems } from '@/features/items/api'
import type { Item } from '@/features/items/types'

vi.mock('@/features/items/api', () => ({
  listItems: vi.fn(),
}))

const mockedListItems = vi.mocked(listItems)

function makeItem(overrides: Partial<Item> = {}): Item {
  return {
    id: 'item-1',
    media_type: 'anime',
    title: '星屑のシンフォニア',
    status: 'in_progress',
    is_favorite: false,
    cover_image_url: null,
    season_label: null,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    ...overrides,
  }
}

function renderHomePage(initialEntries: string[] = ['/']) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={initialEntries}>
        <HomePage />
      </MemoryRouter>
    </QueryClientProvider>,
  )
}

describe('HomePage', () => {
  beforeEach(() => {
    mockedListItems.mockReset()
  })

  it('fetches items on mount and renders them as MediaCards in the card grid', async () => {
    mockedListItems.mockResolvedValueOnce({
      data: [makeItem({ id: '1', title: '星屑のシンフォニア' }), makeItem({ id: '2', title: '夜明けの境界線' })],
      pagination: { page: 1, limit: 20, total: 2 },
    } as never)

    renderHomePage()

    expect(mockedListItems).toHaveBeenCalled()
    await waitFor(() => {
      expect(screen.getByText('星屑のシンフォニア')).toBeInTheDocument()
      expect(screen.getByText('夜明けの境界線')).toBeInTheDocument()
    })
  })

  it('refetches with updated query params when a filter chip is toggled', async () => {
    mockedListItems.mockResolvedValue({
      data: [makeItem()],
      pagination: { page: 1, limit: 20, total: 1 },
    } as never)

    const user = userEvent.setup()
    renderHomePage()

    await waitFor(() => expect(mockedListItems).toHaveBeenCalledTimes(1))

    const favoriteChip = screen.getByRole('button', { name: /お気に入り/ })
    await user.click(favoriteChip)

    await waitFor(() => expect(mockedListItems).toHaveBeenCalledTimes(2))
    expect(mockedListItems.mock.calls[1][0]).toEqual(
      expect.objectContaining({ is_favorite: true }),
    )
  })

  it('reflects search box input in the refetch query params', async () => {
    mockedListItems.mockResolvedValue({
      data: [makeItem()],
      pagination: { page: 1, limit: 20, total: 1 },
    } as never)

    const user = userEvent.setup()
    renderHomePage()

    await waitFor(() => expect(mockedListItems).toHaveBeenCalledTimes(1))

    const input = screen.getByLabelText('タイトルで検索')
    await user.type(input, 'テスト')

    await waitFor(() => {
      expect(mockedListItems.mock.calls.at(-1)?.[0]).toEqual(
        expect.objectContaining({ title: 'テスト' }),
      )
    })
  })

  it('shows a loading indicator with role=status while the initial fetch is pending', () => {
    mockedListItems.mockImplementationOnce(() => new Promise(() => {}) as never)

    renderHomePage()

    expect(screen.getByRole('status')).toHaveTextContent('読み込み中')
  })

  it('shows an error message and retry button on fetch failure, and refetches on click', async () => {
    mockedListItems.mockRejectedValueOnce(new Error('network error'))
    mockedListItems.mockResolvedValueOnce({
      data: [makeItem()],
      pagination: { page: 1, limit: 20, total: 1 },
    } as never)

    const user = userEvent.setup()
    renderHomePage()

    await waitFor(() => expect(screen.getByRole('status')).toHaveTextContent('取得に失敗'))

    const retryButton = screen.getByRole('button', { name: '再試行' })
    await user.click(retryButton)

    await waitFor(() => expect(screen.getByText('星屑のシンフォニア')).toBeInTheDocument())
  })

  it('shows an empty state message and add-item link when there are no items', async () => {
    mockedListItems.mockResolvedValueOnce({
      data: [],
      pagination: { page: 1, limit: 20, total: 0 },
    } as never)

    renderHomePage()

    await waitFor(() => expect(screen.getByRole('status')).toHaveTextContent('該当するアイテムがありません'))
    expect(screen.getAllByRole('link', { name: /作品を追加/ }).length).toBeGreaterThan(0)
  })

  it('renders the titlebar heading and add-item link (01_home.html structure)', () => {
    mockedListItems.mockResolvedValueOnce({
      data: [],
      pagination: { page: 1, limit: 20, total: 0 },
    } as never)

    renderHomePage()

    expect(screen.getByRole('heading', { level: 1, name: '全体一覧' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: /作品を追加/ })).toBeInTheDocument()
  })
})
