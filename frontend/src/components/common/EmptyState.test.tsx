import { describe, expect, it, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { EmptyState } from './EmptyState'

describe('EmptyState', () => {
  // ===== 正常系 =====

  it('TC-ES-N-01: message が表示される', () => {
    // 【テスト目的】: 必須 prop message がそのまま描画されることを確認する
    // 【テスト内容】: message='コレクションがありません' を渡し、テキストが表示されるか検証する
    // 【期待される動作】: 「コレクションがありません」等のメッセージが表示される
    // 🔵 信頼性レベル: requirements.md「2.4」, EDGE-101, note.md より確実

    // 【テストデータ準備】: EDGE-101 で例示される代表的な空状態メッセージ
    // 【初期条件設定】: message のみ指定
    render(<EmptyState message="コレクションがありません" />)

    // 【結果検証】: message 文字列が DOM に存在するか確認する
    // 【期待値確認】: message 文字列が改変されず表示されること
    expect(screen.getByText('コレクションがありません')).toBeInTheDocument() // 【確認内容】: 空状態メッセージが正しく表示されることを確認
  })

  it('TC-ES-N-02: actionLabel + onAction 指定時にアクションボタンが表示される', () => {
    // 【テスト目的】: actionLabel + onAction が両方指定されたときボタンが描画されることを確認する
    // 【テスト内容】: actionLabel='アイテムを追加' を渡し、ボタンが表示されるか検証する
    // 【期待される動作】: ボタン要素に actionLabel のテキストが表示される
    // 🟡 信頼性レベル: requirements.md「2.4」+ EDGE-101「追加画面への導線」より妥当な推測

    // 【テストデータ準備】: 追加導線（EDGE-101「追加画面への導線」）の代表ケース
    // 【初期条件設定】: actionLabel と onAction を両方指定
    render(
      <EmptyState
        message="コレクションがありません"
        actionLabel="アイテムを追加"
        onAction={vi.fn()}
      />
    )

    // 【結果検証】: ボタンが role="button" かつ名前一致で取得できるか確認する
    // 【期待値確認】: actionLabel がボタンの可視ラベルとして反映されること
    expect(screen.getByRole('button', { name: 'アイテムを追加' })).toBeInTheDocument() // 【確認内容】: アクション導線ボタンが正しいラベルで表示されることを確認
  })

  it('TC-ES-N-03: アクションボタンクリックで onAction が呼ばれる', () => {
    // 【テスト目的】: ボタンクリックで onAction が1回呼ばれることを確認する
    // 【テスト内容】: onAction=vi.fn() を渡しボタンをクリックして呼び出し回数を検証する
    // 【期待される動作】: クリックイベントで onAction() が発火する
    // 🟡 信頼性レベル: requirements.md「2.4」, note.md「テスト対象: ボタンクリックで onAction が呼ばれる」より妥当な推測

    // 【テストデータ準備】: 追加画面遷移トリガとしてのクリック導線を検証
    // 【初期条件設定】: actionLabel と onAction（モック）を指定
    const onAction = vi.fn()
    render(
      <EmptyState message="コレクションがありません" actionLabel="アイテムを追加" onAction={onAction} />
    )

    // 【実際の処理実行】: アクションボタンをクリックする
    // 【処理内容】: fireEvent.click でクリックイベントを発火する
    fireEvent.click(screen.getByRole('button', { name: 'アイテムを追加' }))

    // 【結果検証】: onAction が1回呼ばれたことを確認する
    // 【期待値確認】: クリック1回で正確に1回呼ばれること
    expect(onAction).toHaveBeenCalledTimes(1) // 【確認内容】: アクションコールバックが過不足なく1回発火することを確認
  })

  // ===== 異常系 =====

  it('TC-ES-E-01: actionLabel 指定で onAction 未指定時もクリックでクラッシュしない', () => {
    // 【テスト目的】: actionLabel のみ指定（onAction 省略）でクリックしても例外が出ないことを確認する
    // 【テスト内容】: onAction を渡さずボタンが描画される場合にクリックして例外が出ないことを検証する
    // 【期待される動作】: クリックしても TypeError（undefined 呼び出し）が発生しない
    // 🟡 信頼性レベル: requirements.md「2.4 入出力の関係性」からの妥当な推測

    // 【テストデータ準備】: 呼び出し側の prop 渡し漏れを想定
    // 【初期条件設定】: actionLabel のみ指定し onAction を渡さない
    render(<EmptyState message="コレクションがありません" actionLabel="追加" />)

    // 【実際の処理実行】: ボタンが存在する場合にクリックする（描画されない実装の場合は queryByRole が null）
    // 【処理内容】: クリック操作が例外を投げないことを確認する
    const button = screen.queryByRole('button', { name: '追加' })
    expect(() => {
      if (button) fireEvent.click(button)
    }).not.toThrow() // 【確認内容】: undefined コールバックへの安全なガードがされていることを確認
  })

  // ===== 境界値 =====

  it('TC-ES-B-01: actionLabel/onAction 省略時はメッセージのみ表示しボタンを描画しない', () => {
    // 【テスト目的】: action 系 props 省略時にボタンを描画しないことを確認する
    // 【テスト内容】: message のみ指定し、queryByRole('button') が null になるか検証する
    // 【期待される動作】: メッセージのみの最小構成が成立する
    // 🟡 信頼性レベル: requirements.md「4.2 EmptyState（actionなし）: メッセージのみ表示」より妥当な推測

    // 【テストデータ準備】: 単純な空状態（導線不要）の表示を想定
    // 【初期条件設定】: actionLabel/onAction を渡さない
    render(<EmptyState message="コレクションがありません" />)

    // 【結果検証】: メッセージは表示され、ボタンは存在しないことを確認する
    // 【期待値確認】: action 未指定時にボタンを出さないこと
    expect(screen.getByText('コレクションがありません')).toBeInTheDocument() // 【確認内容】: メッセージ自体は表示されることを確認
    expect(screen.queryByRole('button')).toBeNull() // 【確認内容】: 余計なボタンが描画されないことを確認
  })

  it('TC-ES-B-02: message が空文字でもエラーなく描画される', () => {
    // 【テスト目的】: message 空文字でも EmptyState がクラッシュしないことを確認する
    // 【テスト内容】: message='' を渡し、例外が発生せずコンテナが描画されるか検証する
    // 【期待される動作】: 例外なくコンテナが描画される
    // 🟡 信頼性レベル: message は必須 string。空文字扱いは明記なし（妥当な推測）

    // 【テストデータ準備】: メッセージ未設定時の防御的描画を想定
    // 【初期条件設定】: message を空文字に設定
    let container: HTMLElement | undefined

    // 【実際の処理実行】: EmptyState を空文字 message でレンダリングする
    // 【処理内容】: 空文字を許容しつつ破綻しないことを確認する
    expect(() => {
      const result = render(<EmptyState message="" />)
      container = result.container
    }).not.toThrow() // 【確認内容】: 空文字 message でも例外が発生しないことを確認

    // 【結果検証】: 空状態コンテナが存在することを確認する
    // 【期待値確認】: 空・非空で描画構造が一貫していること
    expect(container?.querySelector('[data-testid="empty-state"]')).not.toBeNull() // 【確認内容】: message 空文字時もコンテナ自体は描画されることを確認
  })

  // ===== スタイル回帰確認（TASK-0005） =====

  it('TC-ES-S-01: ルート要素に .empty-state クラスが付与される', () => {
    // 【テスト目的】: _shared.css .empty-state 相当の中央揃え・パディングスタイルが適用されることを確認する
    // 🔵 信頼性レベル: _shared.css .empty-state定義より
    render(<EmptyState message="コレクションがありません" />)

    expect(screen.getByTestId('empty-state')).toHaveClass('empty-state') // 【確認内容】: ルート要素にスタイル用クラスが付与されていることを確認
  })

  it('TC-ES-S-02: 見出し部分に .empty-state-heading クラス（Source Serif 4フォント適用）が付与される', () => {
    // 【テスト目的】: message 表示要素に見出し相当のクラスが付与されることを確認する
    // 🟡 信頼性レベル: タスク指示「見出しにSource Serif 4フォント」からの妥当な推測
    render(<EmptyState message="コレクションがありません" />)

    expect(screen.getByText('コレクションがありません')).toHaveClass('empty-state-heading') // 【確認内容】: 見出し要素にフォントスタイル用クラスが付与されていることを確認
  })
})
