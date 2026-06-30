import { describe, expect, it, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { ConfirmDialog } from './ConfirmDialog'

describe('ConfirmDialog', () => {
  // ===== 正常系 =====

  it('TC-CD-N-01: open=true で title と description が表示される', () => {
    // 【テスト目的】: 表示状態でタイトルと本文が描画されることを確認する
    // 【テスト内容】: open=true, title, description を渡し、両方が DOM に出力されるか検証する
    // 【期待される動作】: title・description 文字列が DOM に出力される
    // 🟡 信頼性レベル: requirements.md「2.5」より妥当な推測（shadcn Dialog ベース）

    // 【テストデータ準備】: 削除確認（REQ-007）の代表ケース
    // 【初期条件設定】: open=true, title, description, onConfirm/onCancel を指定
    render(
      <ConfirmDialog
        open={true}
        title="削除しますか？"
        description="この操作は取り消せません"
        onConfirm={vi.fn()}
        onCancel={vi.fn()}
      />
    )

    // 【結果検証】: title と description が表示されているか確認する
    // 【期待値確認】: タイトル・本文の双方が表示されること
    expect(screen.getByText('削除しますか？')).toBeInTheDocument() // 【確認内容】: ダイアログタイトルが表示されることを確認
    expect(screen.getByText('この操作は取り消せません')).toBeInTheDocument() // 【確認内容】: ダイアログ本文（description）が表示されることを確認
  })

  it('TC-CD-N-02: 確認ボタンクリックで onConfirm が呼ばれる', () => {
    // 【テスト目的】: 確認ボタンクリックで onConfirm が1回呼ばれることを確認する
    // 【テスト内容】: confirmLabel='削除' を渡し、ボタンをクリックして呼び出し回数を検証する
    // 【期待される動作】: クリックで onConfirm() が発火する
    // 🟡 信頼性レベル: requirements.md「2.5」, note.md より妥当な推測

    // 【テストデータ準備】: 削除実行トリガとなる確認操作を検証
    // 【初期条件設定】: confirmLabel='削除' と onConfirm（モック）を指定
    const onConfirm = vi.fn()
    render(
      <ConfirmDialog
        open={true}
        title="削除しますか？"
        confirmLabel="削除"
        onConfirm={onConfirm}
        onCancel={vi.fn()}
      />
    )

    // 【実際の処理実行】: 確認ボタンをクリックする
    // 【処理内容】: fireEvent.click でクリックイベントを発火する
    fireEvent.click(screen.getByRole('button', { name: '削除' }))

    // 【結果検証】: onConfirm が1回呼ばれたことを確認する
    // 【期待値確認】: 確認ボタンと onConfirm が正しく結線されていること
    expect(onConfirm).toHaveBeenCalledTimes(1) // 【確認内容】: 確認コールバックが過不足なく1回発火することを確認
  })

  it('TC-CD-N-03: キャンセルボタンクリックで onCancel が呼ばれる', () => {
    // 【テスト目的】: キャンセルボタンクリックで onCancel が1回呼ばれ、onConfirm は呼ばれないことを確認する
    // 【テスト内容】: cancelLabel='キャンセル' を渡し、ボタンをクリックして呼び出しを検証する
    // 【期待される動作】: クリックで onCancel() が発火し、onConfirm は呼ばれない
    // 🟡 信頼性レベル: requirements.md「2.5」, note.md より妥当な推測

    // 【テストデータ準備】: 操作中止トリガとなるキャンセル操作を検証
    // 【初期条件設定】: cancelLabel='キャンセル' と onCancel/onConfirm（モック）を指定
    const onConfirm = vi.fn()
    const onCancel = vi.fn()
    render(
      <ConfirmDialog
        open={true}
        title="削除しますか？"
        cancelLabel="キャンセル"
        onConfirm={onConfirm}
        onCancel={onCancel}
      />
    )

    // 【実際の処理実行】: キャンセルボタンをクリックする
    // 【処理内容】: fireEvent.click でクリックイベントを発火する
    fireEvent.click(screen.getByRole('button', { name: 'キャンセル' }))

    // 【結果検証】: onCancel が1回呼ばれ、onConfirm が呼ばれていないことを確認する
    // 【期待値確認】: キャンセル操作で onConfirm が誤って呼ばれないこと
    expect(onCancel).toHaveBeenCalledTimes(1) // 【確認内容】: キャンセルコールバックが過不足なく1回発火することを確認
    expect(onConfirm).not.toHaveBeenCalled() // 【確認内容】: キャンセル操作で確認コールバックが誤発火しないことを確認
  })

  it('TC-CD-N-04: confirmLabel/cancelLabel 指定時にラベルがボタンへ反映される', () => {
    // 【テスト目的】: 任意ラベル prop がボタンの可視テキストになることを確認する
    // 【テスト内容】: confirmLabel='削除', cancelLabel='やめる' を渡し、ボタン名を検証する
    // 【期待される動作】: confirmLabel='削除', cancelLabel='やめる' がそれぞれボタンに出る
    // 🟡 信頼性レベル: requirements.md「2.5 入力（confirmLabel/cancelLabel 任意）」より妥当な推測

    // 【テストデータ準備】: 操作に応じたカスタムラベル（削除文脈）の代表ケース
    // 【初期条件設定】: confirmLabel='削除', cancelLabel='やめる' を指定
    render(
      <ConfirmDialog
        open={true}
        title="削除しますか？"
        confirmLabel="削除"
        cancelLabel="やめる"
        onConfirm={vi.fn()}
        onCancel={vi.fn()}
      />
    )

    // 【結果検証】: それぞれのラベルを持つボタンが存在することを確認する
    // 【期待値確認】: 渡したラベルがそのままボタン名になること
    expect(screen.getByRole('button', { name: '削除' })).toBeInTheDocument() // 【確認内容】: confirmLabel がボタンの可視テキストに反映されることを確認
    expect(screen.getByRole('button', { name: 'やめる' })).toBeInTheDocument() // 【確認内容】: cancelLabel がボタンの可視テキストに反映されることを確認
  })

  // ===== 異常系 =====

  it('TC-CD-E-01: 連続クリックでも onConfirm が呼ばれる（多重発火の検証）', () => {
    // 【テスト目的】: 確認ボタンを2回クリックすると onConfirm が2回呼ばれることを確認する
    // 【テスト内容】: 確認ボタンを連続でクリックし、呼び出し回数を検証する
    // 【期待される動作】: onConfirm が2回呼ばれる（本コンポーネントは多重発火抑制を持たず素直に伝播する）
    // 🔴 信頼性レベル: 設計文書に明記なし。controlled コンポーネントの一般挙動からの推測（実装方針により期待値変更可）

    // 【テストデータ準備】: ユーザーの素早い連打を想定
    // 【初期条件設定】: onConfirm（モック）を指定
    const onConfirm = vi.fn()
    render(
      <ConfirmDialog open={true} title="削除しますか？" onConfirm={onConfirm} onCancel={vi.fn()} />
    )

    // 【実際の処理実行】: 確認ボタンを2回連続でクリックする
    // 【処理内容】: fireEvent.click を2回発火する
    const confirmButton = screen.getByRole('button', { name: /確認|OK|削除/ })
    fireEvent.click(confirmButton)
    fireEvent.click(confirmButton)

    // 【結果検証】: onConfirm が2回呼ばれたことを確認する
    // 【期待値確認】: コンポーネント単体は伝播に徹し、冪等性は呼び出し側責務であることを確認
    expect(onConfirm).toHaveBeenCalledTimes(2) // 【確認内容】: 連続クリックでクリック回数分だけ確認コールバックが発火することを確認
  })

  // ===== 境界値 =====

  it('TC-CD-B-01: open=false でダイアログ内容が描画されない', () => {
    // 【テスト目的】: open=false で内容が描画されないことを確認する
    // 【テスト内容】: open=false を渡し、title 等が DOM に存在しないことを検証する
    // 【期待される動作】: queryByText が null（非描画）
    // 🟡 信頼性レベル: requirements.md「2.5」「4.2 ConfirmDialog: open=false」より妥当な推測

    // 【テストデータ準備】: ダイアログ閉状態（初期状態・閉じた後）を想定
    // 【初期条件設定】: open=false を指定
    render(
      <ConfirmDialog
        open={false}
        title="削除しますか？"
        onConfirm={vi.fn()}
        onCancel={vi.fn()}
      />
    )

    // 【結果検証】: title が非表示であることを確認する
    // 【期待値確認】: open=false でコンテンツが描画されない（shadcn Dialog の挙動準拠）
    expect(screen.queryByText('削除しますか？')).toBeNull() // 【確認内容】: 閉状態でダイアログ内容が漏れ表示されないことを確認
  })

  it('TC-CD-B-02: confirmLabel/cancelLabel 省略時にデフォルトラベルが表示される', () => {
    // 【テスト目的】: ラベル省略時にデフォルト文言で確認・キャンセル両ボタンが描画されることを確認する
    // 【テスト内容】: confirmLabel/cancelLabel を渡さずレンダリングし、ボタンが2つ存在するか検証する
    // 【期待される動作】: 確認・キャンセル相当のボタンが2つ描画される
    // 🟡 信頼性レベル: requirements.md「2.5 入出力の関係性（既定文言は実装時決定）」より妥当な推測。具体的文言はアサーションを緩くする

    // 【テストデータ準備】: 標準的な確認ダイアログ（ラベル指定省略）を想定
    // 【初期条件設定】: label 系 props を渡さない
    render(
      <ConfirmDialog open={true} title="削除しますか？" onConfirm={vi.fn()} onCancel={vi.fn()} />
    )

    // 【結果検証】: ボタンが2つ存在することを確認する
    // 【期待値確認】: ラベル未指定でもボタンが欠落しないこと
    const buttons = screen.getAllByRole('button')
    expect(buttons).toHaveLength(2) // 【確認内容】: ラベル指定有無でボタン数が一致（2ボタン）することを確認
  })
})
