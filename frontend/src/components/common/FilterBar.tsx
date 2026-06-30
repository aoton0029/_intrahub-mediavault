import type { ReactNode } from 'react'

interface FilterBarProps {
  children?: ReactNode
}

/**
 * 【機能概要】: 絞り込みUIの器（コンテナコンポーネント）。詳細実装は Phase 2 で行う
 * 【実装方針】: children をそのまま描画するだけの最小コンテナ
 * 【テスト対応】: TC-FB-N-01, TC-FB-E-01, TC-FB-B-01
 * 🔵 信頼性レベル: requirements.md「2.3 FilterBar（枠のみ）」より
 */
export function FilterBar({ children }: FilterBarProps) {
  return (
    <div data-testid="filter-bar" className="flex flex-wrap items-center gap-2">
      {children}
    </div>
  )
}
