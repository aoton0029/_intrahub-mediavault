/**
 * TASK-0034: アクセシビリティ・レスポンシブ対応
 * FilterBar アクセシビリティテスト（追加）
 *
 * TC-04: フィルタUIの各コントロールにラベル関連付け／aria属性がある
 * 🔵 信頼性レベル: TASK-0034 TC-5 と FilterBar.tsx 実装より確実
 */
import { describe, expect, it, vi } from 'vitest'
import { render, screen, within } from '@testing-library/react'
import { FilterBar } from './FilterBar'
import type { Tag, Category } from '@/types'

const mockTags: Tag[] = [
  { id: 'tag-1', name: 'SF' },
  { id: 'tag-2', name: 'アクション' },
]

const mockCategories: Category[] = [
  { id: 'cat-1', name: '趣味' },
  { id: 'cat-2', name: '学習' },
]

describe('FilterBar - アクセシビリティ (TASK-0034)', () => {
  // ===== TC-04: フィルタ各コントロールのラベル関連付け =====

  describe('TC-04: フィルタUI各コントロールにラベルが関連付いている', () => {
    it('TC-04-1: メディアタイプ select が getByLabelText で取得できる', () => {
      // 【テスト目的】: メディアタイプセレクトにアクセシブルネームが付いていることを確認する
      // 【テスト内容】: FilterBar をレンダリングし getByLabelText('メディアタイプ') で select を取得する
      // 【期待される動作】: スクリーンリーダーが「メディアタイプ」ラベルを読み上げ select を識別できる
      // 🔵 信頼性レベル: TASK-0034 TC-5 と FilterBar.tsx 実装（FilterSelectField の id/htmlFor）より確実

      render(
        <FilterBar
          filters={{}}
          onChange={vi.fn()}
          tagOptions={mockTags}
          categoryOptions={mockCategories}
        />
      )

      // 【結果検証】: メディアタイプ select がラベルで取得できることを確認する
      const mediaTypeSelect = screen.getByLabelText('メディアタイプ') // 【確認内容】: メディアタイプラベルが select に関連付いていることを確認
      expect(mediaTypeSelect).toBeInTheDocument()
      expect(mediaTypeSelect.tagName.toLowerCase()).toBe('select') // 【確認内容】: 取得した要素が select であることを確認
    })

    it('TC-04-2: タグ select が getByLabelText で取得できる', () => {
      // 【テスト目的】: タグセレクトにアクセシブルネームが付いていることを確認する
      // 🔵 信頼性レベル: TASK-0034 TC-5 より確実

      render(
        <FilterBar
          filters={{}}
          onChange={vi.fn()}
          tagOptions={mockTags}
          categoryOptions={mockCategories}
        />
      )

      const tagSelect = screen.getByLabelText('タグ') // 【確認内容】: タグラベルが select に関連付いていることを確認
      expect(tagSelect).toBeInTheDocument()
      expect(tagSelect.tagName.toLowerCase()).toBe('select') // 【確認内容】: 取得した要素が select であることを確認
    })

    it('TC-04-3: カテゴリ select が getByLabelText で取得できる', () => {
      // 【テスト目的】: カテゴリセレクトにアクセシブルネームが付いていることを確認する
      // 🔵 信頼性レベル: TASK-0034 TC-5 より確実

      render(
        <FilterBar
          filters={{}}
          onChange={vi.fn()}
          tagOptions={mockTags}
          categoryOptions={mockCategories}
        />
      )

      const categorySelect = screen.getByLabelText('カテゴリ') // 【確認内容】: カテゴリラベルが select に関連付いていることを確認
      expect(categorySelect).toBeInTheDocument()
      expect(categorySelect.tagName.toLowerCase()).toBe('select') // 【確認内容】: 取得した要素が select であることを確認
    })

    it('TC-04-4: お気に入り checkbox が getByLabelText で取得できる', () => {
      // 【テスト目的】: お気に入りチェックボックスにアクセシブルネームが付いていることを確認する
      // 🔵 信頼性レベル: TASK-0034 TC-5 と FilterBar.tsx 実装（label 内包 checkbox）より確実

      render(
        <FilterBar
          filters={{}}
          onChange={vi.fn()}
          tagOptions={mockTags}
          categoryOptions={mockCategories}
        />
      )

      const favoriteCheckbox = screen.getByLabelText('お気に入り') // 【確認内容】: お気に入りラベルが checkbox に関連付いていることを確認
      expect(favoriteCheckbox).toBeInTheDocument()
      expect(favoriteCheckbox).toHaveAttribute('type', 'checkbox') // 【確認内容】: 取得した要素が checkbox type の input であることを確認
    })

    it('TC-04-5: ステータスchipグループが role=group とアクセシブルネームで取得できる', () => {
      // 【テスト目的】: ステータスのchipボタン群がグループとしてアクセシブルネームを持つことを確認する
      // 【変更理由】: TASK-0006でステータス絞り込みがselectから.chip相当のトグルボタン群に変更されたため
      // 🔵 信頼性レベル: TASK-0006 REQ-005・FilterBar.tsx実装（role="group" aria-label="ステータス"）より確実

      render(
        <FilterBar
          filters={{}}
          onChange={vi.fn()}
          tagOptions={mockTags}
          categoryOptions={mockCategories}
        />
      )

      const statusGroup = screen.getByRole('group', { name: 'ステータス' }) // 【確認内容】: ステータスグループがrole/aria-labelで取得できることを確認
      expect(statusGroup).toBeInTheDocument()
      expect(within(statusGroup).getAllByRole('button')).toHaveLength(3) // 【確認内容】: 未開始/進行中/完了の3chipが存在することを確認
    })

    it('TC-04-6: 5 種全コントロール（select×3, chipグループ×1, checkbox×1）がすべてラベル/ロールで取得できる', () => {
      // 【テスト目的】: フィルタUIの全コントロールにアクセシブルネームが付いていることを一括確認する
      // 🔵 信頼性レベル: TASK-0006・TASK-0034 TC-5 より確実

      render(
        <FilterBar
          filters={{}}
          onChange={vi.fn()}
          tagOptions={mockTags}
          categoryOptions={mockCategories}
        />
      )

      // 【結果検証】: 5 種全コントロールがラベル/ロールで取得できることを一括確認する
      expect(screen.getByLabelText('メディアタイプ')).toBeInTheDocument() // 【確認内容】: メディアタイプコントロール
      expect(screen.getByLabelText('タグ')).toBeInTheDocument() // 【確認内容】: タグコントロール
      expect(screen.getByLabelText('カテゴリ')).toBeInTheDocument() // 【確認内容】: カテゴリコントロール
      expect(screen.getByLabelText('お気に入り')).toBeInTheDocument() // 【確認内容】: お気に入りコントロール
      expect(screen.getByRole('group', { name: 'ステータス' })).toBeInTheDocument() // 【確認内容】: ステータスchipグループコントロール
    })
  })
})
