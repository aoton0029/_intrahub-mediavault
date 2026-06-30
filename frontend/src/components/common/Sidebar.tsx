import { NavLink } from 'react-router-dom'
import { cn } from '@/lib/utils'

// 【型定義】: ナビゲーション項目の型。endは"/"のみtrueを持つため省略可能とする 🔵
interface NavItem {
  to: string
  label: string
  end?: boolean
}

// 【データ定義】: ナビゲーション項目とパスの対応表（requirements.md「ナビゲーション項目とパスの対応」より確定） 🔵
// 【設計方針】: ルートパス("/")のみendをtrueにし、NavLinkの前方一致マッチングによる
//              誤アクティブ化（例: "/"が常にアクティブになる）を防止する
const navItems: readonly NavItem[] = [
  { to: '/', label: '全体一覧', end: true },
  { to: '/collections/general', label: '一般メディア' },
  { to: '/collections/academic', label: '学術書・専門書' },
  { to: '/collections/paper', label: '論文・文献' },
  { to: '/mylists', label: 'マイリスト' },
  { to: '/tags-categories', label: 'タグ/カテゴリ' },
  { to: '/staff', label: 'スタッフ' },
  { to: '/settings', label: '設定' },
]

/**
 * 【機能概要】: 全画面共通のグローバルナビゲーション（サイドバー）を表示する
 * 【改善内容】: navItemsの型を明示化し、アクティブ時のaria-current付与でアクセシビリティを向上
 * 【設計方針】: react-router-dom v7のNavLinkを用い、isActiveに応じてactiveクラスとaria-currentを付与する
 * 【保守性】: ナビゲーション項目の追加はnavItems配列への要素追加のみで完結する
 * 【テスト対応】: TC-01〜TC-08（Sidebar.test.tsx）を通すための実装
 * 🔵 信頼性レベル: TASK-0007.md「ナビゲーション構成」「React Router v7との統合」より確定
 */
export function Sidebar() {
  return (
    <nav aria-label="グローバルナビゲーション">
      {navItems.map((item) => (
        <NavLink
          key={item.to}
          to={item.to}
          end={item.end}
          // 【アクティブ判定】: isActiveに応じてactiveクラスとaria-currentを付与し、視覚的・支援技術の双方で現在地を識別できるようにする 🔵
          className={({ isActive }) => cn('nav-link', isActive && 'active')}
        >
          {({ isActive }) => (
            // 【アクセシビリティ】: スクリーンリーダーが現在ページを認識できるようaria-currentを付与 🟡
            <span aria-current={isActive ? 'page' : undefined}>{item.label}</span>
          )}
        </NavLink>
      ))}
    </nav>
  )
}
