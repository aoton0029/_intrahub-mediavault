import { describe, expect, it } from 'vitest'
import { render, screen } from '@testing-library/react'
import { TagPill } from './TagPill'

describe('TagPill', () => {
  // ===== 正常系 =====

  it('TC-TP-N-01: labelに#プレフィックスを付けて表示する', () => {
    render(<TagPill label="anime" />)
    expect(screen.getByText('#anime')).toBeInTheDocument()
  })

  it('TC-TP-N-02: 任意のlabel文字列が正しく描画される', () => {
    render(<TagPill label="お気に入り" />)
    expect(screen.getByText('#お気に入り')).toBeInTheDocument()
  })

  // ===== 異常系 =====

  it('TC-TP-E-01: 空文字列のlabelでも例外を起こさない', () => {
    expect(() => render(<TagPill label="" />)).not.toThrow()
    expect(screen.getByTestId('tag-pill')).toHaveTextContent('#')
  })

  // ===== 境界値 =====

  it('TC-TP-B-01: ルート要素にtag-pillクラスが付与される', () => {
    render(<TagPill label="movie" />)
    expect(screen.getByTestId('tag-pill')).toHaveClass('tag-pill')
  })
})
