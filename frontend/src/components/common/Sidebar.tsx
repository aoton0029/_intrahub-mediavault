import { NavLink } from 'react-router-dom'
import { cn } from '@/lib/utils'

// 【型定義】: ナビゲーション項目の型。endは"/"のみtrueを持つため省略可能とする 🔵
// 【拡張】: count（件数バッジ）・indent（サブカテゴリのインデント階層）を追加 🔵 TASK-0007
interface NavItem {
  to: string
  label: string
  end?: boolean
  count?: number
  indent?: boolean
}

// 【データ定義】: 「ライブラリ」セクション配下のナビ項目（全体一覧＋3メディアグループ） 🔵
// 【設計方針】: requirements.md REQ-003・dataflow.md「Sidebar拡張」フロー図に基づき、
//              全体一覧＋3メディアグループを「ライブラリ」セクションとして扱う（🟡 セクション区分は妥当な推測）
const libraryNavItems: readonly NavItem[] = [
  { to: '/', label: '全体一覧', end: true },
  { to: '/collections/general', label: '一般メディア', indent: true },
  { to: '/collections/academic', label: '学術書・専門書', indent: true },
  { to: '/collections/paper', label: '論文・文献', indent: true },
]

// 【データ定義】: 「ライブラリ」セクション外のナビ項目（見出しなし） 🔵
const otherNavItems: readonly NavItem[] = [
  { to: '/mylists', label: 'マイリスト' },
  { to: '/tags-categories', label: 'タグ/カテゴリ' },
  { to: '/staff', label: 'スタッフ' },
]

// 【データ定義】: Sidebar下部に固定配置する「設定」ナビ項目 🔵
const settingsNavItem: NavItem = { to: '/settings', label: '設定' }

/**
 * 【機能概要】: ナビ項目1件分のリンクを描画する
 * 【設計方針】: NavLinkのisActiveに応じてactiveクラスとaria-currentを付与し、
 *              indent指定時は.indent相当のクラスを、count指定時は件数バッジを表示する
 * 🔵 信頼性レベル: TASK-0007.md 拡張要素2・3より確定
 */
function NavItemLink({ item }: { item: NavItem }) {
  return (
    <NavLink
      to={item.to}
      end={item.end}
      className={({ isActive }) =>
        cn('nav-item', 'nav-link', isActive && 'active', item.indent && 'indent')
      }
    >
      {({ isActive }) => (
        <span aria-current={isActive ? 'page' : undefined} className="flex items-center gap-2 w-full">
          {item.label}
          {item.count !== undefined && <span className="count">{item.count}</span>}
        </span>
      )}
    </NavLink>
  )
}

/**
 * 【機能概要】: 全画面共通のグローバルナビゲーション（サイドバー）を表示する
 * 【改善内容】: ブランドロゴ・件数バッジ・インデント階層・セクション見出し・設定の下部固定配置を追加（TASK-0007, REQ-003）
 * 【設計方針】: react-router-dom v7のNavLinkを用い、isActiveに応じてactiveクラスとaria-currentを付与する
 * 【保守性】: ナビゲーション項目の追加はlibraryNavItems/otherNavItems配列への要素追加のみで完結する
 * 🔵 信頼性レベル: TASK-0007.md「ナビゲーション構成」「React Router v7との統合」「Sidebar拡張」より確定
 */
export function Sidebar() {
  return (
    <nav className="sidebar" aria-label="グローバルナビゲーション">
      {/* 【拡張要素1】: ブランドロゴ（ドットアイコン+MediaVault） 🔵 */}
      <div className="brand">
        <span className="dot" />
        MediaVault
      </div>

      {/* 【拡張要素3・4】: 「ライブラリ」セクション見出し＋インデント階層項目 🔵 */}
      <div className="nav-section">
        <div className="nav-section-label">ライブラリ</div>
        {libraryNavItems.map((item) => (
          <NavItemLink key={item.to} item={item} />
        ))}
      </div>

      {otherNavItems.map((item) => (
        <NavItemLink key={item.to} item={item} />
      ))}

      {/* 【拡張要素5】: 設定ナビ項目をSidebar下部に固定配置 🔵 */}
      <div style={{ marginTop: 'auto' }}>
        <NavItemLink item={settingsNavItem} />
      </div>
    </nav>
  )
}
