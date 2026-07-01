import { describe, expect, it, vi } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useConfirmDialog } from './useConfirmDialog'

describe('useConfirmDialog', () => {
  // ===== 正常系 =====

  it('TC-CD2-N-01: 初期状態でopen===falseである', () => {
    const { result } = renderHook(() => useConfirmDialog())

    expect(result.current.open).toBe(false)
  })

  it('TC-CD2-N-02: confirm(action) 呼び出し後にopen===trueになる', () => {
    const { result } = renderHook(() => useConfirmDialog())

    act(() => {
      result.current.confirm(vi.fn())
    })

    expect(result.current.open).toBe(true)
  })

  it('TC-CD2-N-03: handleConfirm() 呼び出し後に渡したactionが実行されopen===falseに戻る', () => {
    const { result } = renderHook(() => useConfirmDialog())
    const action = vi.fn()

    act(() => {
      result.current.confirm(action)
    })

    act(() => {
      result.current.handleConfirm()
    })

    expect(action).toHaveBeenCalledTimes(1)
    expect(result.current.open).toBe(false)
  })

  it('TC-CD2-N-04: handleCancel() 呼び出し後にactionは実行されずopen===falseになる', () => {
    const { result } = renderHook(() => useConfirmDialog())
    const action = vi.fn()

    act(() => {
      result.current.confirm(action)
    })

    act(() => {
      result.current.handleCancel()
    })

    expect(action).not.toHaveBeenCalled()
    expect(result.current.open).toBe(false)
  })

  // ===== 異常系 =====

  it('TC-CD2-E-01: handleConfirm を open=false の状態で呼んでもエラーにならない', () => {
    const { result } = renderHook(() => useConfirmDialog())

    expect(() => {
      act(() => {
        result.current.handleConfirm()
      })
    }).not.toThrow()

    expect(result.current.open).toBe(false)
  })

  it('TC-CD2-E-02: confirm を複数回呼んだ場合、最後のactionで上書きされる', () => {
    const { result } = renderHook(() => useConfirmDialog())
    const firstAction = vi.fn()
    const secondAction = vi.fn()

    act(() => {
      result.current.confirm(firstAction)
    })
    act(() => {
      result.current.confirm(secondAction)
    })
    act(() => {
      result.current.handleConfirm()
    })

    expect(firstAction).not.toHaveBeenCalled()
    expect(secondAction).toHaveBeenCalledTimes(1)
  })

  // ===== 境界値 =====

  it('TC-CD2-B-01: confirm → handleConfirm → confirm → handleCancel の一連フローが正しく動く', () => {
    const { result } = renderHook(() => useConfirmDialog())
    const action1 = vi.fn()
    const action2 = vi.fn()

    act(() => { result.current.confirm(action1) })
    act(() => { result.current.handleConfirm() })
    expect(action1).toHaveBeenCalledTimes(1)
    expect(result.current.open).toBe(false)

    act(() => { result.current.confirm(action2) })
    act(() => { result.current.handleCancel() })
    expect(action2).not.toHaveBeenCalled()
    expect(result.current.open).toBe(false)
  })
})
