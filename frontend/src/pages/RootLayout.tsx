import { Outlet, useLocation } from 'react-router-dom'
import { Sidebar } from '@/components/common/Sidebar'
import { cn } from '@/lib/utils'

// 【機能概要】: 現在のルートがアイテム詳細画面（/items/:id）かどうかを判定する 🟡
// 【設計方針】: /items/new/* や /items/:id/edit はアイテム詳細画面ではないため除外する
function useIsItemDetailRoute(): boolean {
  const location = useLocation()
  return /^\/items\/(?!new(\/|$))[^/]+\/?$/.test(location.pathname)
}

// 【機能概要】: 全画面共通のapp-shellグリッドレイアウトを提供する 🔵
// 【改善内容】: flexベース2カラム構成から.app-shellグリッド構造へ変更（REQ-006, TASK-0009）
// 【設計方針】: アイテム詳細画面のみ.has-propertiesを付与し3カラム化、properties列は空間確保のみ
// 🔵 信頼性レベル: TASK-0009.md「app-shellグリッド化」より確定
export default function RootLayout() {
  const isItemDetail = useIsItemDetailRoute()
  return (
    <div className={cn('app-shell', isItemDetail && 'has-properties')}>
      <Sidebar />
      <main className="main">
        <Outlet />
      </main>
      {isItemDetail && <aside className="properties" />}
    </div>
  )
}
