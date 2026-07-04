import { describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { PropertiesPanel } from './index'
import type { ItemDetail } from '@/features/items/types'

function makeItemDetail(overrides: Partial<ItemDetail> = {}): ItemDetail {
  return {
    id: 'item-1',
    media_type: 'anime',
    title: '星屑のシンフォニア',
    status: 'in_progress',
    is_favorite: true,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    consumed_date: '2026-02-01',
    rating: 4,
    source: 'api (Jikan)',
    release_date: '2025-10-01',
    studio: 'Studio A',
    tags: [],
    categories: [],
    calibre_links: [],
    ...overrides,
  }
}

describe('PropertiesPanel', () => {
  it('renders all properties when fully populated (TC-004-02)', () => {
    render(<PropertiesPanel item={makeItemDetail()} onStatusChange={vi.fn()} />)

    expect(screen.getByText('anime')).toBeInTheDocument()
    expect(screen.getByText('2026-02-01')).toBeInTheDocument()
    expect(screen.getByText('★★★★☆ (4.0)')).toBeInTheDocument()
    expect(screen.getByText('★ お気に入り')).toBeInTheDocument()
    expect(screen.getByText('api (Jikan)')).toBeInTheDocument()
    expect(screen.getByText('2025-10-01')).toBeInTheDocument()
    expect(screen.getByText('Studio A')).toBeInTheDocument()
  })

  it('shows the favorite mark when favorite is true and hides it when false', () => {
    const { rerender } = render(
      <PropertiesPanel item={makeItemDetail({ is_favorite: true })} onStatusChange={vi.fn()} />,
    )
    expect(screen.getByText('★ お気に入り')).toBeInTheDocument()

    rerender(<PropertiesPanel item={makeItemDetail({ is_favorite: false })} onStatusChange={vi.fn()} />)
    expect(screen.queryByText('★ お気に入り')).not.toBeInTheDocument()
  })

  it('calls onStatusChange with the selected status when StatusDropdown is used', async () => {
    const onStatusChange = vi.fn()
    const user = userEvent.setup()
    render(
      <PropertiesPanel
        item={makeItemDetail({ status: 'not_started', consumed_date: null })}
        onStatusChange={onStatusChange}
      />,
    )

    await user.click(screen.getByRole('button', { name: /ステータス/ }))
    await user.click(screen.getByRole('menuitem', { name: '視聴中' }))

    expect(onStatusChange).toHaveBeenCalledWith({ status: 'in_progress', consumedDate: null })
  })

  it('shows "—" when consumed_date is null', () => {
    render(<PropertiesPanel item={makeItemDetail({ consumed_date: null })} onStatusChange={vi.fn()} />)
    expect(screen.getByText('—')).toBeInTheDocument()
  })

  it('hides the studio row when studio is not set', () => {
    render(<PropertiesPanel item={makeItemDetail({ studio: null })} onStatusChange={vi.fn()} />)
    expect(screen.queryByText('Studio A')).not.toBeInTheDocument()
    expect(screen.queryByText('studio')).not.toBeInTheDocument()
  })

  it('disables the StatusDropdown when isStatusUpdating is true', () => {
    render(
      <PropertiesPanel
        item={makeItemDetail()}
        onStatusChange={vi.fn()}
        isStatusUpdating
      />,
    )
    expect(screen.getByRole('button', { name: /ステータス/ })).toBeDisabled()
  })
})
