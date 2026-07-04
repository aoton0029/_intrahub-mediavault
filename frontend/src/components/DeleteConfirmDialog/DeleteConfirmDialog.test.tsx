import { describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { DeleteConfirmDialog } from './DeleteConfirmDialog'

describe('DeleteConfirmDialog', () => {
  it('open: trueの場合ダイアログが表示され、itemTitleを含む確認文言が表示されること', () => {
    render(
      <DeleteConfirmDialog
        open
        itemTitle="サンプル作品"
        onConfirm={vi.fn()}
        onCancel={vi.fn()}
      />,
    )

    expect(screen.getByRole('alertdialog')).toBeInTheDocument()
    expect(screen.getByText('作品を削除しますか?')).toBeInTheDocument()
    expect(screen.getByText('『サンプル作品』を削除します。この操作は取り消せません。')).toBeInTheDocument()
  })

  it('open: falseの場合ダイアログが表示されないこと', () => {
    render(
      <DeleteConfirmDialog
        open={false}
        itemTitle="サンプル作品"
        onConfirm={vi.fn()}
        onCancel={vi.fn()}
      />,
    )

    expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument()
  })

  it('確定ボタン押下でonConfirmが1回呼び出されること', async () => {
    const user = userEvent.setup()
    const onConfirm = vi.fn()
    render(
      <DeleteConfirmDialog
        open
        itemTitle="サンプル作品"
        onConfirm={onConfirm}
        onCancel={vi.fn()}
      />,
    )

    await user.click(screen.getByRole('button', { name: '削除' }))

    expect(onConfirm).toHaveBeenCalledTimes(1)
  })

  it('キャンセルボタン押下でonCancelが1回呼び出されること', async () => {
    const user = userEvent.setup()
    const onCancel = vi.fn()
    render(
      <DeleteConfirmDialog
        open
        itemTitle="サンプル作品"
        onConfirm={vi.fn()}
        onCancel={onCancel}
      />,
    )

    await user.click(screen.getByRole('button', { name: 'キャンセル' }))

    expect(onCancel).toHaveBeenCalledTimes(1)
  })

  it('isDeleting: trueの場合、確定ボタンが無効化されること', () => {
    render(
      <DeleteConfirmDialog
        open
        itemTitle="サンプル作品"
        isDeleting
        onConfirm={vi.fn()}
        onCancel={vi.fn()}
      />,
    )

    expect(screen.getByRole('button', { name: '削除' })).toBeDisabled()
  })

  it('itemTitleが空文字の場合でもダイアログ自体はエラーなく表示されること', () => {
    render(
      <DeleteConfirmDialog
        open
        itemTitle=""
        onConfirm={vi.fn()}
        onCancel={vi.fn()}
      />,
    )

    expect(screen.getByRole('alertdialog')).toBeInTheDocument()
    expect(screen.getByText('『』を削除します。この操作は取り消せません。')).toBeInTheDocument()
  })
})
