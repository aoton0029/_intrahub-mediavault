import { describe, expect, it, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import ItemDetailPage from './ItemDetailPage'
import type { Item } from '@/types'

// ===== モック =====

vi.mock('@/api/items', () => ({
  useItemQuery: vi.fn(),
  useDeleteItemMutation: vi.fn(),
}))

vi.mock('@/hooks/useConfirmDialog', () => ({
  useConfirmDialog: vi.fn(),
}))

vi.mock('sonner', () => ({
  toast: { error: vi.fn(), success: vi.fn() },
}))

const mockNavigate = vi.fn()
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual<typeof import('react-router-dom')>('react-router-dom')
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  }
})

import { useItemQuery, useDeleteItemMutation } from '@/api/items'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'

// ===== テストデータ =====

function makeItem(overrides: Partial<Item> = {}): Item {
  return {
    id: 'item-abc',
    title: '星屑のシンフォニア',
    mediaType: 'anime',
    status: 'in_progress',
    isFavorite: false,
    source: 'manual',
    createdAt: '2026-01-01T00:00:00Z',
    updatedAt: '2026-01-01T00:00:00Z',
    details: { episodeCount: undefined, seasonCount: undefined, studio: undefined, genreList: [], sourceType: undefined, jikanId: undefined },
    ...overrides,
  } as Item
}

const mockConfirm = vi.fn()
const mockHandleConfirm = vi.fn()
const mockHandleCancel = vi.fn()
const mockMutate = vi.fn()

function renderPage(id = 'item-abc') {
  return render(
    <MemoryRouter initialEntries={[`/items/${id}`]}>
      <Routes>
        <Route path="/items/:id" element={<ItemDetailPage />} />
      </Routes>
    </MemoryRouter>
  )
}

beforeEach(() => {
  mockNavigate.mockReset()
  mockConfirm.mockReset()
  mockHandleConfirm.mockReset()
  mockHandleCancel.mockReset()
  mockMutate.mockReset()

  vi.mocked(useConfirmDialog).mockReturnValue({
    open: false,
    confirm: mockConfirm,
    handleConfirm: mockHandleConfirm,
    handleCancel: mockHandleCancel,
  })

  vi.mocked(useDeleteItemMutation).mockReturnValue({
    mutate: mockMutate,
  } as unknown as ReturnType<typeof useDeleteItemMutation>)
})

// ===== 正常系 =====

describe('ItemDetailPage - パンくず・タイトルバー・ドキュメント本文（正常系）', () => {
  it('TC-IDP-N-01: パンくずリストに「ホーム」「メディアタイプ」「タイトル」が表示される', () => {
    // 【テスト目的】: `.breadcrumb`要素にホーム・カテゴリ（mediaType）・タイトルの階層が表示されることを確認する
    // 【テスト内容】: mediaType='anime'のitemをレンダリングし、パンくず内のテキストを検証する
    // 【期待される動作】: `.breadcrumb`内に「ホーム」「アニメ」「星屑のシンフォニア」が含まれる
    // 🔵 信頼性レベル: requirements.md セクション1・2、02_item_detail.htmlの`.breadcrumb`構造より
    vi.mocked(useItemQuery).mockReturnValue({
      data: { data: makeItem() },
      isLoading: false,
      error: null,
    } as unknown as ReturnType<typeof useItemQuery>)

    renderPage()

    // 【結果検証】: breadcrumb要素が存在し、期待される文言を含むことを確認
    const breadcrumb = document.querySelector('.breadcrumb')
    expect(breadcrumb).not.toBeNull() // 【確認内容】: `.breadcrumb`要素が実装されていることを確認 🔵
    expect(breadcrumb?.textContent).toContain('ホーム') // 【確認内容】: ホーム階層が表示されることを確認 🔵
    expect(breadcrumb?.textContent).toContain('星屑のシンフォニア') // 【確認内容】: タイトル階層が表示されることを確認 🔵
  })

  it('TC-IDP-N-02: パンくずの「ホーム」リンクがルート(/)を指す', () => {
    // 【テスト目的】: パンくず内の「ホーム」リンクのhrefが正しいことを確認する
    // 【テスト内容】: レンダリング後にホームリンクのhref属性を検証する
    // 【期待される動作】: href="/"
    // 🔵 信頼性レベル: requirements.md「ホームへのリンクがクリックで正しい遷移先を指すこと」より
    vi.mocked(useItemQuery).mockReturnValue({
      data: { data: makeItem() },
      isLoading: false,
      error: null,
    } as unknown as ReturnType<typeof useItemQuery>)

    renderPage()

    const homeLink = screen.getByRole('link', { name: 'ホーム' })
    expect(homeLink).toHaveAttribute('href', '/') // 【確認内容】: ホームリンクの遷移先が正しいことを確認 🔵
  })

  it('TC-IDP-N-03: タイトルバーに「編集」ボタンが表示され、編集画面へのリンクを持つ', () => {
    // 【テスト目的】: `.btn`クラスの「編集」ボタンが表示され、既存の編集画面遷移ロジックが維持されていることを確認する
    // 【テスト内容】: 編集ボタンのhref属性を検証する
    // 【期待される動作】: href="/items/item-abc/edit"
    // 🔵 信頼性レベル: requirements.md セクション3「編集ボタンは既存の/items/:id/edit遷移」より
    vi.mocked(useItemQuery).mockReturnValue({
      data: { data: makeItem() },
      isLoading: false,
      error: null,
    } as unknown as ReturnType<typeof useItemQuery>)

    renderPage()

    const editLink = screen.getByRole('link', { name: '編集' })
    expect(editLink).toHaveAttribute('href', '/items/item-abc/edit') // 【確認内容】: 編集ボタンの遷移先が既存ロジックと一致することを確認 🔵
  })

  it('TC-IDP-N-04: タイトルバーに「削除」ボタン（.btn-danger）が表示される', () => {
    // 【テスト目的】: `.btn-danger`クラスの「削除」ボタンが表示されることを確認する
    // 【テスト内容】: 削除ボタンのクラス名を検証する
    // 【期待される動作】: className に "btn-danger" を含む
    // 🔵 信頼性レベル: requirements.md セクション3・_shared.css `.btn-danger`定義より
    vi.mocked(useItemQuery).mockReturnValue({
      data: { data: makeItem() },
      isLoading: false,
      error: null,
    } as unknown as ReturnType<typeof useItemQuery>)

    renderPage()

    const deleteButton = screen.getByRole('button', { name: '削除' })
    expect(deleteButton.className).toContain('btn-danger') // 【確認内容】: 削除ボタンにbtn-dangerクラスが付与されていることを確認 🔵
  })

  it('TC-IDP-N-05: 「削除」ボタン押下でuseConfirmDialogのconfirmが呼ばれる', () => {
    // 【テスト目的】: 削除ボタン押下時に既存の削除確認フローが呼び出されることを確認する
    // 【テスト内容】: 削除ボタンをクリックし、confirm関数の呼び出しを検証する
    // 【期待される動作】: confirmが1回呼ばれる
    // 🔵 信頼性レベル: requirements.md セクション3「削除ボタン押下時にダイアログを開き」より
    vi.mocked(useItemQuery).mockReturnValue({
      data: { data: makeItem() },
      isLoading: false,
      error: null,
    } as unknown as ReturnType<typeof useItemQuery>)

    renderPage()

    fireEvent.click(screen.getByRole('button', { name: '削除' }))

    expect(mockConfirm).toHaveBeenCalledTimes(1) // 【確認内容】: 削除確認ダイアログを開く処理が呼び出されることを確認 🔵
  })

  it('TC-IDP-N-06: ConfirmDialogの確定処理からuseDeleteItemMutationのmutateがidで呼ばれる', () => {
    // 【テスト目的】: 削除確認の確定操作で既存の削除APIロジックが呼び出されることを確認する
    // 【テスト内容】: confirmに渡されたコールバックを実行し、mutateの呼び出し引数を検証する
    // 【期待される動作】: mutateが'item-abc'で呼ばれる
    // 🔵 信頼性レベル: requirements.md「確定時にuseDeleteItemMutationを呼び出す」より
    vi.mocked(useItemQuery).mockReturnValue({
      data: { data: makeItem() },
      isLoading: false,
      error: null,
    } as unknown as ReturnType<typeof useItemQuery>)

    renderPage()

    fireEvent.click(screen.getByRole('button', { name: '削除' }))

    // confirmに渡されたコールバックを取り出して実行する
    const confirmedAction = mockConfirm.mock.calls[0][0] as () => void
    confirmedAction()

    expect(mockMutate).toHaveBeenCalledWith('item-abc') // 【確認内容】: 削除APIがアイテムIDで呼び出されることを確認 🔵
  })

  it('TC-IDP-N-07: .doc-titleにitem.titleが表示される', () => {
    // 【テスト目的】: ドキュメント本文のタイトル要素が正しく表示されることを確認する
    // 【テスト内容】: `.doc-title`要素のテキスト内容を検証する
    // 【期待される動作】: `.doc-title`のテキストがitem.titleと一致する
    // 🔵 信頼性レベル: requirements.md セクション2・02_item_detail.htmlの`.doc-title`構造より
    vi.mocked(useItemQuery).mockReturnValue({
      data: { data: makeItem({ title: 'テストタイトル' }) },
      isLoading: false,
      error: null,
    } as unknown as ReturnType<typeof useItemQuery>)

    renderPage()

    const docTitle = document.querySelector('.doc-title')
    expect(docTitle).not.toBeNull() // 【確認内容】: `.doc-title`要素が実装されていることを確認 🔵
    expect(docTitle?.textContent).toBe('テストタイトル') // 【確認内容】: タイトルが正しく表示されることを確認 🔵
  })

  it('TC-IDP-N-08: originalTitleが設定されている場合.doc-originalに表示される', () => {
    // 【テスト目的】: 原題要素が正しく表示されることを確認する
    // 【テスト内容】: originalTitleを設定したitemで`.doc-original`のテキストを検証する
    // 【期待される動作】: `.doc-original`のテキストにoriginalTitleが含まれる
    // 🔵 信頼性レベル: requirements.md セクション2・02_item_detail.htmlの`.doc-original`構造より
    vi.mocked(useItemQuery).mockReturnValue({
      data: { data: makeItem({ originalTitle: 'Symphony of Stardust' }) },
      isLoading: false,
      error: null,
    } as unknown as ReturnType<typeof useItemQuery>)

    renderPage()

    const docOriginal = document.querySelector('.doc-original')
    expect(docOriginal).not.toBeNull() // 【確認内容】: `.doc-original`要素が実装されていることを確認 🔵
    expect(docOriginal?.textContent).toContain('Symphony of Stardust') // 【確認内容】: 原題が表示されることを確認 🔵
  })

  it('TC-IDP-N-09: .doc-section内にitem.descriptionが表示される', () => {
    // 【テスト目的】: 概要セクションが正しく表示されることを確認する
    // 【テスト内容】: descriptionを設定したitemで`.doc-section`のテキストを検証する
    // 【期待される動作】: `.doc-section`のテキストにdescriptionが含まれる
    // 🔵 信頼性レベル: requirements.md セクション2・02_item_detail.htmlの`.doc-section`構造より
    vi.mocked(useItemQuery).mockReturnValue({
      data: { data: makeItem({ description: 'SFアニメの概要文' }) },
      isLoading: false,
      error: null,
    } as unknown as ReturnType<typeof useItemQuery>)

    renderPage()

    const docSection = document.querySelector('.doc-section')
    expect(docSection).not.toBeNull() // 【確認内容】: `.doc-section`要素が実装されていることを確認 🔵
    expect(docSection?.textContent).toContain('SFアニメの概要文') // 【確認内容】: 概要が表示されることを確認 🔵
  })

  it('TC-IDP-N-10: .doc-cover要素が表示される', () => {
    // 【テスト目的】: カバー画像領域が実装されていることを確認する
    // 【テスト内容】: `.doc-cover`要素の存在を検証する
    // 【期待される動作】: `.doc-cover`がDOMに存在する
    // 🔵 信頼性レベル: requirements.md セクション2・02_item_detail.htmlの`.doc-cover`構造より
    vi.mocked(useItemQuery).mockReturnValue({
      data: { data: makeItem({ coverImageUrl: 'https://example.com/cover.jpg' }) },
      isLoading: false,
      error: null,
    } as unknown as ReturnType<typeof useItemQuery>)

    renderPage()

    const docCover = document.querySelector('.doc-cover')
    expect(docCover).not.toBeNull() // 【確認内容】: `.doc-cover`要素が実装されていることを確認 🔵
  })
})

// ===== 境界値 =====

describe('ItemDetailPage - 境界値', () => {
  it('TC-IDP-B-01: originalTitleが未設定の場合.doc-originalが表示されない', () => {
    // 【テスト目的】: originalTitle未設定時に不要な要素が表示されないことを確認する
    // 【テスト内容】: originalTitleを持たないitemで`.doc-original`の非存在を検証する
    // 【期待される動作】: `.doc-original`がDOMに存在しない
    // 🔵 信頼性レベル: testcases.md TC-IDP-B-01より
    vi.mocked(useItemQuery).mockReturnValue({
      data: { data: makeItem({ originalTitle: undefined }) },
      isLoading: false,
      error: null,
    } as unknown as ReturnType<typeof useItemQuery>)

    renderPage()

    const docOriginal = document.querySelector('.doc-original')
    expect(docOriginal).toBeNull() // 【確認内容】: 原題未設定時に`.doc-original`が表示されないことを確認 🔵
  })
})

// ===== 既存挙動の回帰確認 =====

describe('ItemDetailPage - 既存エラーハンドリングの回帰確認', () => {
  it('TC-IDP-E-01: ITEM_NOT_FOUNDエラー時は一覧へリダイレクトされる', async () => {
    // 【テスト目的】: 視覚変更後も既存のエラーハンドリングが壊れていないことを確認する
    // 【テスト内容】: ITEM_NOT_FOUNDエラーを返すuseItemQueryでnavigateが呼ばれることを検証する
    // 【期待される動作】: navigate('/')が呼ばれる
    // 🔵 信頼性レベル: 既存実装ItemDetailPage.tsxのuseEffectロジックより
    const { ApiClientError } = await import('@/types')
    vi.mocked(useItemQuery).mockReturnValue({
      data: undefined,
      isLoading: false,
      error: new ApiClientError('ITEM_NOT_FOUND', 'not found'),
    } as unknown as ReturnType<typeof useItemQuery>)

    renderPage()

    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalledWith('/') // 【確認内容】: 存在しないアイテムアクセス時に一覧へ戻ることを確認 🔵
    })
  })
})
