import { describe, expect, it } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { FilterBar } from './FilterBar'

function renderWithRouter(initialEntries: string[] = ['/']) {
  return render(
    <MemoryRouter initialEntries={initialEntries}>
      <FilterBar />
    </MemoryRouter>,
  )
}

describe('FilterBar', () => {
  it('clicking the favorite chip updates useItemFilters state via the URL', async () => {
    const user = userEvent.setup()
    renderWithRouter()
    const chip = screen.getByRole('button', { name: /お気に入り/ })
    await user.click(chip)
    expect(chip).toHaveAttribute('aria-pressed', 'true')
  })

  it('allows selecting multiple chips simultaneously (favorite + tag)', async () => {
    const user = userEvent.setup()
    renderWithRouter()
    const favoriteChip = screen.getByRole('button', { name: /お気に入り/ })
    const tagChip = screen.getByRole('button', { name: '#SF' })

    await user.click(favoriteChip)
    await user.click(tagChip)

    expect(favoriteChip).toHaveAttribute('aria-pressed', 'true')
    expect(tagChip).toHaveAttribute('aria-pressed', 'true')
  })

  it('updates the title filter immediately on every keystroke', async () => {
    const user = userEvent.setup()
    renderWithRouter()
    const input = screen.getByLabelText('タイトルで検索')
    await user.type(input, 'テスト')
    expect(input).toHaveValue('テスト')
  })

  it('clears all filters when the "すべて" chip is clicked', async () => {
    const user = userEvent.setup()
    renderWithRouter()
    const favoriteChip = screen.getByRole('button', { name: /お気に入り/ })
    const allChip = screen.getByRole('button', { name: 'すべて' })

    await user.click(favoriteChip)
    expect(favoriteChip).toHaveAttribute('aria-pressed', 'true')

    await user.click(allChip)
    expect(favoriteChip).toHaveAttribute('aria-pressed', 'false')
    expect(allChip).toHaveAttribute('aria-pressed', 'true')
  })

  it('marks the selected chip with aria-pressed="true" and others with "false"', async () => {
    const user = userEvent.setup()
    renderWithRouter()
    const statusChip = screen.getByRole('button', { name: '視聴中' })
    await user.click(statusChip)
    expect(statusChip).toHaveAttribute('aria-pressed', 'true')

    const otherStatusChip = screen.getByRole('button', { name: '未着手' })
    expect(otherStatusChip).toHaveAttribute('aria-pressed', 'false')
  })
})
