import type { Category, ItemRelationView, ItemStaffView, Mylist, Tag } from '@/features/items/types'

export interface ItemMetaSectionsProps {
  tags: Tag[]
  categories: Category[]
  relations: ItemRelationView[]
  staff: ItemStaffView[]
  mylists: Mylist[]
}

function staffSubText(member: ItemStaffView): string {
  return member.character_name ? `${member.role}・${member.character_name}` : member.role
}

/**
 * 詳細画面「タグ・カテゴリ・関連付け・スタッフ・マイリスト」セクション群（TASK-0018）
 *
 * 02_item_detail.htmlの `.prop-group`/`.prop-taglist`/`.tag-pill`/`.prop-list-item`
 * 構造に準拠する。各セクションは対応する配列が空の場合、非表示にする。
 */
export function ItemMetaSections({ tags, categories, relations, staff, mylists }: ItemMetaSectionsProps) {
  if (
    tags.length === 0 &&
    categories.length === 0 &&
    relations.length === 0 &&
    staff.length === 0 &&
    mylists.length === 0
  ) {
    return null
  }

  return (
    <>
      {tags.length > 0 && (
        <section className="prop-group" aria-labelledby="item-meta-tags-heading">
          <h2 id="item-meta-tags-heading">タグ</h2>
          <ul className="prop-taglist">
            {tags.map((tag) => (
              <li key={tag.id}>
                <span className="tag-pill">{tag.name}</span>
              </li>
            ))}
          </ul>
        </section>
      )}

      {categories.length > 0 && (
        <section className="prop-group" aria-labelledby="item-meta-categories-heading">
          <h2 id="item-meta-categories-heading">カテゴリ</h2>
          <ul className="prop-taglist">
            {categories.map((category) => (
              <li key={category.id}>
                <span className="tag-pill">{category.name}</span>
              </li>
            ))}
          </ul>
        </section>
      )}

      {relations.length > 0 && (
        <section className="prop-group" aria-labelledby="item-meta-relations-heading">
          <h2 id="item-meta-relations-heading">関連付け</h2>
          <ul>
            {relations.map((relation) => (
              <li className="prop-list-item" key={relation.id}>
                <span className="label">{relation.related_item_title}</span>
                <span className="sub">{relation.relation_type}</span>
              </li>
            ))}
          </ul>
        </section>
      )}

      {staff.length > 0 && (
        <section className="prop-group" aria-labelledby="item-meta-staff-heading">
          <h2 id="item-meta-staff-heading">スタッフ</h2>
          <ul>
            {staff.map((member) => (
              <li className="prop-list-item" key={member.item_staff_id}>
                <span className="label">{member.name}</span>
                <span className="sub">{staffSubText(member)}</span>
              </li>
            ))}
          </ul>
        </section>
      )}

      {mylists.length > 0 && (
        <section className="prop-group" aria-labelledby="item-meta-mylists-heading">
          <h2 id="item-meta-mylists-heading">マイリスト</h2>
          <ul className="prop-taglist">
            {mylists.map((mylist) => (
              <li key={mylist.id}>
                <span className="tag-pill">{mylist.name}</span>
              </li>
            ))}
          </ul>
        </section>
      )}
    </>
  )
}
