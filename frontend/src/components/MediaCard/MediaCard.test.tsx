import { describe, expect, it, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MediaCard } from './MediaCard'
import type { Item } from '@/features/items/types'
import { useUpdateItemStatus } from '@/features/items/hooks'

const navigateMock = vi.fn()

vi.mock('react-router-dom', () => ({
  useNavigate: () => navigateMock,
}))

const mutateMock = vi.fn()

vi.mock('@/features/items/hooks', () => ({
  useUpdateItemStatus: vi.fn(),
}))

const mockedUseUpdateItemStatus = vi.mocked(useUpdateItemStatus)

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

describe('MediaCard', () => {
  beforeEach(() => {
    navigateMock.mockClear()
    mutateMock.mockClear()
    mockedUseUpdateItemStatus.mockReturnValue({
      mutate: mutateMock,
      isPending: false,
    } as unknown as ReturnType<typeof useUpdateItemStatus>)
  })

  it('renders the media_type badge text', () => {
    render(<MediaCard item={makeItem({ media_type: 'movie' })} />)
    expect(screen.getByText('MOVIE')).toBeInTheDocument()
  })

  it('shows the favorite star when is_favorite is true', () => {
    render(<MediaCard item={makeItem({ is_favorite: true })} />)
    expect(screen.getByText('★')).toBeInTheDocument()
  })

  it('does not show the favorite star when is_favorite is false', () => {
    render(<MediaCard item={makeItem({ is_favorite: false })} />)
    expect(screen.queryByText('★')).not.toBeInTheDocument()
  })

  it('renders the title and status label via StatusDropdown', () => {
    render(<MediaCard item={makeItem({ title: 'テストタイトル', status: 'completed' })} />)
    expect(screen.getByText('テストタイトル')).toBeInTheDocument()
    expect(screen.getByText('視聴済')).toBeInTheDocument()
  })

  it('appends the season_label suffix when present', () => {
    render(<MediaCard item={makeItem({ status: 'in_progress', season_label: 'S2' })} />)
    expect(screen.getByText('· S2')).toBeInTheDocument()
  })

  it('does not append a suffix when season_label is absent', () => {
    render(<MediaCard item={makeItem({ status: 'in_progress', season_label: null })} />)
    expect(screen.queryByText('· S2')).not.toBeInTheDocument()
  })

  it('navigates to the item detail page when clicked', async () => {
    const user = userEvent.setup()
    render(<MediaCard item={makeItem({ id: 'abc-123' })} />)
    await user.click(screen.getByRole('button', { name: /詳細を開く/ }))
    expect(navigateMock).toHaveBeenCalledWith('/items/abc-123')
  })

  it('navigates when Enter key is pressed', async () => {
    const user = userEvent.setup()
    render(<MediaCard item={makeItem({ id: 'xyz-789' })} />)
    screen.getByRole('button', { name: /詳細を開く/ }).focus()
    await user.keyboard('{Enter}')
    expect(navigateMock).toHaveBeenCalledWith('/items/xyz-789')
  })

  it('calls useUpdateItemStatus.mutate with the correct arguments when a status is selected', async () => {
    const user = userEvent.setup()
    render(<MediaCard item={makeItem({ id: 'item-1', status: 'not_started' })} />)

    await user.click(screen.getByRole('button', { name: /ステータス/ }))
    await user.click(screen.getByRole('menuitem', { name: '視聴中' }))

    expect(mutateMock).toHaveBeenCalledWith({
      id: 'item-1',
      body: { status: 'in_progress' },
    })
  })

  it('does not navigate when the status dropdown trigger is clicked', async () => {
    const user = userEvent.setup()
    render(<MediaCard item={makeItem({ id: 'item-1' })} />)

    await user.click(screen.getByRole('button', { name: /ステータス/ }))

    expect(navigateMock).not.toHaveBeenCalled()
  })
})
