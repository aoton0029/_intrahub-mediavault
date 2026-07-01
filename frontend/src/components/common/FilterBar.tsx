import type { ChangeEvent } from 'react'
import type { ItemListFilters, Tag, Category, MediaType, ItemStatus } from '@/types'

// 【定数: 全MediaType】: mediaTypeOptions未指定時のデフォルト選択肢 🔵 interfaces.ts MediaType型より
const ALL_MEDIA_TYPES: MediaType[] = [
  'anime', 'movie', 'drama', 'manga', 'novel', 'game', 'academic_book', 'paper',
]

// 【定数: MediaTypeラベル】: selectのoption表示テキストとして使用 🟡 MediaTypeBadgeのラベルから推測
const MEDIA_TYPE_LABELS: Record<MediaType, string> = {
  anime: 'アニメ',
  movie: '映画',
  drama: 'ドラマ',
  manga: '漫画',
  novel: '小説',
  game: 'ゲーム',
  academic_book: '学術書',
  paper: '論文',
}

// 【定数: ItemStatusラベル】: statusセレクトのoption表示テキストとして使用 🟡 ItemStatus型より
const STATUS_LABELS: Record<ItemStatus, string> = {
  not_started: '未開始',
  in_progress: '進行中',
  completed: '完了',
}

const ALL_STATUSES: ItemStatus[] = ['not_started', 'in_progress', 'completed']

// 【Selectクラス】: 全selectに共通するTailwindクラス（DRY原則）🔵
const SELECT_CLASS = 'rounded border border-border bg-background px-2 py-1 text-sm'

/**
 * 【機能概要】: label + select の繰り返しパターンを抽象化したヘルパーコンポーネント
 * 【改善内容】: label/select の組み合わせが4回繰り返されていたのを共通化（DRY原則）
 * 【アクセシビリティ】: htmlFor/id紐付けでキーボード操作対応（NFR-202）🟡
 */
interface FilterSelectFieldProps {
  id: string
  label: string
  value: string
  onChange: (e: ChangeEvent<HTMLSelectElement>) => void
  disabled: boolean
  children: React.ReactNode
}

function FilterSelectField({ id, label, value, onChange, disabled, children }: FilterSelectFieldProps) {
  return (
    <>
      <label htmlFor={id} className="text-sm font-medium">
        {label}
      </label>
      <select
        id={id}
        value={value}
        onChange={onChange}
        disabled={disabled}
        className={SELECT_CLASS}
      >
        {/* 【空オプション】: 空文字選択→undefinedとして扱いURLパラメータを除去 🟡 */}
        <option value="">すべて</option>
        {children}
      </select>
    </>
  )
}

/**
 * 【機能概要】: FilterBarコンポーネントのProps型定義
 * 【設計方針】: controlledコンポーネント。filters/onChangeを外部（Page）から注入する
 * 🔵 TASK-0010実装詳細・requirements.mdより確実
 */
interface FilterBarProps {
  /** 現在の絞り込み状態 🔵 */
  filters: ItemListFilters
  /** 絞り込み変更時のコールバック 🔵 */
  onChange: (filters: ItemListFilters) => void
  /** タグ選択肢（呼び出し元から受け取る・API呼び出し不可） 🔵 */
  tagOptions: Tag[]
  /** カテゴリ選択肢（呼び出し元から受け取る・API呼び出し不可） 🔵 */
  categoryOptions: Category[]
  /** 表示するmedia_typeを制限。省略時は全8種 🔵 REQ-004連携 */
  mediaTypeOptions?: MediaType[]
  /** ローディング中など全フィルタUIを一括で無効化する 🟡 */
  disabled?: boolean
}

/**
 * 【機能概要】: 一覧画面の絞り込みUIコンポーネント
 * 【設計方針】: controlledコンポーネント。内部stateを持たず、filters/onChangeで完全制御される。
 *              FilterBar自身はタグ/カテゴリ取得APIを呼ばない（関心の分離）。
 * 【テスト対応】: TC-FB-N-01〜07, TC-FB-E-01〜03, TC-FB-B-01〜04
 * 🔵 TASK-0010・REQ-002・REQ-003より確実
 */
export function FilterBar({
  filters,
  onChange,
  tagOptions,
  categoryOptions,
  mediaTypeOptions,
  disabled = false,
}: FilterBarProps) {
  // 【選択肢決定】: mediaTypeOptionsが指定されていれば制限、未指定なら全8種を使用 🔵
  const availableMediaTypes = mediaTypeOptions ?? ALL_MEDIA_TYPES

  // 【ハンドラ群】: 各フィルタ操作でfiltersをスプレッドして変更フィールドのみ上書きする 🔵
  // 空文字列はundefinedに変換してURLクエリパラメータを除去する

  const handleMediaTypeChange = (e: ChangeEvent<HTMLSelectElement>) => {
    const v = e.target.value
    onChange({ ...filters, mediaType: v ? (v as MediaType) : undefined })
  }

  const handleTagChange = (e: ChangeEvent<HTMLSelectElement>) => {
    onChange({ ...filters, tagId: e.target.value || undefined })
  }

  const handleCategoryChange = (e: ChangeEvent<HTMLSelectElement>) => {
    onChange({ ...filters, categoryId: e.target.value || undefined })
  }

  const handleStatusChange = (e: ChangeEvent<HTMLSelectElement>) => {
    const v = e.target.value
    onChange({ ...filters, status: v ? (v as ItemStatus) : undefined })
  }

  // 【お気に入りハンドラ】: checked=trueならisFavorite=true、falseならundefined（false不使用）🔵
  const handleFavoriteChange = (e: ChangeEvent<HTMLInputElement>) => {
    onChange({ ...filters, isFavorite: e.target.checked ? true : undefined })
  }

  // 【クリアハンドラ】: onChange({})で全フィルタをリセットしURLクエリパラメータを全て除去する 🟡
  const handleClear = () => onChange({})

  return (
    <div
      data-testid="filter-bar"
      className="flex flex-wrap items-center gap-2 sm:flex-row flex-col"
    >
      {/* メディアタイプ */}
      <FilterSelectField
        id="filter-media-type"
        label="メディアタイプ"
        value={filters.mediaType ?? ''}
        onChange={handleMediaTypeChange}
        disabled={disabled}
      >
        {availableMediaTypes.map(mt => (
          <option key={mt} value={mt}>{MEDIA_TYPE_LABELS[mt]}</option>
        ))}
      </FilterSelectField>

      {/* タグ */}
      <FilterSelectField
        id="filter-tag"
        label="タグ"
        value={filters.tagId ?? ''}
        onChange={handleTagChange}
        disabled={disabled}
      >
        {tagOptions.map(tag => (
          <option key={tag.id} value={tag.id}>{tag.name}</option>
        ))}
      </FilterSelectField>

      {/* カテゴリ */}
      <FilterSelectField
        id="filter-category"
        label="カテゴリ"
        value={filters.categoryId ?? ''}
        onChange={handleCategoryChange}
        disabled={disabled}
      >
        {categoryOptions.map(cat => (
          <option key={cat.id} value={cat.id}>{cat.name}</option>
        ))}
      </FilterSelectField>

      {/* お気に入り: true=チェック済み / undefined=未チェック（falseは使用しない）🟡 */}
      <label htmlFor="filter-favorite" className="flex items-center gap-1 text-sm font-medium">
        <input
          id="filter-favorite"
          type="checkbox"
          checked={filters.isFavorite ?? false}
          onChange={handleFavoriteChange}
          disabled={disabled}
          className="rounded"
        />
        お気に入り
      </label>

      {/* ステータス */}
      <FilterSelectField
        id="filter-status"
        label="ステータス"
        value={filters.status ?? ''}
        onChange={handleStatusChange}
        disabled={disabled}
      >
        {ALL_STATUSES.map(s => (
          <option key={s} value={s}>{STATUS_LABELS[s]}</option>
        ))}
      </FilterSelectField>

      {/* クリアボタン: 全フィルタをリセットしURLクエリパラメータを全て除去する 🟡 */}
      <button
        type="button"
        onClick={handleClear}
        disabled={disabled}
        className="rounded border border-border px-3 py-1 text-sm hover:bg-muted"
      >
        クリア
      </button>
    </div>
  )
}
