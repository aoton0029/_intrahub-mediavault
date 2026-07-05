import { NavSection } from './NavSection'
import { NavItem } from './NavItem'

export function Sidebar() {
  return (
    <aside className="collapse:hidden flex flex-col overflow-y-auto border-r border-border-soft bg-bg-sidebar p-3.5 px-2">
      <div className="flex items-center gap-2 px-2 pt-1 pb-4 font-display text-base font-semibold text-text-primary">
        <span className="h-2 w-2 shrink-0 rounded-full bg-accent" />
        MediaVault
      </div>

      <NavSection>
        <NavItem to="/" label="ホーム" />
      </NavSection>

      <NavSection label="ライブラリ">
        <NavItem to="/general-media" label="一般メディア" />
        <NavItem to="/academic-books" label="学術書" />
        <NavItem to="/papers" label="論文" />
        <NavItem to="/mylists" label="マイリスト" />
      </NavSection>

      <NavSection label="管理">
        <NavItem to="/tags" label="タグ" />
        <NavItem to="/categories" label="カテゴリ" />
        <NavItem to="/staff" label="スタッフ" />
      </NavSection>
    </aside>
  )
}
