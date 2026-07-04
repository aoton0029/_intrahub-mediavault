import { describe, expect, it, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { StatusDropdown } from './StatusDropdown'

describe('StatusDropdown', () => {
  it('現在のstatus値がトリガー表示に反映されていること', () => {
    render(<StatusDropdown status="in_progress" onStatusChange={vi.fn()} />)

    const trigger = screen.getByRole('button', { name: 'ステータス: 視聴中。変更する' })
    expect(trigger).toBeInTheDocument()
    expect(trigger).toHaveTextContent('視聴中')
  })

  it('トリガークリックでメニューが開閉すること', async () => {
    const user = userEvent.setup()
    render(<StatusDropdown status="not_started" onStatusChange={vi.fn()} />)

    const trigger = screen.getByRole('button', { name: 'ステータス: 未着手。変更する' })
    expect(screen.queryByRole('menu')).not.toBeInTheDocument()

    await user.click(trigger)
    await waitFor(() => expect(screen.getByRole('menu')).toBeInTheDocument())

    await user.keyboard('{Escape}')
    await waitFor(() => expect(screen.queryByRole('menu')).not.toBeInTheDocument())
  })

  it('status選択時にonStatusChangeが選択値を引数として呼び出されること', async () => {
    const user = userEvent.setup()
    const onStatusChange = vi.fn()
    render(<StatusDropdown status="not_started" onStatusChange={onStatusChange} />)

    await user.click(screen.getByRole('button', { name: 'ステータス: 未着手。変更する' }))
    const item = await screen.findByRole('menuitem', { name: '視聴中' })
    await user.click(item)

    expect(onStatusChange).toHaveBeenCalledTimes(1)
    expect(onStatusChange).toHaveBeenCalledWith('in_progress')
  })

  it('isError指定時にエラー視覚状態が表示されること', () => {
    render(<StatusDropdown status="not_started" onStatusChange={vi.fn()} isError />)

    const trigger = screen.getByRole('button', { name: 'ステータス: 未着手。変更する' })
    expect(trigger).toHaveAttribute('aria-invalid', 'true')
    expect(trigger.className).toMatch(/error/)
  })

  it('Escapeキーでメニューが閉じること', async () => {
    const user = userEvent.setup()
    render(<StatusDropdown status="completed" onStatusChange={vi.fn()} />)

    await user.click(screen.getByRole('button', { name: 'ステータス: 視聴済。変更する' }))
    await waitFor(() => expect(screen.getByRole('menu')).toBeInTheDocument())

    await user.keyboard('{Escape}')
    await waitFor(() => expect(screen.queryByRole('menu')).not.toBeInTheDocument())
  })
})
