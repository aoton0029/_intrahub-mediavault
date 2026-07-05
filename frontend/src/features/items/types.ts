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
 * api_provider enum（docs/backend/mediavault-api/index.md）のうち、jikanを含めた
 * 一般メディア検索で実際に使われるプロバイダのみ。jikanはAPIキー不要のため
 * PUT /settings/api-keys/{provider} の対象一覧には含まれない。
 */
export type ExternalProvider = 'jikan' | 'tmdb' | 'open_library' | 'igdb';

export const externalProviderLabels: Record<ExternalProvider, string> = {
  jikan: 'Jikan',
  tmdb: 'TMDB',
  open_library: 'OpenLibrary',
  igdb: 'IGDB',
};

/** media_type ごとに使用される検索プロバイダ（12_general_media_search.htmlのコメントより） */
export const mediaTypeProvider: Record<GeneralMediaType, ExternalProvider> = {
  anime: 'jikan',
  manga: 'jikan',
  movie: 'tmdb',
  drama: 'tmdb',
  novel: 'open_library',
  game: 'igdb',
};

/**
 * GET /items/search のレスポンス要素。
 * 【ドキュメント未確定】docs/backend/mediavault-api/items.md にエンドポイント定義はあるが、
 * ExternalSearchResult のフィールド一覧は本文中に無い（mediavault-model/items.md にも未記載）。
 * 12_general_media_search.html のHTMLコメント（実装者向け注記）を一次情報として、
 * 正規化フィールドを持たない title・external_id・provider・raw_data のみの形で定義する。
 */
export interface ExternalSearchResult {
  title: string;
  external_id: string;
  provider: ExternalProvider;
  raw_data: Record<string, unknown>;
}

/**
 * POST /items/import のリクエストボディ。
 * 【ドキュメント未確定】ExternalSearchResult同様、バックエンド側の型がまだ
 * ドキュメント化されていないため、12_general_media_search.htmlのコメントに基づく
 * 最小契約として定義する。実装時にレスポンス実体で要検証。
 */
export interface ImportItemRequest {
  media_type: GeneralMediaType;
  external_id: string;
  title: string;
  provider: ExternalProvider;
  raw_data?: Record<string, unknown>;
}
