import { describe, expect, it, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { MediaCard } from './MediaCard'
import type { Item } from '@/types'

// 【共通フィクスチャ】: Item判別共用体の必須フィールドを満たす最小 anime Item 生成ヘルパー
// 🔵 frontend/src/types/index.ts ItemBase/AnimeDetails の必須フィールドより
const makeAnimeItem = (overrides?: Partial<Item>): Item =>
  ({
    id: 'item-1',
    title: 'テストアニメ',
    status: 'not_started',
    isFavorite: false,
    source: 'manual',
    createdAt: '2026-01-01T00:00:00Z',
    updatedAt: '2026-01-01T00:00:00Z',
    mediaType: 'anime',
    details: { genreList: [] },
    ...overrides,
  }) as Item

describe('MediaCard', () => {
  // ===== 正常系 =====

  it('TC-MC-N-01: アイテムのタイトルが表示される', () => {
    // 【テスト目的】: item.title がカード内に表示されることを確認する
    // 【テスト内容】: makeAnimeItem で生成した Item を渡し、title 文字列の描画を検証する
    // 【期待される動作】: 渡した item.title の文字列が DOM に出力される
    // 🔵 信頼性レベル: requirements.md「2.1 MediaCard 出力」, note.md より確実

    // 【テストデータ準備】: 一覧表示で最も重要な識別情報であるタイトルを検証するため代表的な日本語タイトルを使用
    // 【初期条件設定】: details.genreList=[] 等、判別共用体の必須フィールドを満たす
    const item = makeAnimeItem({ title: 'テストアニメ' })

    // 【実際の処理実行】: MediaCard をレンダリングする
    // 【処理内容】: render() で jsdom 上に MediaCard をマウントする
    render(<MediaCard item={item} />)

    // 【結果検証】: title 文字列が DOM に存在するか検証する
    // 【期待値確認】: getByText('テストアニメ') が要素を返すこと
    expect(screen.getByText('テストアニメ')).toBeInTheDocument() // 【確認内容】: タイトルが欠落・改変されずに表示されることを確認
  })

  it('TC-MC-N-02: mediaType に応じた MediaTypeBadge が表示される', () => {
    // 【テスト目的】: カード内に当該 mediaType のバッジ（日本語ラベル）が描画されることを確認する
    // 【テスト内容】: mediaType='anime' のとき「アニメ」ラベルのバッジが描画されることを検証する
    // 【期待される動作】: MediaCard が MediaTypeBadge を内包し、ラベルが委譲表示される
    // 🔵 信頼性レベル: requirements.md「2.1 出力（mediaType→MediaTypeBadge）」より確実

    // 【テストデータ準備】: バッジ委譲の代表ケースとして anime を使用
    // 【初期条件設定】: AnimeDetails の必須フィールド genreList を満たす
    const item = makeAnimeItem({ mediaType: 'anime', details: { genreList: [] } })

    // 【実際の処理実行】: MediaCard をレンダリングする
    // 【処理内容】: MediaTypeBadge への委譲が行われるかを確認する
    render(<MediaCard item={item} />)

    // 【結果検証】: MediaTypeBadge 経由のラベルが表示されているか検証する
    // 【期待値確認】: 「アニメ」ラベルが DOM に存在すること
    expect(screen.getByText('アニメ')).toBeInTheDocument() // 【確認内容】: MediaCard が MediaTypeBadge を内包し委譲表示していることを確認
  })

  it('TC-MC-N-03: onClick 指定時にカードクリックで onClick(item) が呼ばれる', () => {
    // 【テスト目的】: カードクリックで onClick コールバックが item を引数に1回呼ばれることを確認する
    // 【テスト内容】: onClick=vi.fn() を渡し、カードをクリックして呼び出し回数・引数を検証する
    // 【期待される動作】: クリックイベントで onClick(item) が発火する
    // 🔵 信頼性レベル: requirements.md「2.1」, note.md より確実

    // 【テストデータ準備】: 一覧→詳細遷移の起点となるクリック導線を検証するためモック関数を用意
    // 【初期条件設定】: onClick コールバックと item を準備
    const onClick = vi.fn()
    const item = makeAnimeItem()
    render(<MediaCard item={item} onClick={onClick} />)

    // 【実際の処理実行】: タイトルテキストを起点にカード要素をクリックする
    // 【処理内容】: fireEvent.click でクリックイベントを発火する
    fireEvent.click(screen.getByText(item.title))

    // 【結果検証】: onClick の呼び出し回数と引数を確認する
    // 【期待値確認】: 1回だけ呼ばれ、引数が item と一致すること
    expect(onClick).toHaveBeenCalledTimes(1) // 【確認内容】: 過不足なく1回だけ発火することを確認
    expect(onClick).toHaveBeenCalledWith(item) // 【確認内容】: 当該 item がそのまま引数に渡ることを確認
  })

  it('TC-MC-N-04: coverImageUrl 指定時に画像（cover）が表示される', () => {
    // 【テスト目的】: item.coverImageUrl が img の src として描画されることを確認する
    // 【テスト内容】: coverImageUrl を指定し、画像要素の src が一致するか検証する
    // 【期待される動作】: 画像要素の参照先 URL が渡した値になる
    // 🟡 信頼性レベル: requirements.md「2.1」で画像表示は明記だが img role/alt 等の具体実装は推測

    // 【テストデータ準備】: 一覧でのビジュアル識別に使われるカバー画像の代表 URL
    // 【初期条件設定】: coverImageUrl を明示的に設定
    const item = makeAnimeItem({ coverImageUrl: 'https://example.com/cover.jpg' })

    // 【実際の処理実行】: MediaCard をレンダリングする
    render(<MediaCard item={item} />)

    // 【結果検証】: img 要素の src 属性を確認する
    // 【期待値確認】: 渡した URL が正しく src に反映されていること
    const img = screen.getByRole('img')
    expect(img).toHaveAttribute('src', 'https://example.com/cover.jpg') // 【確認内容】: カバー画像 URL が img の src に正しく反映されていることを確認
  })

  it('TC-MC-N-05: isFavorite=true のときお気に入り表示がされる', () => {
    // 【テスト目的】: isFavorite=true のときお気に入り状態を示す要素が描画されることを確認する
    // 【テスト内容】: isFavorite=true を指定し、data-favorite 属性等の表示を検証する
    // 【期待される動作】: お気に入りインジケータが表示状態になる
    // 🟡 信頼性レベル: requirements.md「2.1」で表示は明記だが具体的表現は実装時決定の推測

    // 【テストデータ準備】: お気に入りON状態の代表ケース
    // 【初期条件設定】: isFavorite=true を明示的に設定
    const item = makeAnimeItem({ isFavorite: true })

    // 【実際の処理実行】: MediaCard をレンダリングする
    render(<MediaCard item={item} />)

    // 【結果検証】: お気に入りを示す要素（data-favorite="true"）が存在するか確認する
    // 【期待値確認】: ON 状態が判別可能な形でレンダリングされていること
    const favoriteIndicator = screen.getByTestId('media-card-favorite')
    expect(favoriteIndicator).toHaveAttribute('data-favorite', 'true') // 【確認内容】: お気に入りON状態が視覚的に判別可能な形で反映されていることを確認
  })

  it('TC-MC-N-06: status が視覚表示される', () => {
    // 【テスト目的】: item.status が表示されることを確認する
    // 【テスト内容】: status='in_progress' を指定し、対応する表示要素を検証する
    // 【期待される動作】: 各 status を示すラベルまたは属性が描画される
    // 🟡 信頼性レベル: requirements.md「2.1」で表示は明記だが表示形式は推測

    // 【テストデータ準備】: 進行中状態の代表ケース（3状態のうち中間値）
    // 【初期条件設定】: status='in_progress' を明示的に設定
    const item = makeAnimeItem({ status: 'in_progress' })

    // 【実際の処理実行】: MediaCard をレンダリングする
    render(<MediaCard item={item} />)

    // 【結果検証】: status を示す要素（data-status="in_progress"）が存在するか確認する
    // 【期待値確認】: 渡した status と表示が対応していること
    const statusIndicator = screen.getByTestId('media-card-status')
    expect(statusIndicator).toHaveAttribute('data-status', 'in_progress') // 【確認内容】: 渡した status が表示に正しく反映されていることを確認
  })

  // ===== 異常系 =====

  it('TC-MC-E-01: onClick 未指定でもクリックしてエラーにならない', () => {
    // 【テスト目的】: onClick 省略時にカードクリックしても例外が発生しないことを確認する
    // 【テスト内容】: onClick を渡さずにクリックし、クラッシュしないことを検証する
    // 【期待される動作】: クリックしても例外を throw せず、レンダリングが維持される
    // 🟡 信頼性レベル: requirements.md「4.2 MediaCard（onClickなし）」から妥当な推測

    // 【テストデータ準備】: クリック不要なプレビュー用途等で onClick を渡さない場面を想定
    // 【初期条件設定】: onClick を渡さない Item を用意
    const item = makeAnimeItem()
    render(<MediaCard item={item} />)

    // 【実際の処理実行】: onClick 未指定の状態でカードをクリックする
    // 【処理内容】: クリック実行が例外を投げないことを確認する
    expect(() => fireEvent.click(screen.getByText(item.title))).not.toThrow() // 【確認内容】: undefined を呼び出す TypeError が起きないことを確認

    // 【結果検証】: クリック後もタイトルが引き続き表示されていることを確認する
    // 【期待値確認】: レンダリングが破綻せず維持されていること
    expect(screen.getByText(item.title)).toBeInTheDocument() // 【確認内容】: クリック後もカードの描画が維持されていることを確認
  })

  it('TC-MC-E-02: coverImageUrl 欠落時にプレースホルダ表示で描画が破綻しない', () => {
    // 【テスト目的】: coverImageUrl が undefined でもエラーなく描画されることを確認する
    // 【テスト内容】: coverImageUrl を未設定にし、title 等が表示され例外が出ないことを検証する
    // 【期待される動作】: 画像欠落はプレースホルダ等で吸収され例外が出ない
    // 🟡 信頼性レベル: requirements.md「4.2 MediaCard（任意項目欠落）」から妥当な推測

    // 【テストデータ準備】: source='manual' で画像未設定のアイテム表示時を想定
    // 【初期条件設定】: coverImageUrl を undefined に設定
    const item = makeAnimeItem({ coverImageUrl: undefined })

    // 【実際の処理実行】: MediaCard をレンダリングする
    // 【処理内容】: coverImageUrl 欠落時の描画が例外を投げないことを確認する
    expect(() => render(<MediaCard item={item} />)).not.toThrow() // 【確認内容】: 画像欠落でカード全体が消えない・例外で落ちないことを確認

    // 【結果検証】: title は引き続き表示されることを確認する
    // 【期待値確認】: 画像欠落時も他の要素は正常に描画されること
    expect(screen.getByText(item.title)).toBeInTheDocument() // 【確認内容】: 画像欠落時もタイトル等の他要素は正常に描画されることを確認
  })

  // ===== 境界値 =====

  it.each([
    'anime',
    'movie',
    'drama',
    'manga',
    'novel',
    'game',
    'academic_book',
    'paper',
  ] as const)('TC-MC-B-01: mediaType=%s の Item をエラーなく描画する', (mediaType) => {
    // 【テスト目的】: 8種別すべての Item をエラーなく描画できることを確認する
    // 【テスト内容】: it.each で全 MediaType を反復し、各種別の details 必須フィールドを満たして描画する
    // 【期待される動作】: いずれの種別でも例外が発生せず title が表示される
    // 🔵 信頼性レベル: frontend/src/types/index.ts MediaType（8種別）, requirements.md「4.2 8種別網羅」より確実

    // 【テストデータ準備】: 種別ごとに異なる details 必須フィールドを満たす最小データを用意
    // 【初期条件設定】: details は種別ごとの必須配列フィールド（genreList/platformList/authorList）を考慮
    const detailsMap: Record<string, unknown> = {
      anime: { genreList: [] },
      movie: { genreList: [] },
      drama: { genreList: [] },
      manga: {},
      novel: {},
      game: { platformList: [] },
      academic_book: {},
      paper: { authorList: [] },
    }
    const item = makeAnimeItem({
      mediaType,
      title: `タイトル-${mediaType}`,
      details: detailsMap[mediaType],
    } as Partial<Item>)

    // 【実際の処理実行】: MediaCard をレンダリングする
    // 【処理内容】: 判別共用体の全分岐で描画が成立するかを確認する
    expect(() => render(<MediaCard item={item} />)).not.toThrow() // 【確認内容】: details の型が種別ごとに異なっても例外なく描画されることを確認

    // 【結果検証】: 該当 mediaType のタイトルが表示されることを確認する
    // 【期待値確認】: 全種別で title 描画とバッジ委譲が一貫していること
    expect(screen.getByText(`タイトル-${mediaType}`)).toBeInTheDocument() // 【確認内容】: 種別によらずタイトルが正しく表示されることを確認
  })

  it('TC-MC-B-02: タイトルが空文字でも描画が破綻しない', () => {
    // 【テスト目的】: title が空文字でも MediaCard がクラッシュしないことを確認する
    // 【テスト内容】: title='' を指定し、例外が発生せずカードコンテナが描画されることを検証する
    // 【期待される動作】: 例外が発生せず、カードコンテナがレンダリングされる
    // 🟡 信頼性レベル: title は必須 string。空文字の扱いは設計文書に明記なし（妥当な推測）

    // 【テストデータ準備】: データ不整合や未入力タイトルの防御的描画を想定
    // 【初期条件設定】: title を空文字に設定
    const item = makeAnimeItem({ title: '' })

    // 【実際の処理実行】: MediaCard をレンダリングする
    // 【処理内容】: 空文字タイトルでも例外を投げないことを確認する
    let container: HTMLElement | undefined
    expect(() => {
      const result = render(<MediaCard item={item} />)
      container = result.container
    }).not.toThrow() // 【確認内容】: 空文字を許容しつつ描画破綻しないことを確認

    // 【結果検証】: カードコンテナ要素が存在することを確認する
    // 【期待値確認】: 空・非空で描画構造が一貫していること
    expect(container?.querySelector('[data-testid="media-card"]')).not.toBeNull() // 【確認内容】: タイトル空文字時もカード枠自体は描画されることを確認
  })
})
