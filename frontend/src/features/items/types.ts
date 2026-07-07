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

/**
 * 一般メディア検索（GET /items/search）で使う media_type の部分集合。
 * 学術書・論文はこのAPIの対象外（12_general_media_search.htmlの種別セレクタと一致）。
 */
export type GeneralMediaType = 'anime' | 'movie' | 'drama' | 'manga' | 'novel' | 'game';

/**
 * api_provider enum（バックエンド ApiProvider の serde 表現）。
 * jikan はAPIキー不要のため enum に含まれず、ワイヤ上は provider: null で表現される。
 */
export type ExternalProvider = 'tmdb' | 'igdb' | 'ndl' | 'steam' | 'open_library' | 'ani_list';

export const externalProviderLabels: Record<ExternalProvider, string> = {
  tmdb: 'TMDB',
  igdb: 'IGDB',
  ndl: 'NDL',
  steam: 'Steam',
  open_library: 'OpenLibrary',
  ani_list: 'AniList',
};

/** provider の表示名（null = Jikan：キー不要プロバイダ） */
export function providerLabel(provider: ExternalProvider | null): string {
  return provider ? externalProviderLabels[provider] : 'Jikan';
}

/** media_type ごとに使用される検索プロバイダ（null = Jikan、キー不要） */
export const mediaTypeProvider: Record<GeneralMediaType, ExternalProvider | null> = {
  anime: null,
  manga: null,
  movie: 'tmdb',
  drama: 'tmdb',
  novel: 'ndl',
  game: 'igdb',
};

/**
 * ノーマライズ済みドメインモデル（docs/backend/mediavault-api/items.md「MediaDetails」）。
 * GET /items/search のレスポンス要素・POST /items/import のリクエストボディの両方に使う。
 * ワイヤ上はフラットな1オブジェクトで、media_type が判別子。
 */
export interface MediaCore {
  media_type: MediaType;
  provider: ExternalProvider | null;
  external_id: string;
  title: string;
  original_title: string | null;
  alternative_titles: string[];
  description: string | null;
  release_date: string | null;
  image_url: string | null;
  genres: string[];
  rating: number | null;
  url: string | null;
}

export interface AnimeDetails extends MediaCore {
  media_type: 'anime';
  episodes: number | null;
  status: string | null;
  season: string | null;
  year: number | null;
  studios: string[];
  source: string | null;
  duration: string | null;
  trailer_url: string | null;
}

export interface MangaDetails extends MediaCore {
  media_type: 'manga';
  chapters: number | null;
  volumes: number | null;
  status: string | null;
  authors: string[];
  serializations: string[];
}

export interface MovieDetails extends MediaCore {
  media_type: 'movie';
  runtime_minutes: number | null;
  original_language: string | null;
  vote_count: number | null;
  collection: string | null;
  production_companies: string[];
}

export interface DramaDetails extends MediaCore {
  media_type: 'drama';
  number_of_seasons: number | null;
  number_of_episodes: number | null;
  networks: string[];
  status: string | null;
  original_language: string | null;
  first_air_date: string | null;
  last_air_date: string | null;
}

export interface GameDetails extends MediaCore {
  media_type: 'game';
  platforms: string[];
  developers: string[];
  publishers: string[];
  screenshots: string[];
  metacritic: number | null;
  steam_appid: number | null;
  storyline: string | null;
}

/** 小説・学術書・論文は書誌形状を共有する */
export interface NovelDetails extends MediaCore {
  media_type: 'novel' | 'academic_book' | 'paper';
  authors: string[];
  publisher: string | null;
  isbn: string | null;
  page_count: number | null;
  physical_format: string | null;
}

export type MediaDetails =
  | AnimeDetails
  | MangaDetails
  | MovieDetails
  | DramaDetails
  | GameDetails
  | NovelDetails;

/** POST /items/import のリクエストボディは検索結果要素（MediaDetails）そのもの */
export type ImportItemRequest = MediaDetails;
