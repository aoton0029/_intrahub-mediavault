import { NavSection } from './NavSection'
import { NavItem } from './NavItem'
import { useCategoriesQuery, useMediaTypeCountsQuery } from '@/features/items/api'
import { mediaTypeLabels, type MediaType } from '@/features/items/types'

const GENERAL_MEDIA_TYPES: MediaType[] = ['anime', 'movie', 'drama', 'manga', 'novel', 'game']

export function Sidebar() {
  // 【失敗許容】: タグ/カテゴリ・件数取得の失敗はサイドバー内で局所的に握りつぶし、
  // 一覧本体（GeneralMediaPage）の描画・E2E契約には影響させない
  const countsQuery = useMediaTypeCountsQuery()
  const categoriesQuery = useCategoriesQuery()

  // 「一般メディア」件数はUI側のみの概念で対応する集計エンドポイントが無いため、
  // 各 media_type の件数をクライアント側で合算する
  const generalMediaCount = countsQuery.data
    ? GENERAL_MEDIA_TYPES.reduce((sum, mediaType) => sum + (countsQuery.data?.[mediaType] ?? 0), 0)
    : undefined

  return (
    <aside className="max-collapse:hidden sticky top-0 flex h-screen flex-col overflow-y-auto border-r border-border-soft bg-bg-sidebar p-3.5 px-2">
      <div className="flex items-center gap-2 px-2 pt-1 pb-4 font-display text-base font-semibold text-text-primary">
        <span className="h-2 w-2 shrink-0 rounded-full bg-accent" />
        MediaVault
      </div>

      <NavSection>
        <NavItem to="/" icon="📚" label="全体一覧" count={countsQuery.data?.total} />
        <NavItem to="/general-media" icon="🎬" label="一般メディア" count={generalMediaCount} />
        {GENERAL_MEDIA_TYPES.map((mediaType) => (
          <NavItem
            key={mediaType}
            to="/general-media"
            label={mediaTypeLabels[mediaType]}
            count={countsQuery.data?.[mediaType]}
            indent
          />
        ))}
        <NavItem
          to="/academic-books"
          icon="🎓"
          label="学術書・専門書"
          count={countsQuery.data?.academic_book}
        />
        <NavItem to="/papers" icon="📄" label="論文・文献" count={countsQuery.data?.paper} />
      </NavSection>

      <NavSection label="ライブラリ">
        <NavItem to="/mylists" icon="⭐" label="マイリスト" />
        <NavItem to="/tags" icon="🏷️" label="タグ" />
        <NavItem to="/categories" icon="📁" label="カテゴリ" />
        {(categoriesQuery.data ?? []).map((category) => (
          <NavItem
            key={category.id}
            to="/categories"
            label={category.name}
            count={category.item_count}
            indent
          />
        ))}
        <NavItem to="/staff" icon="👤" label="スタッフ" />
      </NavSection>

      <NavSection className="mt-auto">
        <NavItem to="/settings" icon="⚙️" label="設定" />
      </NavSection>
    </aside>
  )
}
