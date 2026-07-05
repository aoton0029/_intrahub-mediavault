/**
 * ホーム画面（一覧）向けのItem型。詳細ページ用の `src/types/item.ts` の Item とは
 * 別に、一覧カードが必要とするフィールドのみを持つ軽量な型として定義する。
 */
export type MediaType =
  | 'anime'
  | 'movie'
  | 'drama'
  | 'manga'
  | 'novel'
  | 'game'
  | 'academic_book'
  | 'paper';

export type ItemStatus = 'not_started' | 'in_progress' | 'completed';

export interface TagRef {
  id: string;
  name: string;
}

export interface CategoryRef {
  id: string;
  name: string;
}

export interface Item {
  id: string;
  media_type: MediaType;
  title: string;
  status?: ItemStatus;
  is_favorite: boolean;
  rating?: number;
  cover_image_url?: string;
  tags?: TagRef[];
  categories?: CategoryRef[];
  created_at: string;
  updated_at: string;
}

export const mediaTypeLabels: Record<MediaType, string> = {
  anime: 'アニメ',
  movie: '映画',
  drama: 'ドラマ',
  manga: '漫画',
  novel: '小説',
  game: 'ゲーム',
  academic_book: '学術書・専門書',
  paper: '論文・文献',
};

export const statusLabels: Record<ItemStatus, string> = {
  not_started: '未着手',
  in_progress: '視聴中',
  completed: '視聴済',
};
