import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import ItemDetailPage from './index'
import { getItemDetail, updateItemStatus, deleteItem } from '@/features/items/api'
import { ApiClientError } from '@/lib/apiClient'
import { toast } from 'sonner'
import type { ItemDetail, ItemGroup } from '@/features/items/types'

/**
 * ItemDetailPage 統合テスト（TASK-0020, Redフェーズ）
 *
 * 対象: frontend/src/pages/ItemDetailPage/index.tsx（現状TASK-0001プレースホルダー）
 * 参照: docs/implements/collection-browsing/TASK-0020/testcases.md
 *
 * 【重要な前提】: ItemDetail型には groups/links/files/trailers/relations/staff/mylists は
 * ネストされない（tags/categories/calibre_links/detailのみ）。よってSeasonEpisodeList/
 * ItemMediaLinksへは空配列を渡す配線が本タスクの最小合格ラインとなる（testcases.md 0-2参照）。
 */

vi.mock('@/features/items/api', () => ({
  getItemDetail: vi.fn(),
  updateItemStatus: vi.fn(),
  deleteItem: vi.fn(),
}))

// 【トーストモック】: sonnerのtoast.errorをスパイし、描画タイミングに依存せず文言一致を検証する（testcases.md 0-3） 🟡
vi.mock('sonner', () => ({ toast: { error: vi.fn(), success: vi.fn() } }))

const mockedGetItemDetail = vi.mocked(getItemDetail)
const mockedUpdateItemStatus = vi.mocked(updateItemStatus)
const mockedDeleteItem = vi.mocked(deleteItem)
const mockedToastError = vi.mocked(toast.error)

function makeItemDetail(overrides: Partial<ItemDetail> = {}): ItemDetail {
  return {
    id: 'item-1',
    media_type: 'anime',
    title: '星屑のシンフォニア',
    status: 'in_progress',
    is_favorite: false,
    cover_image_url: null,
    season_label: null,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    original_title: 'Stardust Symphonia',
    description: 'あらすじ本文',
    consumed_date: null,
    rating: null,
    source: 'api (Jikan)',
    release_date: null,
    tags: [],
    categories: [],
    calibre_links: [],
    ...overrides,
  }
}

function renderItemDetailPage(id = 'item-1', seasonGroups?: ItemGroup[]) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  const result = render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={[`/items/${id}`]}>
        <Routes>
          <Route path="/items/:id" element={<ItemDetailPage seasonGroups={seasonGroups} />} />
          <Route path="/items/:id/edit" element={<div>編集画面プレースホルダー</div>} />
          <Route path="/" element={<div>ホーム画面プレースホルダー</div>} />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>,
  )
  // 【queryClient公開】: TC-006-E01のinvalidateQueries検証用にqueryClientをテスト側へ返す 🟡
  return { ...result, queryClient }
}

describe('ItemDetailPage', () => {
  beforeEach(() => {
    mockedGetItemDetail.mockReset()
    mockedUpdateItemStatus.mockReset()
    mockedDeleteItem.mockReset()
    mockedToastError.mockReset()
  })

  // ===== 正常系 =====

  it('TC-N-01: 有効なidで概要・プロパティパネル・メタセクションが一連で表示される', async () => {
    // 【テスト目的】: getItemDetailが成功レスポンスを返したとき、タイトル・原題・概要・プロパティパネル・タグ/カテゴリが描画されることを確認する
    // 【テスト内容】: ApiOk<ItemDetail>を解決させ、ItemDetailPageの本体レンダリングを検証する
    // 【期待される動作】: data状態でページ本体（.content + .properties）がレンダリングされる
    // 🔵 信頼性レベル: TC-004-01/02・requirements.md出力仕様表より

    // 【テストデータ準備】: 概要3項目とメタ配列(tags/categories)を含む代表的なアイテムを用意する
    // 【初期条件設定】: QueryClient(retry:false) + MemoryRouter(/items/item-1) をマウントする
    mockedGetItemDetail.mockResolvedValueOnce({
      success: true,
      data: makeItemDetail({
        id: 'item-1',
        title: '星屑のシンフォニア',
        original_title: 'Stardust Symphonia',
        description: 'あらすじ本文',
        media_type: 'anime',
        tags: [{ id: 't1', name: 'SF' }],
        categories: [{ id: 'c1', name: 'テレビ' }],
      }),
    })

    // 【実際の処理実行】: ItemDetailPageをレンダリングし、useItemDetailQueryのフェッチ完了を待つ
    renderItemDetailPage('item-1')

    // 【結果検証】: 主要セクションの存在と見出し階層を検証する
    await waitFor(() => {
      expect(screen.getByRole('heading', { level: 1, name: '星屑のシンフォニア' })).toBeInTheDocument() // 【確認内容】: ページタイトルが<h1>で表示されること 🔵
    })

    expect(screen.getByText('Stardust Symphonia')).toBeInTheDocument() // 【確認内容】: 原題が表示されること 🔵
    expect(screen.getByText('あらすじ本文')).toBeInTheDocument() // 【確認内容】: 概要本文が表示されること 🔵
    expect(screen.getByText('SF')).toBeInTheDocument() // 【確認内容】: タグ「SF」が表示されること 🔵
    expect(screen.getByText('テレビ')).toBeInTheDocument() // 【確認内容】: カテゴリ「テレビ」が表示されること 🔵
  })

  it('TC-N-02: media_type=animeかつseason groupsありでシーズン構成が表示される（seam）', async () => {
    // 【テスト目的】: mediaType='anime'とseasonを含むgroupsが渡ると「シーズン構成」見出しが出ることを確認する
    // 【テスト内容】: ページのシーズンデータ供給seam(seasonGroups prop)にテスト値を注入して検証する
    // 【期待される動作】: <h3>シーズン構成</h3>と各話数行が描画される
    // 🟡 信頼性レベル: TC-004-01は🔵だが、groupsの供給経路が型未定義のためseam実装方針は妥当な推測

    mockedGetItemDetail.mockResolvedValueOnce({
      success: true,
      data: makeItemDetail({ id: 'item-1', media_type: 'anime', title: '星屑のシンフォニア' }),
    })

    const seasonGroups: ItemGroup[] = [
      {
        id: 'g1',
        group_type: 'season',
        group_name: '第1期',
        display_order: 0,
        episodes: [{ id: 'e1', episode_number: 1, title: '出会い' }],
      },
    ]

    renderItemDetailPage('item-1', seasonGroups)

    await waitFor(() => {
      expect(screen.getByRole('heading', { level: 1, name: '星屑のシンフォニア' })).toBeInTheDocument()
    })

    // 【結果検証】: シーズン構成見出しと注入したデータが描画されることを確認する
    expect(screen.getByRole('heading', { level: 3, name: 'シーズン構成' })).toBeInTheDocument() // 【確認内容】: シーズン構成見出しが表示されること 🟡
    expect(screen.getByText('第1期')).toBeInTheDocument() // 【確認内容】: シーズン名が表示されること 🟡
    expect(screen.getByText('出会い')).toBeInTheDocument() // 【確認内容】: 話数タイトルが表示されること 🟡
  })

  it('TC-N-03: media_type=movieではシーズン構成セクションが描画されない', async () => {
    // 【テスト目的】: mediaType='movie'のとき、SeasonEpisodeListがnullを返し見出しも出ないことを確認する
    // 【テスト内容】: media_type='movie'のItemDetailを解決させ、「シーズン構成」見出しがDOMに存在しないことを検証する
    // 【期待される動作】: 「シーズン構成」見出しが存在しない
    // 🔵 信頼性レベル: REQ-202・SeasonEpisodeList実装より

    // 【テストデータ準備】: movieはanime/drama以外のためシーズン非表示となるはずのケース
    mockedGetItemDetail.mockResolvedValueOnce({
      success: true,
      data: makeItemDetail({ id: 'item-2', media_type: 'movie', title: '銀幕の旅人' }),
    })

    renderItemDetailPage('item-2')

    await waitFor(() => {
      expect(screen.getByRole('heading', { level: 1, name: '銀幕の旅人' })).toBeInTheDocument()
    })

    // 【結果検証】: 「シーズン構成」見出しがDOMに存在しないことを確認する
    expect(screen.queryByRole('heading', { name: 'シーズン構成' })).not.toBeInTheDocument() // 【確認内容】: movieでシーズン構成の器ごと非表示になること 🔵
  })

  it('TC-N-04: 編集ボタン押下で/items/{id}/editへnavigateする', async () => {
    // 【テスト目的】: 「編集」ボタンクリックでnavigate('/items/item-1/edit')が発火することを確認する
    // 【テスト内容】: 編集ボタンをuserEvent.clickし、ルーティング先が編集画面プレースホルダーに変わることを検証する
    // 【期待される動作】: 現在ロケーションが/items/item-1/editへ変わる
    // 🟡 信頼性レベル: REQ-007は🔵だが遷移先ルートは別要件未確定のため妥当な推測

    mockedGetItemDetail.mockResolvedValueOnce({
      success: true,
      data: makeItemDetail({ id: 'item-1' }),
    })

    const user = userEvent.setup()
    renderItemDetailPage('item-1')

    await waitFor(() => {
      expect(screen.getByRole('heading', { level: 1 })).toBeInTheDocument()
    })

    // 【実際の処理実行】: 「編集」ボタンをクリックする
    const editButton = screen.getByRole('button', { name: /編集/ })
    await user.click(editButton)

    // 【結果検証】: 編集画面プレースホルダーへ遷移したことを確認する
    await waitFor(() => {
      expect(screen.getByText('編集画面プレースホルダー')).toBeInTheDocument() // 【確認内容】: /items/item-1/edit への遷移が発火したこと 🟡
    })
  })

  it('TC-N-05: パンくずがnav aria-label="パンくずリスト"として表示される', async () => {
    // 【テスト目的】: .titlebar内に<nav aria-label="パンくずリスト">があり、アイテムタイトルを含むことを確認する
    // 【テスト内容】: 有効データ表示後、navigationロールをaria-labelで取得できることを検証する
    // 【期待される動作】: screen.getByRole('navigation', { name: 'パンくずリスト' })が取得できる
    // 🟡 信頼性レベル: note.md注意事項2・02_item_detail.htmlより妥当な推測

    mockedGetItemDetail.mockResolvedValueOnce({
      success: true,
      data: makeItemDetail({ id: 'item-1', media_type: 'anime', title: '星屑のシンフォニア' }),
    })

    renderItemDetailPage('item-1')

    await waitFor(() => {
      expect(screen.getByRole('heading', { level: 1 })).toBeInTheDocument()
    })

    // 【結果検証】: パンくずナビゲーションのaria-labelとタイトル包含を確認する
    const breadcrumb = screen.getByRole('navigation', { name: 'パンくずリスト' })
    expect(breadcrumb).toBeInTheDocument() // 【確認内容】: aria-label="パンくずリスト"のnav要素が存在すること 🟡
    expect(breadcrumb).toHaveTextContent('星屑のシンフォニア') // 【確認内容】: パンくず内にアイテムタイトルを含むこと 🟡
  })

  it('TC-N-06: プロパティパネルのstatus変更でonStatusChange配線がクラッシュしない', async () => {
    // 【テスト目的】: PropertiesPanelのStatusDropdownでstatusを変更したとき、ページが渡したonStatusChange配線が
    // 例外なく発火することを確認する（API連携本実装はTASK-0021の責務範囲外）
    // 【テスト内容】: 有効データ表示後、ステータスドロップダウンを開いて別ステータスを選択する
    // 【期待される動作】: 例外なく状態選択でき、updateStatus(mutate)配線がクラッシュしない
    // 🟡 信頼性レベル: requirements.md責務境界より妥当な推測

    mockedGetItemDetail.mockResolvedValue({
      success: true,
      data: makeItemDetail({ id: 'item-1', title: '星屑のシンフォニア', status: 'not_started', consumed_date: null }),
    })
    mockedUpdateItemStatus.mockResolvedValueOnce({
      success: true,
      data: { id: 'item-1', status: 'in_progress' } as never,
    })

    const user = userEvent.setup()
    renderItemDetailPage('item-1')

    await waitFor(() => {
      expect(screen.getByRole('heading', { level: 1, name: '星屑のシンフォニア' })).toBeInTheDocument()
    })

    // 【実際の処理実行】: ステータスドロップダウンを開き、別ステータスを選択する
    await user.click(screen.getByRole('button', { name: /ステータス/ }))
    await expect(
      user.click(screen.getByRole('menuitem', { name: '視聴中' })),
    ).resolves.not.toThrow() // 【確認内容】: ステータス変更操作が例外を発生させないこと 🟡

    // 【結果検証】: 変更操作後もページ本体（タイトル）が引き続き表示され、クラッシュしていないことを確認する
    expect(screen.getByRole('heading', { level: 1, name: '星屑のシンフォニア' })).toBeInTheDocument() // 【確認内容】: onStatusChange配線発火後もページがクラッシュしないこと 🟡
  })

  // ===== 異常系 =====

  it('TC-E-01: ITEM_NOT_FOUNDで専用エラー画面と一覧へ戻る導線が表示される', async () => {
    // 【テスト目的】: getItemDetailがITEM_NOT_FOUNDでrejectしたとき、専用エラーメッセージと一覧への戻る導線が表示されることを確認する
    // 【テスト内容】: ApiClientError('ITEM_NOT_FOUND', 404, ...)でrejectさせ、エラー画面表示を検証する
    // 【期待される動作】: 「アイテムが見つかりません」表示 + 「一覧へ戻る」導線が存在し、本体は描画されない
    // 🔵 信頼性レベル: TC-004-E01・items.md 404・dataflow.mdより

    mockedGetItemDetail.mockRejectedValueOnce(
      new ApiClientError('ITEM_NOT_FOUND', 'アイテムが見つかりません'),
    )

    renderItemDetailPage('missing-id')

    // 【結果検証】: 404専用のエラーメッセージと戻る導線を確認する
    await waitFor(() => {
      expect(screen.getByText('アイテムが見つかりません')).toBeInTheDocument() // 【確認内容】: 404専用の日本語エラーメッセージが表示されること 🔵
    })

    expect(screen.getByRole('link', { name: /一覧へ戻る/ })).toBeInTheDocument() // 【確認内容】: 一覧へ戻る導線が存在すること 🔵
    expect(screen.queryByRole('heading', { level: 1 })).not.toBeInTheDocument() // 【確認内容】: 本体(タイトル等)が描画されないこと 🔵
  })

  it('TC-E-02: INTERNAL_ERRORで汎用メッセージと再試行ボタンが表示され、クリックで再取得される', async () => {
    // 【テスト目的】: サーバーエラー時に汎用メッセージと再試行ボタンを表示し、クリックでrefetchが走ることを確認する
    // 【テスト内容】: 1回目INTERNAL_ERRORでreject、再試行クリック後に成功データを解決させる
    // 【期待される動作】: 「エラーが発生しました」表示 + 「再試行」ボタン押下で本体表示に遷移
    // 🔵 信頼性レベル: dataflow.md・items.mdより（文言は🟡）

    mockedGetItemDetail.mockRejectedValueOnce(
      new ApiClientError('INTERNAL_ERROR', 'エラーが発生しました'),
    )
    mockedGetItemDetail.mockResolvedValueOnce({
      success: true,
      data: makeItemDetail({ id: 'item-1', title: '星屑のシンフォニア' }),
    })

    const user = userEvent.setup()
    renderItemDetailPage('item-1')

    // 【結果検証】: 汎用エラーメッセージと再試行ボタンの存在を確認する
    await waitFor(() => {
      expect(screen.getByText('エラーが発生しました')).toBeInTheDocument() // 【確認内容】: 404とは区別された汎用エラーメッセージが表示されること 🔵
    })

    const retryButton = screen.getByRole('button', { name: '再試行' })
    await user.click(retryButton)

    // 【結果検証】: 再試行後に本体（タイトル）が表示されることを確認する
    await waitFor(() => {
      expect(screen.getByRole('heading', { level: 1, name: '星屑のシンフォニア' })).toBeInTheDocument() // 【確認内容】: refetch成功後に本体が表示されること 🔵
    })
  })

  it('TC-E-03: 404エラー画面の「一覧へ戻る」クリックで/へ遷移する', async () => {
    // 【テスト目的】: TC-E-01の状態から「一覧へ戻る」クリックでホームへ遷移することを確認する
    // 【テスト内容】: ITEM_NOT_FOUND状態からリンクをuserEvent.clickし、遷移先を検証する
    // 【期待される動作】: ロケーションが / になり、ホーム画面プレースホルダーが表示される
    // 🔵 信頼性レベル: TC-004-E01・dataflow.mdより

    mockedGetItemDetail.mockRejectedValueOnce(
      new ApiClientError('ITEM_NOT_FOUND', 'アイテムが見つかりません'),
    )

    const user = userEvent.setup()
    renderItemDetailPage('missing-id')

    await waitFor(() => {
      expect(screen.getByText('アイテムが見つかりません')).toBeInTheDocument()
    })

    const backLink = screen.getByRole('link', { name: /一覧へ戻る/ })
    await user.click(backLink)

    // 【結果検証】: ホーム画面プレースホルダーへ遷移したことを確認する
    await waitFor(() => {
      expect(screen.getByText('ホーム画面プレースホルダー')).toBeInTheDocument() // 【確認内容】: / への遷移が発生したこと 🔵
    })
  })

  // ===== 境界値・状態 =====

  it('TC-B-01: isLoading中はスケルトンを表示し、本体・エラー文言は表示されない', () => {
    // 【テスト目的】: 初回ロード中はレイアウト骨格のスケルトンのみが表示されることを確認する
    // 【テスト内容】: getItemDetailが解決しないPromiseを返す状態でレンダリングし、role=statusのスケルトン領域を検証する
    // 【期待される動作】: role="status"のスケルトン表示があり、<h1>実タイトル・エラー文言は未表示
    // 🟡 信頼性レベル: UI/UX要件（スケルトン）は妥当な推測、検証手法はHomePage.test.tsx既存パターンに準拠 🔵

    // 【テストデータ準備】: 解決しないPromiseでローディング状態を維持する
    mockedGetItemDetail.mockImplementationOnce(() => new Promise(() => {}) as never)

    renderItemDetailPage('item-1')

    // 【結果検証】: スケルトン表示（role=status）が存在し、本体タイトルやエラー文言が無いことを確認する
    expect(screen.getByRole('status')).toBeInTheDocument() // 【確認内容】: ローディング中のスケルトン/ステータス表示が存在すること 🟡
    expect(screen.queryByRole('heading', { level: 1 })).not.toBeInTheDocument() // 【確認内容】: ロード中は実タイトルが描画されないこと 🟡
    expect(screen.queryByText('アイテムが見つかりません')).not.toBeInTheDocument() // 【確認内容】: ロード中はエラー文言が出ないこと 🟡
  })

  it('TC-B-02: cover_image_urlがnullのときプレースホルダーを表示する', async () => {
    // 【テスト目的】: cover_image_urlがnullのとき、.doc-cover内にプレースホルダー（背景/代替表示）が
    // 表示され、壊れたimg srcが出ないことを確認する
    // 【テスト内容】: cover_image_url: nullの有効データを解決させ、.doc-cover-placeholderの存在とimg不在を検証する
    // 【期待される動作】: .doc-coverにプレースホルダーがあり、imgタグは描画されない
    // 🟡 信頼性レベル: EC-2・02_item_detail.htmlより妥当な推測

    mockedGetItemDetail.mockResolvedValueOnce({
      success: true,
      data: makeItemDetail({ id: 'item-1', title: '星屑のシンフォニア', cover_image_url: null }),
    })

    const { container } = renderItemDetailPage('item-1')

    await waitFor(() => {
      expect(screen.getByRole('heading', { level: 1, name: '星屑のシンフォニア' })).toBeInTheDocument()
    })

    // 【結果検証】: プレースホルダー要素が存在し、imgタグが描画されないことを確認する
    expect(container.querySelector('.doc-cover-placeholder')).toBeInTheDocument() // 【確認内容】: cover_image_url null時にプレースホルダーが表示されること 🟡
    expect(container.querySelector('.doc-cover img')).not.toBeInTheDocument() // 【確認内容】: 壊れたimg srcが出ないこと 🟡
  })

  it('TC-B-03: 全てのメタ配列が空のとき空セクションの見出しを描画しない', async () => {
    // 【テスト目的】: tags/categories等が空でも空セクション見出しを残さないことを確認する
    // 【テスト内容】: tags=[], categories=[]の最小データを解決させ、各見出しが存在しないことを検証する
    // 【期待される動作】: 「タグ」「カテゴリ」「関連付け」「スタッフ」「マイリスト」「シーズン構成」見出しがいずれも存在しない
    // 🔵 信頼性レベル: 各コンポーネントの空配列null返却（実装確認済み）より

    mockedGetItemDetail.mockResolvedValueOnce({
      success: true,
      data: makeItemDetail({ id: 'item-3', title: '最小アイテム', media_type: 'movie', tags: [], categories: [] }),
    })

    renderItemDetailPage('item-3')

    await waitFor(() => {
      expect(screen.getByRole('heading', { level: 1, name: '最小アイテム' })).toBeInTheDocument()
    })

    // 【結果検証】: 各空セクションの見出しが存在しないことを確認する
    expect(screen.queryByRole('heading', { name: 'タグ' })).not.toBeInTheDocument() // 【確認内容】: 空のタグセクションが描画されないこと 🔵
    expect(screen.queryByRole('heading', { name: 'カテゴリ' })).not.toBeInTheDocument() // 【確認内容】: 空のカテゴリセクションが描画されないこと 🔵
    expect(screen.queryByRole('heading', { name: 'シーズン構成' })).not.toBeInTheDocument() // 【確認内容】: 空のシーズン構成セクションが描画されないこと 🔵
    expect(screen.queryByRole('heading', { name: 'リンク・ファイル・トレーラー' })).not.toBeInTheDocument() // 【確認内容】: 空のリンク・ファイル・トレーラーセクションが描画されないこと 🔵

    // 【結果検証】: プロパティパネル（item直渡し）は表示されることを確認する
    expect(screen.getByText('最小アイテム')).toBeInTheDocument() // 【確認内容】: 概要・プロパティパネルは空データでも描画されること 🔵
  })

  it('TC-B-04: idが空文字のときgetItemDetailが呼ばれずクラッシュしない', () => {
    // 【テスト目的】: 不正パス（id空文字）で安全にハンドリングされることを確認する
    // 【テスト内容】: ルートパラメータid=''でレンダリングし、getItemDetailが呼ばれないことを検証する
    // 【期待される動作】: フェッチされず、クラッシュしない
    // 🔵 信頼性レベル: hooks.ts L97-103 enabled: id!==''・requirements.md EC-3より

    const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })

    // 【実際の処理実行】: id空文字相当のパス(/items/)でレンダリングする
    expect(() => {
      render(
        <QueryClientProvider client={queryClient}>
          <MemoryRouter initialEntries={['/items/']}>
            <Routes>
              <Route path="/items/:id?" element={<ItemDetailPage />} />
            </Routes>
          </MemoryRouter>
        </QueryClientProvider>,
      )
    }).not.toThrow() // 【確認内容】: id空文字相当のパスでもレンダリング時に例外が発生しないこと 🔵

    // 【結果検証】: enabled=falseによりgetItemDetailが呼び出されないことを確認する
    expect(mockedGetItemDetail).not.toHaveBeenCalled() // 【確認内容】: 空idではAPIフェッチが発生しないこと 🔵
  })

  it('TC-B-05: original_title/descriptionがnullのとき該当箇所を安全に省略しつつタイトルは表示される', async () => {
    // 【テスト目的】: オプショナルなテキスト項目がnullでも表示が崩れないことを確認する
    // 【テスト内容】: original_title/descriptionをnullにしたItemDetailを解決させ、<h1>タイトルは表示されつつ例外にならないことを検証する
    // 【期待される動作】: <h1>タイトルは表示、原題/概要の空要素で例外にならない
    // 🟡 信頼性レベル: types.tsのnullable定義より妥当な推測

    mockedGetItemDetail.mockResolvedValueOnce({
      success: true,
      data: makeItemDetail({
        id: 'item-4',
        title: '無題の記録',
        original_title: null,
        description: null,
      }),
    })

    renderItemDetailPage('item-4')

    // 【結果検証】: タイトルは表示されnullフィールドで例外が起きないことを確認する
    await waitFor(() => {
      expect(screen.getByRole('heading', { level: 1, name: '無題の記録' })).toBeInTheDocument() // 【確認内容】: original_title/descriptionがnullでもタイトルは表示されること 🟡
    })
  })
})

/**
 * TASK-0021: 詳細画面からのstatus変更・削除統合 統合テスト（Redフェーズ）
 *
 * 対象: frontend/src/pages/ItemDetailPage/index.tsx（削除ボタンの配線が未実装）
 * 参照: docs/implements/collection-browsing/TASK-0021/testcases.md（TC-005/TC-006, 9件）
 *
 * 【重要な前提】: 削除ボタン（トリガー）とDeleteConfirmDialogの確定ボタンは、いずれもアクセシブルネームが
 * 「削除」であるため、確定ボタンの取得は `within(screen.getByRole('alertdialog'))` のスコープで行う。
 */
describe('ItemDetailPage - status変更・削除統合（TASK-0021）', () => {
  beforeEach(() => {
    mockedGetItemDetail.mockReset()
    mockedUpdateItemStatus.mockReset()
    mockedDeleteItem.mockReset()
    mockedToastError.mockReset()
  })

  function makeDetailItem(overrides: Partial<ItemDetail> = {}): ItemDetail {
    return {
      id: 'item-1',
      media_type: 'anime',
      title: '星屑のシンフォニア',
      status: 'not_started',
      is_favorite: false,
      cover_image_url: null,
      season_label: null,
      created_at: '2026-01-01T00:00:00Z',
      updated_at: '2026-01-01T00:00:00Z',
      original_title: 'Stardust Symphonia',
      description: 'あらすじ本文',
      consumed_date: null,
      rating: null,
      source: 'api (Jikan)',
      release_date: null,
      tags: [],
      categories: [],
      calibre_links: [],
      ...overrides,
    }
  }

  // ===== 正常系 =====

  it('TC-005-01/02: PropertiesPanelでstatusを変更するとPATCHが呼ばれ、成功時にパネル表示が更新される', async () => {
    // 【テスト目的】: StatusDropdownでの選択→onStatusChange→useUpdateItemStatus.mutate→updateItemStatusの統合経路と、成功後のパネル表示反映を確認する
    // 【テスト内容】: 「未視聴→視聴中」の状態遷移で、updateItemStatusが正しい引数で呼ばれ、パネルに「視聴中」が表示されることを検証する
    // 【期待される動作】: updateItemStatusが('item-1', { status:'in_progress', ... })で1回呼ばれ、パネルに新statusが反映される
    // 🔵 信頼性レベル: requirements.md 5正常系・testcases.md TC-005-01/02より（status変更経路は配線済みのため回帰テスト）

    // 【テストデータ準備】: 初期status='not_started'の代表的なアイテムを用意する
    // 【初期条件設定】: getItemDetail成功・updateItemStatus成功をモックする
    // 【補足】: onSettledで一覧・詳細キャッシュがinvalidateされ再フェッチされるため、
    // 再フェッチ分もin_progressを返すよう複数回分をキューイングする（onMutateの楽観更新→再フェッチ確定の順で反映される）
    mockedGetItemDetail.mockResolvedValueOnce({
      success: true,
      data: makeDetailItem({ id: 'item-1', status: 'not_started', consumed_date: null }),
    })
    mockedGetItemDetail.mockResolvedValue({
      success: true,
      data: makeDetailItem({ id: 'item-1', status: 'in_progress', consumed_date: null }),
    })
    mockedUpdateItemStatus.mockResolvedValueOnce({
      success: true,
      data: { id: 'item-1', status: 'in_progress' } as never,
    })

    const user = userEvent.setup()
    renderItemDetailPage('item-1')

    await waitFor(() => {
      expect(screen.getByRole('heading', { level: 1, name: '星屑のシンフォニア' })).toBeInTheDocument()
    })

    // 【実際の処理実行】: ステータスドロップダウンを開き「視聴中」を選択する
    // 【処理内容】: onStatusChange→updateStatus({id, body:{status, consumed_date}})の発火
    await user.click(screen.getByRole('button', { name: /ステータス/ }))
    await user.click(screen.getByRole('menuitem', { name: '視聴中' }))

    // 【結果検証】: updateItemStatusの呼び出し引数と、成功後のパネル表示反映を確認する
    // 【期待値確認】: dataflow.md機能4「成功→キャッシュ確定→パネル反映」に一致すること
    await waitFor(() => {
      expect(mockedUpdateItemStatus).toHaveBeenCalledWith(
        'item-1',
        expect.objectContaining({ status: 'in_progress' }),
      ) // 【確認内容】: idとstatusを含むbodyでupdateItemStatusが呼ばれること 🔵
    })
    await waitFor(() => {
      expect(screen.getByText('視聴中')).toBeInTheDocument() // 【確認内容】: 成功後にパネル表示が「視聴中」へ更新されること 🔵
    })
  })

  it('TC-006-01: 削除ボタン押下で確認ダイアログが開き、確定するとDELETEが呼ばれ成功時に/へ遷移する', async () => {
    // 【テスト目的】: 削除ボタンonClick→deleteDialogOpen=true→DeleteConfirmDialog表示→onConfirm→useDeleteItem.mutate(id)→deleteItem(id)→onSuccess navigate('/')の全経路を確認する
    // 【テスト内容】: 本タスクの主目的。現状の削除ボタンはonClickが未配線のため、このテストはRedで失敗する想定
    // 【期待される動作】: 削除ボタンクリックでダイアログが出現し、確定ボタンクリックでdeleteItem('item-1')が1回呼ばれ、成功後にホーム画面プレースホルダーへ遷移する
    // 🔵 信頼性レベル: requirements.md 4基本フロー・testcases.md TC-006-01より（本タスクの真の新規実装対象）

    // 【テストデータ準備】: 削除対象の有効アイテムを用意する（titleはダイアログ表示にも使用）
    // 【初期条件設定】: getItemDetail成功・deleteItem成功（204相当）をモックする
    mockedGetItemDetail.mockResolvedValueOnce({
      success: true,
      data: makeDetailItem({ id: 'item-1', title: '星屑のシンフォニア' }),
    })
    mockedDeleteItem.mockResolvedValueOnce(undefined)

    const user = userEvent.setup()
    renderItemDetailPage('item-1')

    await waitFor(() => {
      expect(screen.getByRole('heading', { level: 1 })).toBeInTheDocument()
    })

    // 【実際の処理実行】: 削除ボタンをクリックしてダイアログを開く
    // 【処理内容】: deleteDialogOpen=trueによりDeleteConfirmDialogが表示されるはず
    await user.click(screen.getByRole('button', { name: '削除' }))

    // 【結果検証】: ダイアログが表示されることを確認する（現状未配線のため失敗が期待される）
    const dialog = await screen.findByRole('alertdialog') // 【確認内容】: 削除ボタン押下でAlertDialogが表示されること 🔵

    // 【実際の処理実行】: ダイアログ内の確定ボタンをクリックする
    await user.click(within(dialog).getByRole('button', { name: '削除' }))

    // 【結果検証】: DELETE呼び出しとホーム遷移を確認する
    // 【期待値確認】: dataflow.md機能5「成功204→navigate('/')」・TC-006-01に一致すること
    expect(mockedDeleteItem).toHaveBeenCalledWith('item-1') // 【確認内容】: 正しいidでdeleteItemが呼ばれること 🔵
    await waitFor(() => {
      expect(screen.getByText('ホーム画面プレースホルダー')).toBeInTheDocument() // 【確認内容】: 成功後に/へ遷移すること 🔵
    })
  })

  it('TC-006-CANCEL: 削除確認ダイアログでキャンセルするとDELETEが呼ばれずダイアログが閉じる', async () => {
    // 【テスト目的】: onCancelによるダイアログクローズと、削除mutateが未発火であることを確認する
    // 【テスト内容】: 削除ボタン→ダイアログのキャンセルボタンをクリックし、deleteItemが呼ばれないことを検証する
    // 【期待される動作】: キャンセル後deleteItemは呼ばれず、詳細画面（タイトル）が表示されたまま
    // 🟡 信頼性レベル: requirements.md 4「キャンセル」注記からの妥当な推測（明示TCなし）

    mockedGetItemDetail.mockResolvedValueOnce({
      success: true,
      data: makeDetailItem({ id: 'item-1', title: '星屑のシンフォニア' }),
    })

    const user = userEvent.setup()
    renderItemDetailPage('item-1')

    await waitFor(() => {
      expect(screen.getByRole('heading', { level: 1 })).toBeInTheDocument()
    })

    // 【実際の処理実行】: 削除ボタン→ダイアログのキャンセルボタンをクリックする
    await user.click(screen.getByRole('button', { name: '削除' }))
    const dialog = await screen.findByRole('alertdialog')
    await user.click(within(dialog).getByRole('button', { name: 'キャンセル' }))

    // 【結果検証】: 削除mutateが未発火であり、ダイアログが閉じ、詳細画面に留まることを確認する
    expect(mockedDeleteItem).not.toHaveBeenCalled() // 【確認内容】: キャンセル時にdeleteItemが呼ばれないこと 🟡
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument() // 【確認内容】: ダイアログが閉じること 🟡
    })
    expect(screen.getByRole('heading', { level: 1 })).toBeInTheDocument() // 【確認内容】: 詳細画面に留まること 🟡
  })

  // ===== 異常系 =====

  it('TC-005-E01 (EDGE-003): status更新失敗で楽観更新がロールバックされエラートーストが出る', async () => {
    // 【テスト目的】: PATCH失敗時に楽観的更新がロールバックされ、エラートーストが表示されることを確認する
    // 【テスト内容】: updateItemStatusをApiClientErrorでreject させ、ロールバック後のパネル表示とtoast.error呼び出しを検証する
    // 【期待される動作】: toast.errorが'ステータスの更新に失敗しました'で呼ばれ、パネル表示が元のstatus（未視聴）へ戻る
    // 🔵 信頼性レベル: requirements.md 5異常系・EDGE-003・hooks.ts:166-176より

    // 【テストデータ準備】: 初期status='not_started'（表示「未視聴」）
    // 【不正な理由】: サーバー内部エラーで更新が確定できない状態を模す
    mockedGetItemDetail.mockResolvedValue({
      success: true,
      data: makeDetailItem({ id: 'item-1', status: 'not_started', consumed_date: null }),
    })
    mockedUpdateItemStatus.mockRejectedValueOnce(new ApiClientError('INTERNAL_ERROR', '更新に失敗しました'))

    const user = userEvent.setup()
    renderItemDetailPage('item-1')

    await waitFor(() => {
      expect(screen.getByRole('heading', { level: 1, name: '星屑のシンフォニア' })).toBeInTheDocument()
    })

    // 【実際の処理実行】: ドロップダウンで「視聴中」を選択する
    await user.click(screen.getByRole('button', { name: /ステータス/ }))
    await user.click(screen.getByRole('menuitem', { name: '視聴中' }))

    // 【結果検証】: エラートーストとロールバックを確認する
    // 【エラーメッセージの内容】: 操作対象（ステータス）が分かる日本語文言であること
    await waitFor(() => {
      expect(mockedToastError).toHaveBeenCalledWith('ステータスの更新に失敗しました') // 【確認内容】: 規定のエラートースト文言で通知されること 🔵
    })
    await waitFor(() => {
      expect(screen.getByText('未着手')).toBeInTheDocument() // 【確認内容】: 楽観更新が元のstatus表示へロールバックされること 🔵
    })
  })

  it('TC-006-E01 (EDGE-002): 削除404でエラートースト＋一覧invalidate、遷移しない', async () => {
    // 【テスト目的】: DELETE失敗（404 ITEM_NOT_FOUND）時に、エラートースト表示＋一覧クエリのinvalidateQueriesが呼ばれ、遷移しないことを確認する
    // 【テスト内容】: deleteItemをApiClientError('ITEM_NOT_FOUND', ...)でrejectさせ、トースト文言・invalidateQueries呼び出し・非遷移を検証する
    // 【期待される動作】: toast.errorが'アイテムが見つかりませんでした'で呼ばれ、queryClient.invalidateQueriesが['items']で呼ばれ、ホームへ遷移しない
    // 🔵 信頼性レベル: requirements.md 5異常系・EDGE-002・hooks.ts:204-211より

    mockedGetItemDetail.mockResolvedValueOnce({
      success: true,
      data: makeDetailItem({ id: 'item-1', title: '星屑のシンフォニア' }),
    })
    mockedDeleteItem.mockRejectedValueOnce(new ApiClientError('ITEM_NOT_FOUND', '見つかりません'))

    const user = userEvent.setup()
    const { queryClient } = renderItemDetailPage('item-1')
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries')

    await waitFor(() => {
      expect(screen.getByRole('heading', { level: 1 })).toBeInTheDocument()
    })

    // 【実際の処理実行】: 削除ボタン→ダイアログ確定ボタンをクリックする
    await user.click(screen.getByRole('button', { name: '削除' }))
    const dialog = await screen.findByRole('alertdialog')
    await user.click(within(dialog).getByRole('button', { name: '削除' }))

    // 【結果検証】: エラートースト・invalidateQueries呼び出し・非遷移を確認する
    // 【システムの安全性】: 遷移せず、一覧キャッシュだけ再取得対象になること
    await waitFor(() => {
      expect(mockedToastError).toHaveBeenCalledWith('アイテムが見つかりませんでした') // 【確認内容】: 404専用のエラートースト文言で通知されること 🔵
    })
    expect(invalidateSpy).toHaveBeenCalledWith(
      expect.objectContaining({ queryKey: ['items'] }),
    ) // 【確認内容】: 一覧クエリのinvalidateQueriesが呼ばれること 🔵
    expect(screen.queryByText('ホーム画面プレースホルダー')).not.toBeInTheDocument() // 【確認内容】: 404時は遷移しないこと 🔵
    expect(screen.getByRole('heading', { level: 1 })).toBeInTheDocument() // 【確認内容】: 詳細画面に留まること 🔵
  })

  it('TC-006-E02: 削除がその他エラー（非404）で汎用エラートースト、遷移しない', async () => {
    // 【テスト目的】: ITEM_NOT_FOUND以外の失敗で汎用の削除失敗トーストが表示され、遷移しないことを確認する
    // 【テスト内容】: deleteItemをApiClientError('INTERNAL_ERROR', ...)でrejectさせ、汎用トースト文言と非遷移を検証する
    // 【期待される動作】: toast.errorが'アイテムの削除に失敗しました'で呼ばれ、遷移しない
    // 🟡 信頼性レベル: hooks.ts:210汎用分岐より妥当な推測（明示TCなし、分岐網羅目的）

    mockedGetItemDetail.mockResolvedValueOnce({
      success: true,
      data: makeDetailItem({ id: 'item-1', title: '星屑のシンフォニア' }),
    })
    mockedDeleteItem.mockRejectedValueOnce(new ApiClientError('INTERNAL_ERROR', 'サーバーエラー'))

    const user = userEvent.setup()
    renderItemDetailPage('item-1')

    await waitFor(() => {
      expect(screen.getByRole('heading', { level: 1 })).toBeInTheDocument()
    })

    // 【実際の処理実行】: 削除ボタン→ダイアログ確定ボタンをクリックする
    await user.click(screen.getByRole('button', { name: '削除' }))
    const dialog = await screen.findByRole('alertdialog')
    await user.click(within(dialog).getByRole('button', { name: '削除' }))

    // 【結果検証】: 汎用エラートーストと非遷移を確認する
    await waitFor(() => {
      expect(mockedToastError).toHaveBeenCalledWith('アイテムの削除に失敗しました') // 【確認内容】: 非404時は汎用の削除失敗トーストが出ること 🟡
    })
    expect(screen.queryByText('ホーム画面プレースホルダー')).not.toBeInTheDocument() // 【確認内容】: 非404時も遷移しないこと 🟡
    expect(screen.getByRole('heading', { level: 1 })).toBeInTheDocument() // 【確認内容】: 詳細画面に留まること 🟡
  })

  // ===== 境界値・状態 =====

  it('TC-006-B01: 削除中（isPending=true）は確定ボタンが無効化され二重送信されない', async () => {
    // 【テスト目的】: 削除mutate実行中は確定ボタンが無効化され、連打してもmutateは1回のみであることを確認する
    // 【テスト内容】: deleteItemを未解決Promiseにしてpending状態を維持し、確定ボタンを2回クリックしても1回しか呼ばれないことを検証する
    // 【期待される動作】: deleteItemが1回のみ呼ばれ、確定ボタンがdisabledになる
    // 🟡 信頼性レベル: requirements.md 5二重送信防止・note.md注意事項4より妥当な推測

    mockedGetItemDetail.mockResolvedValueOnce({
      success: true,
      data: makeDetailItem({ id: 'item-1', title: '星屑のシンフォニア' }),
    })
    // 【境界値選択の根拠】: 未解決Promiseでpending状態を維持し、レスポンス遅延時のユーザー連打を模す
    mockedDeleteItem.mockImplementationOnce(() => new Promise(() => {}))

    const user = userEvent.setup()
    renderItemDetailPage('item-1')

    await waitFor(() => {
      expect(screen.getByRole('heading', { level: 1 })).toBeInTheDocument()
    })

    // 【実際の処理実行】: 削除ボタン→ダイアログ確定ボタンを2回クリックする
    await user.click(screen.getByRole('button', { name: '削除' }))
    const dialog = await screen.findByRole('alertdialog')
    const confirmButton = within(dialog).getByRole('button', { name: '削除' })
    await user.click(confirmButton)

    // 【結果検証】: pending中は確定ボタンが無効化され、mutateが1回のみ発火することを確認する
    await waitFor(() => {
      expect(confirmButton).toBeDisabled() // 【確認内容】: isDeleting伝搬により確定ボタンが無効化されること 🟡
    })
    // 【境界での正確性】: 無効化中は追加クリックが発火しないため、userEvent.clickは実施せず呼び出し回数のみ検証する
    expect(mockedDeleteItem).toHaveBeenCalledTimes(1) // 【確認内容】: pending中に追加でmutateが発火しないこと 🟡
  })

  it('TC-005-B01: status更新中（isPending=true）はStatusDropdownが無効化される', async () => {
    // 【テスト目的】: status更新mutate実行中はStatusDropdownが無効化されることを確認する
    // 【テスト内容】: updateItemStatusを未解決Promiseにしてpending状態を維持し、ドロップダウントリガーがdisabledになることを検証する
    // 【期待される動作】: 「視聴中」選択後、ステータストリガーボタンがdisabledになる
    // 🟡 信頼性レベル: requirements.md 5二重送信防止・index.tsx:191既配線より妥当な推測

    mockedGetItemDetail.mockResolvedValue({
      success: true,
      data: makeDetailItem({ id: 'item-1', status: 'not_started', consumed_date: null }),
    })
    // 【境界値選択の根拠】: 未解決Promiseでpending状態を維持し、更新レスポンス待ちでの連続操作抑止を検証する
    mockedUpdateItemStatus.mockImplementationOnce(() => new Promise(() => {}))

    const user = userEvent.setup()
    renderItemDetailPage('item-1')

    await waitFor(() => {
      expect(screen.getByRole('heading', { level: 1, name: '星屑のシンフォニア' })).toBeInTheDocument()
    })

    // 【実際の処理実行】: ステータスドロップダウンを開き「視聴中」を選択する
    const statusTrigger = screen.getByRole('button', { name: /ステータス/ })
    await user.click(statusTrigger)
    await user.click(screen.getByRole('menuitem', { name: '視聴中' }))

    // 【結果検証】: pending中はステータストリガーボタンが無効化されることを確認する
    await waitFor(() => {
      expect(statusTrigger).toBeDisabled() // 【確認内容】: isStatusUpdating伝搬によりStatusDropdownトリガーが無効化されること 🟡
    })
  })

  it('TC-006-B02: 削除ボタン押下前はDeleteConfirmDialogが開いていない（初期状態境界）', async () => {
    // 【テスト目的】: 初期表示時はDeleteConfirmDialogが開いていないことを確認する
    // 【テスト内容】: 有効データ表示直後（削除ボタン未押下）でalertdialogがDOMに存在しないことを検証する
    // 【期待される動作】: alertdialogが存在せず、削除ボタン自体は存在する
    // 🟡 信頼性レベル: note.md注意事項5 useState(false)パターンより妥当な推測

    mockedGetItemDetail.mockResolvedValueOnce({
      success: true,
      data: makeDetailItem({ id: 'item-1', title: '星屑のシンフォニア' }),
    })

    renderItemDetailPage('item-1')

    await waitFor(() => {
      expect(screen.getByRole('heading', { level: 1 })).toBeInTheDocument()
    })

    // 【結果検証】: ダイアログ開閉stateの初期状態を確認する
    // 【堅牢性の確認】: 初期レンダリングで誤ってダイアログが開かないこと
    expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument() // 【確認内容】: 初期表示時にDeleteConfirmDialogが開いていないこと 🟡
    expect(screen.getByRole('button', { name: '削除' })).toBeInTheDocument() // 【確認内容】: 削除ボタン自体は表示されていること 🟡
  })
})
