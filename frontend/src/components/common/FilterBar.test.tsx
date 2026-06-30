import { describe, expect, it } from 'vitest'
import { render, screen } from '@testing-library/react'
import { FilterBar } from './FilterBar'

describe('FilterBar', () => {
  // ===== 正常系 =====

  it('TC-FB-N-01: children が正しくレンダリングされる', () => {
    // 【テスト目的】: 渡した子要素がコンテナ内に出力されることを確認する
    // 【テスト内容】: <FilterBar><button>絞り込み</button></FilterBar> として描画し、テキストが出力されるか検証する
    // 【期待される動作】: 「絞り込み」テキストが描画される
    // 🔵 信頼性レベル: requirements.md「2.3 出力（children をそのまま内包）」, note.md より確実

    // 【テストデータ準備】: 後続タスクで差し込む絞り込みUIの代理として任意の ReactNode を使用
    // 【初期条件設定】: button 要素を children として渡す
    render(
      <FilterBar>
        <button>絞り込み</button>
      </FilterBar>
    )

    // 【結果検証】: children のテキストが DOM に存在するか確認する
    // 【期待値確認】: children が改変されず描画されること
    expect(screen.getByText('絞り込み')).toBeInTheDocument() // 【確認内容】: children がそのまま描画されることを確認
  })

  // ===== 異常系 =====

  it('TC-FB-E-01: children に複数要素を渡しても全て描画される', () => {
    // 【テスト目的】: 単一でなく複数子要素を渡しても全て描画されることを確認する
    // 【テスト内容】: 2つの span 要素を children として渡し、両方表示されるか検証する
    // 【期待される動作】: 「A」「B」両方が描画される
    // 🟡 信頼性レベル: requirements.md「2.3」から妥当な推測（複数子は ReactNode の一般仕様）

    // 【テストデータ準備】: media_type/タグ/status 等、複数のフィルタ要素を並置する場面を想定
    // 【初期条件設定】: 配列形式で複数の span を children に渡す
    render(
      <FilterBar>
        <span key="a">A</span>
        <span key="b">B</span>
      </FilterBar>
    )

    // 【結果検証】: 両方のテキストが存在することを確認する
    // 【期待値確認】: 複数子で描画が欠落しないこと
    expect(screen.getByText('A')).toBeInTheDocument() // 【確認内容】: 1つ目の子要素が描画されることを確認
    expect(screen.getByText('B')).toBeInTheDocument() // 【確認内容】: 2つ目の子要素も漏れなく描画されることを確認
  })

  // ===== 境界値 =====

  it('TC-FB-B-01: children 未指定でもコンテナのみエラーなく描画される', () => {
    // 【テスト目的】: children 省略時にコンテナのみエラーなく描画されることを確認する
    // 【テスト内容】: <FilterBar /> として描画し、例外が発生しないことを検証する
    // 【期待される動作】: 例外なくコンテナ要素がレンダリングされる
    // 🟡 信頼性レベル: requirements.md「2.3」「4.2」より妥当な推測

    // 【テストデータ準備】: 初期表示やフィルタ要素未配置時の器のみ描画を想定
    // 【初期条件設定】: children を渡さない
    let container: HTMLElement | undefined

    // 【実際の処理実行】: FilterBar を children なしでレンダリングする
    // 【処理内容】: undefined children を安全に無視できるか確認する
    expect(() => {
      const result = render(<FilterBar />)
      container = result.container
    }).not.toThrow() // 【確認内容】: children 未指定でも例外が発生しないことを確認

    // 【結果検証】: コンテナ要素が描画されていることを確認する
    // 【期待値確認】: 子あり・なしで器構造が一貫していること
    expect(container?.querySelector('[data-testid="filter-bar"]')).not.toBeNull() // 【確認内容】: children なしでも器（コンテナ）自体は描画されることを確認
  })
})
