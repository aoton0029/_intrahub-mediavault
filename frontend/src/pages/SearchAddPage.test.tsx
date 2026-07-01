/**
 * TASK-0016: SearchAddPage テスト
 */
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ApiClientError } from '@/types'
import type { ExternalSearchResultItem, Item } from '@/types'
import SearchAddPage from './SearchAddPage'

vi.mock('@/api/search', () => ({
  useExternalSearchQuery: vi.fn(),
  useImportItemMutation: vi.fn(),
}))

vi.mock('sonner', () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}))

const mockNavigate = vi.fn()
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual<typeof import('react-router-dom')>('react-router-dom')
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  }
})

import { useExternalSearchQuery, useImportItemMutation } from '@/api/search'
import { toast } from 'sonner'

const mockSearchResult: ExternalSearchResultItem = {
  externalId: 'ext-001',
  title: '進撃の巨人',
  coverImageUrl: 'https://example.com/cover.jpg',
  releaseDate: '2013-04-07',
  raw: { id: 'ext-001' },
}

const mockItem: Item = {
  id: 'item-001',
  title: '進撃の巨人',
  mediaType: 'anime',
  status: 'not_started',
  source: 'api',
  isFavorite: false,
  tags: [],
  categories: [],
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: '2026-01-01T00:00:00Z',
} as unknown as Item

const defaultMutate = vi.fn()

function renderPage(group: 'general' | 'academic' | 'paper' = 'general') {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter initialEntries={[`/search/${group}`]}>
        <SearchAddPage group={group} />
      </MemoryRouter>
    </QueryClientProvider>
  )
}

beforeEach(() => {
  vi.mocked(useExternalSearchQuery).mockReturnValue({
    data: undefined,
    isLoading: false,
    isError: false,
    error: null,
    refetch: vi.fn(),
  } as ReturnType<typeof useExternalSearchQuery>)

  vi.mocked(useImportItemMutation).mockReturnValue({
    mutate: defaultMutate,
    isPending: false,
  } as unknown as ReturnType<typeof useImportItemMutation>)

  mockNavigate.mockReset()
  defaultMutate.mockReset()
  vi.mocked(toast.success).mockReset()
  vi.mocked(toast.error).mockReset()
})

describe('テストケース1: groupに応じたmedia_type選択肢', () => {
  it('group=academicのとき academic_book のみが選択肢に表示される', () => {
    renderPage('academic')

    const select = screen.getByRole('combobox', { name: /メディアタイプ/i })
    const options = Array.from(select.querySelectorAll('option')).map(o => o.value)

    expect(options).toEqual(['academic_book'])
  })

  it('group=paperのとき paper のみが選択肢に表示される', () => {
    renderPage('paper')

    const select = screen.getByRole('combobox', { name: /メディアタイプ/i })
    const options = Array.from(select.querySelectorAll('option')).map(o => o.value)

    expect(options).toEqual(['paper'])
  })

  it('group=generalのとき6種のmedia_typeが選択肢に表示される', () => {
    renderPage('general')

    const select = screen.getByRole('combobox', { name: /メディアタイプ/i })
    const options = Array.from(select.querySelectorAll('option')).map(o => o.value)

    expect(options).toContain('anime')
    expect(options).toContain('movie')
    expect(options).toContain('drama')
    expect(options).toContain('manga')
    expect(options).toContain('novel')
    expect(options).toContain('game')
    expect(options).not.toContain('academic_book')
    expect(options).not.toContain('paper')
  })
})

describe('テストケース2: 検索実行で結果一覧が表示される', () => {
  it('検索語を入力して検索ボタンを押すとuseExternalSearchQueryが結果を返し一覧表示される', async () => {
    vi.mocked(useExternalSearchQuery).mockReturnValue({
      data: [mockSearchResult],
      isLoading: false,
      isError: false,
      error: null,
      refetch: vi.fn(),
    } as ReturnType<typeof useExternalSearchQuery>)

    renderPage('general')

    const input = screen.getByLabelText('検索語')
    fireEvent.change(input, { target: { value: '進撃' } })
    const searchBtn = screen.getByRole('button', { name: /検索/i })
    fireEvent.click(searchBtn)

    await waitFor(() => {
      expect(screen.getByText('進撃の巨人')).toBeInTheDocument()
    })
  })

  it('isLoading=trueのときローディングインジケータが表示されボタンがdisabledになる', () => {
    vi.mocked(useExternalSearchQuery).mockReturnValue({
      data: undefined,
      isLoading: true,
      isError: false,
      error: null,
      refetch: vi.fn(),
    } as ReturnType<typeof useExternalSearchQuery>)

    renderPage()

    expect(screen.getByLabelText('ローディング')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /検索/i })).toBeDisabled()
  })
})

describe('テストケース3: 結果選択→インポートで詳細画面へ遷移', () => {
  it('「追加」ボタン押下でuseImportItemMutationが実行され成功時にtoastとnavigateが呼ばれる', async () => {
    vi.mocked(useExternalSearchQuery).mockReturnValue({
      data: [mockSearchResult],
      isLoading: false,
      isError: false,
      error: null,
      refetch: vi.fn(),
    } as ReturnType<typeof useExternalSearchQuery>)

    vi.mocked(useImportItemMutation).mockReturnValue({
      mutate: vi.fn((_, opts) => opts?.onSuccess?.(mockItem, {} as any, {} as any)),
      isPending: false,
    } as unknown as ReturnType<typeof useImportItemMutation>)

    renderPage('general')

    const input = screen.getByLabelText('検索語')
    fireEvent.change(input, { target: { value: '進撃' } })
    fireEvent.click(screen.getByRole('button', { name: /検索/i }))

    await waitFor(() => {
      expect(screen.getByText('進撃の巨人')).toBeInTheDocument()
    })

    fireEvent.click(screen.getByRole('button', { name: /追加/i }))

    expect(toast.success).toHaveBeenCalledWith('追加しました')
    expect(mockNavigate).toHaveBeenCalledWith('/items/item-001')
  })
})

describe('テストケース4: API_KEY_NOT_CONFIGURED時に手動追加導線が表示される', () => {
  it('API_KEY_NOT_CONFIGUREDエラー時に「手動で追加する」ボタンが表示されリンクが正しい', async () => {
    const error = new ApiClientError('API_KEY_NOT_CONFIGURED', 'APIキーが設定されていません')
    vi.mocked(useExternalSearchQuery).mockReturnValue({
      data: undefined,
      isLoading: false,
      isError: true,
      error,
      refetch: vi.fn(),
    } as ReturnType<typeof useExternalSearchQuery>)

    renderPage('general')

    const input = screen.getByLabelText('検索語')
    fireEvent.change(input, { target: { value: '進撃' } })
    fireEvent.click(screen.getByRole('button', { name: /検索/i }))

    await waitFor(() => {
      expect(screen.getByText('APIキーが設定されていません')).toBeInTheDocument()
    })

    const manualLink = screen.getByRole('link', { name: /手動で追加する/i })
    expect(manualLink).toBeInTheDocument()
    expect(manualLink.getAttribute('href')).toBe('/items/new/general')
  })
})

describe('テストケース5: EXTERNAL_API_TIMEOUT時に再試行ボタンが表示される', () => {
  it('EXTERNAL_API_TIMEOUTエラー時に「再試行」ボタンが表示され押すとrefetchが呼ばれる', async () => {
    const mockRefetch = vi.fn()
    const error = new ApiClientError('EXTERNAL_API_TIMEOUT', '検索がタイムアウトしました')
    vi.mocked(useExternalSearchQuery).mockReturnValue({
      data: undefined,
      isLoading: false,
      isError: true,
      error,
      refetch: mockRefetch,
    } as ReturnType<typeof useExternalSearchQuery>)

    renderPage('general')

    const input = screen.getByLabelText('検索語')
    fireEvent.change(input, { target: { value: '進撃' } })
    fireEvent.click(screen.getByRole('button', { name: /検索/i }))

    await waitFor(() => {
      expect(screen.getByText('検索がタイムアウトしました')).toBeInTheDocument()
    })

    const retryBtn = screen.getByRole('button', { name: /再試行/i })
    expect(retryBtn).toBeInTheDocument()
    fireEvent.click(retryBtn)

    expect(mockRefetch).toHaveBeenCalledTimes(1)
  })
})
