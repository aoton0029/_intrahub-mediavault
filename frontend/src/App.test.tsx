import { describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import App from './App'

vi.mock('@/api/items', () => ({
  useItemsQuery: vi.fn(() => ({
    data: { data: [], pagination: undefined },
    isLoading: false,
    isError: false,
  })),
}))

vi.mock('@/hooks/useSearchParamsFilter', () => ({
  useSearchParamsFilter: vi.fn(() => ({ filters: {}, setFilters: vi.fn() })),
}))

describe('App', () => {
  it('renders the home page at the root route', () => {
    render(<App />)
    expect(screen.getByTestId('filter-bar')).toBeInTheDocument()
  })
})
