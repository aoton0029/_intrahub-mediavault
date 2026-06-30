import { describe, expect, it } from 'vitest'
import { render, screen } from '@testing-library/react'
import { MediaTypeBadge } from './MediaTypeBadge'
import type { MediaType } from '@/types'

describe('MediaTypeBadge', () => {
  // ===== 正常系 =====

  it('TC-MB-N-01: anime のラベルとアクセントクラスが適用される', () => {
    // 【テスト目的】: mediaType='anime' に対し「アニメ」ラベルと text-accent-anime クラスが適用されることを確認する
    // 【テスト内容】: anime を渡してレンダリングし、ラベルとクラスの両方を検証する
    // 【期待される動作】: 「アニメ」テキストを持つ要素に text-accent-anime クラスが付与される
    // 🔵🟡 信頼性レベル: media-type-accent.ts（クラス確認・確実）+ 🟡 日本語ラベルは妥当な推測

    // 【テストデータ準備】: 代表的な media_type（8種別の先頭）を使用
    // 【初期条件設定】: mediaType='anime' を指定
    const { container } = render(<MediaTypeBadge mediaType="anime" />)

    // 【結果検証】: ラベルとアクセントクラスの両方を確認する
    // 【期待値確認】: getMediaTypeAccentClass の戻り値が実際に className へ反映されていること
    expect(screen.getByText('アニメ')).toBeInTheDocument() // 【確認内容】: 日本語ラベル「アニメ」が表示されることを確認
    expect(container.querySelector('.text-accent-anime')).not.toBeNull() // 【確認内容】: media-type-accent.ts 由来のクラスが適用されていることを確認
  })

  it('TC-MB-N-02: アクセントクラスはハードコードでなく getMediaTypeAccentClass 経由で適用される', () => {
    // 【テスト目的】: 各種別のクラス文字列が media-type-accent.ts の定義と一致することを確認する
    // 【テスト内容】: movie/manga を渡し、対応するアクセントクラスが反映されるか検証する
    // 【期待される動作】: movie→text-accent-movie, manga→text-accent-manga が反映される
    // 🔵 信頼性レベル: media-type-accent.ts L11-20（実体確認）, requirements.md「3. 制約条件」より確実

    // 【テストデータ準備】: anime 以外でクラスマッピングが正しいか確認する代表値
    // 【初期条件設定】: movie と manga の2種別を検証
    const { container: movieContainer } = render(<MediaTypeBadge mediaType="movie" />)
    const { container: mangaContainer } = render(<MediaTypeBadge mediaType="manga" />)

    // 【結果検証】: それぞれのバッジ要素に対応するアクセントクラスが含まれることを確認する
    // 【期待値確認】: 色のハードコードでなく関数戻り値に依存していること
    expect(movieContainer.querySelector('.text-accent-movie')).not.toBeNull() // 【確認内容】: movie のアクセントクラスが正しく適用されていることを確認
    expect(mangaContainer.querySelector('.text-accent-manga')).not.toBeNull() // 【確認内容】: manga のアクセントクラスが正しく適用されていることを確認
  })

  // ===== 異常系 =====

  it('TC-MB-E-01: 想定外の文字列を渡してもクラッシュしない（型外入力の防御）', () => {
    // 【テスト目的】: MediaType の8値に含まれない文字列が渡されてもクラッシュしないことを確認する
    // 【テスト内容】: 'unknown' を強制キャストして渡し、例外が出ないことを検証する
    // 【期待される動作】: 例外を throw せずに描画される（クラスが undefined/空でもバッジ枠は出る）
    // 🔴 信頼性レベル: 設計文書に明記なし。一般的な防御的UI観点からの推測（実装方針により skip 可）

    // 【テストデータ準備】: バックエンド種別追加・型ズレ・破損データ受信時を想定
    // 【初期条件設定】: mediaType に型を逸脱した値を渡す
    const invalidMediaType = 'unknown' as MediaType

    // 【実際の処理実行】: MediaTypeBadge をレンダリングする
    // 【処理内容】: 型外入力でも例外が発生しないことを確認する
    expect(() => render(<MediaTypeBadge mediaType={invalidMediaType} />)).not.toThrow() // 【確認内容】: undefined クラスで React が落ちないことを確認
  })

  // ===== 境界値 =====

  it.each([
    ['anime', 'text-accent-anime'],
    ['movie', 'text-accent-movie'],
    ['drama', 'text-accent-drama'],
    ['manga', 'text-accent-manga'],
    ['novel', 'text-accent-novel'],
    ['game', 'text-accent-game'],
    ['academic_book', 'text-accent-academic-book'],
    ['paper', 'text-accent-paper'],
  ] as [MediaType, string][])(
    'TC-MB-B-01: mediaType=%s で %s クラスが適用される',
    (mediaType, accentClass) => {
      // 【テスト目的】: MediaType 全列挙値で例外なく描画され、対応クラスが適用されることを確認する
      // 【テスト内容】: it.each で8種別を反復し、各 className に対応アクセントクラスが含まれるか検証する
      // 【期待される動作】: 全分岐でラベル描画とクラス適用が成立する
      // 🔵 信頼性レベル: media-type-accent.ts L11-20（8キー・クラス名を実体確認）, requirements.md「4.2 8種別網羅」より確実

      // 【テストデータ準備】: MediaType の取りうる全値（media-type-accent.ts の8キー）
      // 【初期条件設定】: mediaType と期待アクセントクラスの組を反復
      const { container } = render(<MediaTypeBadge mediaType={mediaType} />)

      // 【結果検証】: 対応するアクセントクラスが className に含まれることを確認する
      // 【期待値確認】: academic_book→text-accent-academic-book（アンダースコア→ハイフン変換）も正確であること
      expect(container.querySelector(`.${accentClass}`)).not.toBeNull() // 【確認内容】: 各種別で対応するアクセントクラスが正確に適用されることを確認
    }
  )
})
