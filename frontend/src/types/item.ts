import type { Category, Tag } from './common'

export type MediaType =
  | 'anime'
  | 'movie'
  | 'drama'
  | 'manga'
  | 'novel'
  | 'game'
  | 'academic_book'
  | 'paper'

export type ItemStatus = '未着手' | '視聴中' | '視聴済'

export type RegistrationMethod =
  | { type: 'api'; source: string; external_id: string }
  | { type: 'manual' }

export type MediaTypeDetails =
  | {
      media_type: 'anime'
      episode_count?: number
      season_count?: number
      studio?: string
      source_type?: string
      genres?: string[]
    }
  | {
      media_type: 'movie'
      runtime_minutes?: number
      director?: string
    }
  | {
      media_type: 'drama'
      episode_count?: number
      season_count?: number
      broadcaster?: string
    }
  | {
      media_type: 'manga'
      volume_count?: number
      episode_count?: number
      author?: string
      illustrator?: string
      magazine?: string
    }
  | {
      media_type: 'novel'
      volume_count?: number
      author?: string
      publisher?: string
      isbn?: string
    }
  | {
      media_type: 'game'
      platforms?: string[]
      developer?: string
      publisher?: string
      steam_app_id?: string
    }
  | {
      media_type: 'academic_book'
      author?: string
      publisher?: string
      isbn?: string
      ndl_id?: string
      google_books_id?: string
    }
  | {
      media_type: 'paper'
      doi?: string
      journal_name?: string
      volume_issue?: string
      page_range?: string
      authors?: string[]
    }

export interface ItemBase {
  id: string
  title: string
  original_title?: string
  release_date?: string
  synopsis?: string
  cover_image_url?: string
  related_url?: string
  is_favorite: boolean
  tags: Tag[]
  categories: Category[]
  registration_method: RegistrationMethod
}

/**
 * paper には rating/status が存在しない（詳細ページ/フォームの表示セクションが
 * 丸ごと変わる構造上の差異のため、フィールド設定テーブルではなくこの判別共用体で扱う）
 */
export type Item =
  | (ItemBase &
      Exclude<MediaTypeDetails, { media_type: 'paper' }> & {
        rating?: number
        status?: ItemStatus
      })
  | (ItemBase & Extract<MediaTypeDetails, { media_type: 'paper' }>)
