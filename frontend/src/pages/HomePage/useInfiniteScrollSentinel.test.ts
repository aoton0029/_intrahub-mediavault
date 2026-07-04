import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook } from '@testing-library/react'
import { useInfiniteScrollSentinel } from './useInfiniteScrollSentinel'

type ObserverCallback = (entries: Array<{ isIntersecting: boolean }>) => void

class MockIntersectionObserver {
  static instances: MockIntersectionObserver[] = []
  callback: ObserverCallback
  observedElements: Element[] = []

  constructor(callback: ObserverCallback) {
    this.callback = callback
    MockIntersectionObserver.instances.push(this)
  }

  observe(element: Element) {
    this.observedElements.push(element)
  }

  unobserve() {}
  disconnect() {}

  trigger(isIntersecting: boolean) {
    this.callback([{ isIntersecting }])
  }
}

describe('useInfiniteScrollSentinel', () => {
  beforeEach(() => {
    MockIntersectionObserver.instances = []
    vi.stubGlobal('IntersectionObserver', MockIntersectionObserver)
  })

  it('calls fetchNextPage when the sentinel becomes visible', () => {
    const fetchNextPage = vi.fn()
    const { result } = renderHook(() =>
      useInfiniteScrollSentinel({
        hasNextPage: true,
        isFetchingNextPage: false,
        fetchNextPage,
      }),
    )

    const div = document.createElement('div')
    result.current(div)

    const observer = MockIntersectionObserver.instances[0]
    observer.trigger(true)

    expect(fetchNextPage).toHaveBeenCalledTimes(1)
  })

  it('EDGE-102: does not call fetchNextPage when hasNextPage is false', () => {
    const fetchNextPage = vi.fn()
    const { result } = renderHook(() =>
      useInfiniteScrollSentinel({
        hasNextPage: false,
        isFetchingNextPage: false,
        fetchNextPage,
      }),
    )

    const div = document.createElement('div')
    result.current(div)

    const observer = MockIntersectionObserver.instances[0]
    observer.trigger(true)

    expect(fetchNextPage).not.toHaveBeenCalled()
  })

  it('does not call fetchNextPage while isFetchingNextPage is true', () => {
    const fetchNextPage = vi.fn()
    const { result } = renderHook(() =>
      useInfiniteScrollSentinel({
        hasNextPage: true,
        isFetchingNextPage: true,
        fetchNextPage,
      }),
    )

    const div = document.createElement('div')
    result.current(div)

    const observer = MockIntersectionObserver.instances[0]
    observer.trigger(true)

    expect(fetchNextPage).not.toHaveBeenCalled()
  })

  it('does not call fetchNextPage when the sentinel is not intersecting', () => {
    const fetchNextPage = vi.fn()
    const { result } = renderHook(() =>
      useInfiniteScrollSentinel({
        hasNextPage: true,
        isFetchingNextPage: false,
        fetchNextPage,
      }),
    )

    const div = document.createElement('div')
    result.current(div)

    const observer = MockIntersectionObserver.instances[0]
    observer.trigger(false)

    expect(fetchNextPage).not.toHaveBeenCalled()
  })
})
