/**
 * TASK-0034: アクセシビリティ・レスポンシブ対応
 * Sidebar アクセシビリティテスト（追加）
 *
 * TC-03: サイドバー各リンクが link ロールで取得でき Tab フォーカス可能
 * TC-05: モバイル Sheet/ハンバーガーで開閉（新規実装）
 * TC-09: フォーカス順序が視覚的順序と一致する
 * 🔵🟡 信頼性レベル: TASK-0034 TC-3, TC-5 より
 */
import { describe, expect, it } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { Sidebar } from './Sidebar'

function renderSidebar(path = '/') {
  return render(
    <MemoryRouter initialEntries={[path]}>
      <Sidebar />
    </MemoryRouter>
  )
}

describe('Sidebar - アクセシビリティ (TASK-0034)', () => {
  // ===== TC-03: サイドバー各リンクが link ロールで取得できる =====

  describe('TC-03: サイドバー各リンクが getAllByRole("link") で取得でき Tab フォーカス可能', () => {
    it('TC-03-1: 全ナビ項目が getAllByRole("link") で 8 件取得できる', () => {
      // 【テスト目的】: サイドバーが <div onClick> ではなくネイティブ <a>（NavLink）で実装されていることを確認する
      // 【テスト内容】: MemoryRouter で Sidebar をレンダリングし getAllByRole('link') で件数を検証する
      // 【期待される動作】: キーボード（Tab）で各リンクにフォーカスでき Enter で遷移できる
      // 🔵 信頼性レベル: TASK-0034 TC-3 と Sidebar.tsx 実装より確実

      renderSidebar()

      // 【結果検証】: 8 件の link が存在し、ネイティブ <a> タグであることを確認する
      const links = screen.getAllByRole('link') // 【確認内容】: link ロールで全ナビ項目が取得できることを確認
      expect(links).toHaveLength(8) // 【確認内容】: ナビ項目が過不足なく 8 件であることを確認
    })

    it('TC-03-2: 各リンク要素がネイティブ <a> タグである', () => {
      // 【テスト目的】: NavLink の実装がネイティブアンカー要素であることを確認する
      // 🔵 信頼性レベル: TASK-0034 TC-3 より

      renderSidebar()

      const links = screen.getAllByRole('link')
      links.forEach((link) => {
        expect(link.tagName.toLowerCase()).toBe('a') // 【確認内容】: 各リンクが <a> タグで実装されていることを確認
      })
    })

    it('TC-03-3: nav 要素に aria-label="グローバルナビゲーション" が付与されている', () => {
      // 【テスト目的】: ナビゲーション領域が支援技術で識別できるよう aria-label が付与されていることを確認する
      // 🔵 信頼性レベル: TASK-0034 TC-3 と Sidebar.tsx 実装より確実

      renderSidebar()

      // 【結果検証】: nav 要素が適切なアクセシブルネームを持つことを確認する
      const nav = screen.getByRole('navigation', { name: 'グローバルナビゲーション' }) // 【確認内容】: aria-label 付き nav 要素が存在することを確認
      expect(nav).toBeInTheDocument()
    })
  })

  // ===== TC-05: モバイル Sheet/ハンバーガーで開閉（新規実装）=====

  describe('TC-05: モバイル用ハンバーガーメニューからサイドバーを開閉できる', () => {
    it('TC-05-1: aria-label="メニューを開く" のボタンが存在する（モバイルトリガー）', async () => {
      // 【テスト目的】: モバイル幅でハンバーガーメニューボタンが表示されることを確認する
      // 【テスト内容】: Sidebar をレンダリングし getByRole('button', { name: 'メニューを開く' }) を取得する
      // 【期待される動作】: ハンバーガーアイコンボタンが aria-label 付きで表示される
      // 🟡 信頼性レベル: TASK-0034 UI/UX 要件より妥当な推測
      //
      // ⚠️ RED フェーズ: モバイル Sheet/ハンバーガーは現状未実装。
      //    このテストは失敗する（新規実装を駆動するため）。
      //    Green フェーズで shadcn/ui Sheet を導入し実装する。

      renderSidebar()

      // 【結果検証】: ハンバーガーボタンが存在することを確認する
      const menuButton = screen.getByRole('button', { name: 'メニューを開く' }) // 【確認内容】: aria-label="メニューを開く" のボタンが存在することを確認
      expect(menuButton).toBeInTheDocument()
    })

    it('TC-05-2: ハンバーガーボタンをクリックすると Sheet が開きリンクが表示される', async () => {
      // 【テスト目的】: ハンバーガーボタン押下で Sheet（ドロワー）が開き、ナビリンクが到達可能になることを確認する
      // 【テスト内容】: ボタンをクリックし、Sheet 内に getAllByRole('link') が 8 件現れることを検証する
      // 【期待される動作】: クリック後に Sheet 内にリンクが 8 件表示される
      // 🟡 信頼性レベル: TASK-0034 UI/UX 要件・shadcn/ui Sheet 標準 API より妥当な推測
      //
      // ⚠️ RED フェーズ: 現状未実装。Green フェーズで実装後に通す。

      const user = userEvent.setup()
      renderSidebar()

      // 【初期条件設定】: ハンバーガーボタンを取得しクリックする
      const menuButton = screen.getByRole('button', { name: 'メニューを開く' })
      await user.click(menuButton)

      // 【結果検証】: Sheet が開いてナビリンクが 8 件表示されることを確認する
      const links = screen.getAllByRole('link') // 【確認内容】: Sheet 内にナビリンクが表示されることを確認
      expect(links).toHaveLength(8) // 【確認内容】: ナビ項目が過不足なく 8 件であることを確認
    })
  })

  // ===== TC-09: フォーカス順序が視覚的順序と一致する =====

  describe('TC-09: フォーカス順序が視覚的順序と一致する', () => {
    it('TC-09-1: Tab キーで最初にフォーカスされるのが「全体一覧」リンクである', async () => {
      // 【テスト目的】: サイドバーのフォーカス順序が視覚的な DOM 順と一致することを確認する
      // 【テスト内容】: Tab を 1 回押した後、document.activeElement が「全体一覧」リンクであることを検証する
      // 【期待される動作】: 先頭リンク（全体一覧）が最初にフォーカスされる
      // 🟡 信頼性レベル: TASK-0034 完了条件「フォーカス順序が視覚的順序と一致」より妥当な推測

      const user = userEvent.setup()
      renderSidebar()

      // 【実際の処理実行】: Tab キーを 1 回押してフォーカスを移動する
      await user.tab()

      // 【結果検証】: 最初にフォーカスされる要素が「全体一覧」リンクであることを確認する
      const activeEl = document.activeElement
      expect(activeEl).not.toBeNull()
      expect(activeEl?.tagName.toLowerCase()).toBe('a') // 【確認内容】: フォーカスが <a> 要素に当たっていることを確認
      expect(activeEl).toHaveTextContent('全体一覧') // 【確認内容】: 最初のフォーカス対象が「全体一覧」であることを確認
    })

    it('TC-09-2: Tab を 8 回押すと最後に「設定」リンクにフォーカスが当たる', async () => {
      // 【テスト目的】: サイドバーの全リンクを Tab で順番に走査できることを確認する
      // 🟡 信頼性レベル: TASK-0034 完了条件より妥当な推測

      const user = userEvent.setup()
      renderSidebar()

      // 【実際の処理実行】: Tab キーを 8 回押す
      for (let i = 0; i < 8; i++) {
        await user.tab()
      }

      // 【結果検証】: 8 番目のフォーカス対象が「設定」リンクであることを確認する
      const activeEl = document.activeElement
      expect(activeEl).toHaveTextContent('設定') // 【確認内容】: 8 回 Tab 後に「設定」リンクにフォーカスが当たることを確認
    })
  })
})
