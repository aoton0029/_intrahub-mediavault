import { describe, expect, it } from 'vitest'
import { render, screen } from '@testing-library/react'
import { ItemMetaSections } from './index'
import type { Category, ItemRelationView, ItemStaffView, Mylist, Tag } from '@/features/items/types'

function makeTag(overrides: Partial<Tag> = {}): Tag {
  return { id: 'tag-1', name: 'SF', ...overrides }
}

function makeCategory(overrides: Partial<Category> = {}): Category {
  return { id: 'cat-1', name: '2026年冬クール', ...overrides }
}

function makeRelation(overrides: Partial<ItemRelationView> = {}): ItemRelationView {
  return {
    id: 'rel-1',
    related_item_id: 'item-2',
    related_item_title: '紙の上の庭園',
    relation_type: 'reference',
    ...overrides,
  }
}

function makeStaff(overrides: Partial<ItemStaffView> = {}): ItemStaffView {
  return {
    item_staff_id: 'staff-1',
    staff_id: 's-1',
    name: '川瀬 直人',
    role: '監督',
    character_name: null,
    ...overrides,
  }
}

function makeMylist(overrides: Partial<Mylist> = {}): Mylist {
  return { id: 'mylist-1', name: '2026年に観た作品', ...overrides }
}

describe('ItemMetaSections', () => {
  it('renders tags as tag-pill list', () => {
    render(
      <ItemMetaSections
        tags={[makeTag(), makeTag({ id: 'tag-2', name: '泣ける' })]}
        categories={[]}
        relations={[]}
        staff={[]}
        mylists={[]}
      />,
    )

    expect(screen.getByRole('heading', { name: 'タグ' })).toBeInTheDocument()
    expect(screen.getByText('SF')).toBeInTheDocument()
    expect(screen.getByText('泣ける')).toBeInTheDocument()
  })

  it('renders categories as tag-pill list', () => {
    render(
      <ItemMetaSections
        tags={[]}
        categories={[makeCategory()]}
        relations={[]}
        staff={[]}
        mylists={[]}
      />,
    )

    expect(screen.getByRole('heading', { name: 'カテゴリ' })).toBeInTheDocument()
    expect(screen.getByText('2026年冬クール')).toBeInTheDocument()
  })

  it('renders relations of both reference and dlc types', () => {
    render(
      <ItemMetaSections
        tags={[]}
        categories={[]}
        relations={[
          makeRelation({ relation_type: 'reference', related_item_title: '紙の上の庭園' }),
          makeRelation({ id: 'rel-2', relation_type: 'dlc', related_item_title: '追加シナリオ' }),
        ]}
        staff={[]}
        mylists={[]}
      />,
    )

    expect(screen.getByRole('heading', { name: '関連付け' })).toBeInTheDocument()
    expect(screen.getByText('紙の上の庭園')).toBeInTheDocument()
    expect(screen.getByText('reference')).toBeInTheDocument()
    expect(screen.getByText('追加シナリオ')).toBeInTheDocument()
    expect(screen.getByText('dlc')).toBeInTheDocument()
  })

  it('renders staff with role and character_name combined', () => {
    render(
      <ItemMetaSections
        tags={[]}
        categories={[]}
        relations={[]}
        staff={[
          makeStaff({ name: '川瀬 直人', role: '監督', character_name: null }),
          makeStaff({
            item_staff_id: 'staff-2',
            name: '綾瀬 ひかる',
            role: '声優',
            character_name: '主人公役',
          }),
        ]}
        mylists={[]}
      />,
    )

    expect(screen.getByRole('heading', { name: 'スタッフ' })).toBeInTheDocument()
    expect(screen.getByText('川瀬 直人')).toBeInTheDocument()
    expect(screen.getByText('監督')).toBeInTheDocument()
    expect(screen.getByText('綾瀬 ひかる')).toBeInTheDocument()
    expect(screen.getByText('声優・主人公役')).toBeInTheDocument()
  })

  it('renders mylists as tag-pill list', () => {
    render(
      <ItemMetaSections
        tags={[]}
        categories={[]}
        relations={[]}
        staff={[]}
        mylists={[makeMylist()]}
      />,
    )

    expect(screen.getByRole('heading', { name: 'マイリスト' })).toBeInTheDocument()
    expect(screen.getByText('2026年に観た作品')).toBeInTheDocument()
  })

  it('hides each section when its array is empty', () => {
    render(
      <ItemMetaSections
        tags={[]}
        categories={[makeCategory()]}
        relations={[]}
        staff={[]}
        mylists={[]}
      />,
    )

    expect(screen.queryByRole('heading', { name: 'タグ' })).not.toBeInTheDocument()
    expect(screen.getByRole('heading', { name: 'カテゴリ' })).toBeInTheDocument()
    expect(screen.queryByRole('heading', { name: '関連付け' })).not.toBeInTheDocument()
    expect(screen.queryByRole('heading', { name: 'スタッフ' })).not.toBeInTheDocument()
    expect(screen.queryByRole('heading', { name: 'マイリスト' })).not.toBeInTheDocument()
  })

  it('renders nothing when all arrays are empty', () => {
    const { container } = render(
      <ItemMetaSections tags={[]} categories={[]} relations={[]} staff={[]} mylists={[]} />,
    )
    expect(container).toBeEmptyDOMElement()
  })
})
