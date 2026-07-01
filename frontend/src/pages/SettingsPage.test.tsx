/**
 * TASK-0034: アクセシビリティ・レスポンシブ対応
 * SettingsPage アクセシビリティテスト
 *
 * TC-02: APIキー登録フォームの各入力にラベルが関連付けられている（欠陥修正駆動）
 * TC-08: file 入力のラベル欠落検出
 * 🔵 信頼性レベル: TASK-0034 TC-2 と SettingsPage.tsx 実地調査より確実
 */
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import SettingsPage from './SettingsPage'

// 【モック設定】: TanStack Query フックのモックを設定する
vi.mock('@/api/settings', () => ({
  useApiCredentialsQuery: () => ({ data: [] }),
  useUpsertApiCredentialMutation: () => ({
    mutate: vi.fn(),
    isPending: false,
  }),
}))

vi.mock('@/api/import', () => ({
  useImportBooklogMutation: () => ({
    mutate: vi.fn(),
    isPending: false,
    data: null,
  }),
  useImportSteamMutation: () => ({
    mutate: vi.fn(),
    isPending: false,
    data: null,
  }),
}))

// Sonner トーストのモック
vi.mock('sonner', () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}))

describe('SettingsPage - アクセシビリティ (TASK-0034)', () => {
  // ===== TC-02: APIキー登録フォームのラベル関連付け =====

  describe('TC-02: APIキータブ - ProviderRow ラベル関連付け', () => {
    it('TC-02-1: TMDB のAPIキー入力がラベルで取得できる（getByLabelText）', async () => {
      // 【テスト目的】: TMDB プロバイダ行の <Label> と <Input> が htmlFor/id で関連付いていることを確認する
      // 【テスト内容】: SettingsPage を APIキータブ（デフォルト）でレンダリングし getByLabelText('TMDB') で入力を取得する
      // 【期待される動作】: スクリーンリーダーが「TMDB」ラベルを読み上げ入力欄を識別できる
      // 🔵 信頼性レベル: TASK-0034 TC-2 より。現状欠陥は SettingsPage.tsx 実地調査より確実
      //
      // ⚠️ RED フェーズ: 現状 ProviderRow の <Label> に htmlFor が無く <Input> に id も無い。
      //    このテストは現状では失敗する（欠陥修正を駆動するため）。
      //    Green フェーズで htmlFor/id または aria-label を付与して通す。

      // 【テストデータ準備】: 初期状態（maskedKey 無し）で全プロバイダが編集可能な入力を表示
      render(<SettingsPage />)

      // 【結果検証】: TMDB ラベルで入力欄が取得できることを確認する
      // 【期待値確認】: getByLabelText が <input> を1件返す
      const tmdbInput = screen.getByLabelText('TMDB') // 【確認内容】: TMDB ラベルが入力欄に関連付いていることを確認
      expect(tmdbInput).toBeInTheDocument()
      expect(tmdbInput.tagName.toLowerCase()).toBe('input') // 【確認内容】: ラベルが <input> 要素を指していることを確認
    })

    it('TC-02-2: IGDB のAPIキー入力がラベルで取得できる', async () => {
      // 【テスト目的】: IGDB プロバイダ行の入力にアクセシブルネームがあることを確認する
      // 🔵 信頼性レベル: TASK-0034 TC-2 より
      // ⚠️ RED フェーズ: 現状は失敗する（欠陥修正駆動）

      render(<SettingsPage />)

      const igdbInput = screen.getByLabelText('IGDB') // 【確認内容】: IGDB ラベルが入力欄に関連付いていることを確認
      expect(igdbInput).toBeInTheDocument()
    })

    it('TC-02-3: NDL のAPIキー入力がラベルで取得できる', async () => {
      // 【テスト目的】: NDL プロバイダ行の入力にアクセシブルネームがあることを確認する
      // 🔵 信頼性レベル: TASK-0034 TC-2 より
      // ⚠️ RED フェーズ: 現状は失敗する（欠陥修正駆動）

      render(<SettingsPage />)

      const ndlInput = screen.getByLabelText('NDL') // 【確認内容】: NDL ラベルが入力欄に関連付いていることを確認
      expect(ndlInput).toBeInTheDocument()
    })

    it('TC-02-4: Steam のAPIキー入力がラベルで取得できる', async () => {
      // 【テスト目的】: Steam プロバイダ行の入力にアクセシブルネームがあることを確認する
      // 🔵 信頼性レベル: TASK-0034 TC-2 より
      // ⚠️ RED フェーズ: 現状は失敗する（欠陥修正駆動）

      render(<SettingsPage />)

      const steamInput = screen.getByLabelText('Steam') // 【確認内容】: Steam ラベルが入力欄に関連付いていることを確認
      expect(steamInput).toBeInTheDocument()
    })

    it('TC-02-5: Open Library のAPIキー入力がラベルで取得できる', async () => {
      // 【テスト目的】: Open Library プロバイダ行の入力にアクセシブルネームがあることを確認する
      // 🔵 信頼性レベル: TASK-0034 TC-2 より
      // ⚠️ RED フェーズ: 現状は失敗する（欠陥修正駆動）

      render(<SettingsPage />)

      const openLibInput = screen.getByLabelText('Open Library') // 【確認内容】: Open Library ラベルが入力欄に関連付いていることを確認
      expect(openLibInput).toBeInTheDocument()
    })

    it('TC-02-6: AniList のAPIキー入力がラベルで取得できる', async () => {
      // 【テスト目的】: AniList プロバイダ行の入力にアクセシブルネームがあることを確認する
      // 🔵 信頼性レベル: TASK-0034 TC-2 より
      // ⚠️ RED フェーズ: 現状は失敗する（欠陥修正駆動）

      render(<SettingsPage />)

      const aniListInput = screen.getByLabelText('AniList') // 【確認内容】: AniList ラベルが入力欄に関連付いていることを確認
      expect(aniListInput).toBeInTheDocument()
    })
  })

  // ===== TC-08: インポートタブの file 入力ラベル関連付け確認 =====

  describe('TC-08: インポートタブ - file 入力のラベル関連付け確認', () => {
    it('TC-08-1: CSVファイル入力が getByLabelText で取得できる', async () => {
      // 【テスト目的】: インポートタブの file 入力に <Label htmlFor="booklog-file"> が関連付いていることを確認する
      // 【テスト内容】: インポートタブに切り替え、getByLabelText('CSVファイル') で file 入力を取得する
      // 【期待される動作】: ラベルが関連付いており支援技術で用途が識別できる
      // 🟡 信頼性レベル: TASK-0034 実装詳細1 より。現状は booklog-file の htmlFor/id 済み
      //
      // ✅ このテストは現状の実装で通る（ラベル既実装確認）。
      //    将来ラベルが欠落した場合に失敗して検出する番人テスト。

      const user = userEvent.setup()
      render(<SettingsPage />)

      // 【初期条件設定】: インポートタブに切り替える
      const importTab = screen.getByRole('tab', { name: 'インポート' })
      await user.click(importTab)

      // 【結果検証】: CSVファイル入力がラベルで取得できることを確認する
      const fileInput = screen.getByLabelText('CSVファイル') // 【確認内容】: file 入力に <Label htmlFor> が関連付いていることを確認
      expect(fileInput).toBeInTheDocument()
      expect(fileInput).toHaveAttribute('type', 'file') // 【確認内容】: 取得した要素が file type の input であることを確認
    })

    it('TC-08-2: Steam ID 入力が getByLabelText で取得できる', async () => {
      // 【テスト目的】: インポートタブの Steam ID 入力に <Label htmlFor="steam-id"> が関連付いていることを確認する
      // 🟡 信頼性レベル: TASK-0034 実装詳細1 より
      //
      // ✅ このテストは現状の実装で通る（ラベル既実装確認）。

      const user = userEvent.setup()
      render(<SettingsPage />)

      const importTab = screen.getByRole('tab', { name: 'インポート' })
      await user.click(importTab)

      const steamInput = screen.getByLabelText('Steam ID') // 【確認内容】: Steam ID 入力に <Label htmlFor> が関連付いていることを確認
      expect(steamInput).toBeInTheDocument()
    })
  })
})
