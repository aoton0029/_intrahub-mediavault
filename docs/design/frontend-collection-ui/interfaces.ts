/**
 * frontend-collection-ui 型定義
 *
 * 作成日: 2026-06-22
 * 関連設計: architecture.md, design-interview.md（Q3: Discriminated Union方針）
 * 参照元: docs/design/mediavault-backend/types.rs, api-endpoints.md
 *
 * 信頼性レベル:
 * - 🔵 青信号: EARS要件定義書・バックエンドtypes.rs・既存実装を参考にした確実な型定義
 * - 🟡 黄信号: EARS要件定義書・設計文書から妥当な推測による型定義
 * - 🔴 赤信号: EARS要件定義書・設計文書にない推測による型定義
 */

// ========================================
// 列挙型（backend types.rs MediaType等と同期）
// ========================================

/** メディア種別 🔵 backend types.rs MediaTypeより */
export type MediaType =
  | 'anime'
  | 'movie'
  | 'drama'
  | 'manga'
  | 'novel'
  | 'game'
  | 'academic_book'
  | 'paper';

/** 一般メディア（REQ-004で定義されるグループ1） 🔵 requirements.md REQ-004より */
export type GeneralMediaType = 'anime' | 'movie' | 'drama' | 'manga' | 'novel' | 'game';

/** 視聴・読了ステータス 🟡 backend types.rs ItemStatusより */
export type ItemStatus = 'not_started' | 'in_progress' | 'completed';

/** アイテムの登録元 🔵 backend types.rs ItemSource・REQ-201/202より */
export type ItemSource = 'api' | 'manual';

/** item_groups のグループ種別 🔵 backend types.rs GroupTypeより */
export type GroupType = 'season' | 'volume' | 'chapter';

/** item_relations の関係種別 🔵 backend types.rs RelationTypeより */
export type RelationType = 'reference' | 'dlc';

/** item_files のファイル種別 🟡 backend types.rs FileTypeより */
export type FileType = 'pdf' | 'image' | 'other';

/** 外部APIキーのプロバイダ種別（Jikanはキー不要のため対象外） 🔵 backend types.rs ApiProviderより */
export type ApiProvider = 'tmdb' | 'igdb' | 'ndl' | 'steam' | 'open_library' | 'ani_list';

// ========================================
// メディア別詳細（判別共用体の構成要素） 🔵 ヒアリングQ3・backend types.rsより
// ========================================

export interface AnimeDetails {
  episodeCount?: number;
  seasonCount?: number;
  studio?: string;
  genreList: string[];
  sourceType?: string;
  jikanId?: string;
}

export interface MovieDetails {
  runtimeMinutes?: number;
  director?: string;
  genreList: string[];
  tmdbId?: string;
}

export interface DramaDetails {
  episodeCount?: number;
  seasonCount?: number;
  network?: string;
  genreList: string[];
  tmdbId?: string;
}

export interface MangaDetails {
  volumeCount?: number;
  chapterCount?: number;
  author?: string;
  illustrator?: string;
  magazine?: string;
  jikanId?: string;
}

export interface NovelDetails {
  volumeCount?: number;
  author?: string;
  publisher?: string;
  isbn?: string;
  openlibraryId?: string;
  googleBooksId?: string;
}

export interface GameDetails {
  platformList: string[];
  developer?: string;
  publisher?: string;
  steamAppid?: string;
  igdbId?: string;
}

export interface AcademicBookDetails {
  author?: string;
  publisher?: string;
  isbn?: string;
  ndlId?: string;
  googleBooksId?: string;
}

export interface PaperDetails {
  doi?: string;
  journalName?: string;
  volumeIssue?: string;
  pageRange?: string;
  authorList: string[];
  ndlId?: string;
}

// ========================================
// Item: media_typeによる判別共用体 🔵 ヒアリングQ3で確定
// ========================================

interface ItemBase {
  id: string;
  title: string;
  originalTitle?: string;
  description?: string;
  coverImageUrl?: string;
  releaseDate?: string; // ISO date (YYYY-MM-DD)
  homepageUrl?: string;
  status: ItemStatus;
  consumedDate?: string;
  rating?: number;
  isFavorite: boolean;
  source: ItemSource;
  /** source==='api' の場合のみ表示専用で保持（REQ-201）。source==='manual' では存在しない（REQ-202） */
  externalId?: string;
  createdAt: string;
  updatedAt: string;
}

export type Item =
  | (ItemBase & { mediaType: 'anime'; details: AnimeDetails })
  | (ItemBase & { mediaType: 'movie'; details: MovieDetails })
  | (ItemBase & { mediaType: 'drama'; details: DramaDetails })
  | (ItemBase & { mediaType: 'manga'; details: MangaDetails })
  | (ItemBase & { mediaType: 'novel'; details: NovelDetails })
  | (ItemBase & { mediaType: 'game'; details: GameDetails })
  | (ItemBase & { mediaType: 'academic_book'; details: AcademicBookDetails })
  | (ItemBase & { mediaType: 'paper'; details: PaperDetails });

/** mediaTypeでItemを絞り込むための型ガード 🟡 一般的なTypeScript判別共用体パターンから推測 */
export function isItemOfType<T extends MediaType>(
  item: Item,
  mediaType: T
): item is Item & { mediaType: T } {
  return item.mediaType === mediaType;
}

// ========================================
// item_groups / item_episodes 🔵 backend types.rs ItemGroup/ItemEpisodeより
// ========================================

export interface ItemGroup {
  id: string;
  itemId: string;
  parentItemId?: string;
  groupType: GroupType;
  groupName: string;
  number?: number;
  displayOrder: number;
  createdAt: string;
  updatedAt: string;
}

export interface ItemEpisode {
  id: string;
  groupId: string;
  episodeNumber: number;
  title?: string;
  originalTitle?: string;
  airDate?: string;
  description?: string;
  createdAt: string;
  updatedAt: string;
}

// ========================================
// 関連付け・リンク・ファイル・トレーラー 🔵 backend types.rsより
// ========================================

export interface ItemRelation {
  id: string;
  itemId: string;
  relatedItemId: string;
  relationType: RelationType;
  createdAt: string;
}

export interface ItemLink {
  id: string;
  itemId: string;
  url: string;
  label: string;
  createdAt: string;
}

export interface ItemFile {
  id: string;
  itemId: string;
  path: string;
  label?: string;
  fileType: FileType;
  /** PDFの場合のみ意味を持つ。紐付け済みならCalibre-Web閲覧リンクを表示（REQ-105） */
  calibreBookId?: string;
  createdAt: string;
  updatedAt: string;
}

export interface ItemTrailer {
  id: string;
  itemId: string;
  url: string;
  label?: string;
  createdAt: string;
}

// ========================================
// スタッフ 🔵 backend types.rsより
// ========================================

export interface Staff {
  id: string;
  externalId?: string;
  name: string;
  imageUrl?: string;
  createdAt: string;
}

export interface ItemStaff {
  id: string;
  itemId: string;
  staffId: string;
  role: string;
  characterName?: string;
}

// ========================================
// タグ・カテゴリ・マイリスト 🔵 backend types.rsより
// ========================================

export interface Tag {
  id: string;
  name: string;
}

export interface Category {
  id: string;
  name: string;
}

export interface MyList {
  id: string;
  name: string;
  createdAt: string;
}

// ========================================
// APIキー管理 🔵 backend types.rsより
// ========================================

export interface ApiCredential {
  provider: ApiProvider;
  /** 一覧表示時はマスク済み文字列（末尾数文字以外を***等で隠す） 🟡 NFR-101から妥当な推測 */
  apiKeyMasked: string;
  updatedAt: string;
}

// ========================================
// 一覧フィルタ（URLクエリパラメータ） 🔵 REQ-002/003より
// ========================================

export interface ItemListFilters {
  mediaType?: MediaType;
  tagId?: string;
  categoryId?: string;
  isFavorite?: boolean;
  status?: ItemStatus;
  page?: number;
  limit?: number;
}

// ========================================
// APIリクエスト/レスポンスDTO
// ========================================

/** アイテム新規作成リクエスト（手動 / API取込共通） 🔵 backend CreateItemRequestより */
export interface CreateItemRequest {
  mediaType: MediaType;
  title: string;
  originalTitle?: string;
  description?: string;
  coverImageUrl?: string;
  releaseDate?: string;
  homepageUrl?: string;
  source: ItemSource;
  externalId?: string;
  details: Record<string, unknown>;
}

/** アイテム部分更新リクエスト 🔵 backend UpdateItemRequestより */
export interface UpdateItemRequest {
  title?: string;
  description?: string;
  status?: ItemStatus;
  consumedDate?: string;
  rating?: number;
  isFavorite?: boolean;
}

/** status更新専用リクエスト（REQ-013） 🔵 backend PATCH /items/:id/statusより */
export interface UpdateItemStatusRequest {
  status: ItemStatus;
  consumedDate?: string;
}

/** 外部API検索クエリ 🔵 backend ExternalSearchQueryより */
export interface ExternalSearchQuery {
  mediaType: MediaType;
  q: string;
}

/** 外部API検索結果1件（プロバイダ生データのラップ） 🟡 backend api-endpoints.mdから妥当な推測 */
export interface ExternalSearchResultItem {
  externalId: string;
  title: string;
  coverImageUrl?: string;
  releaseDate?: string;
  raw: Record<string, unknown>;
}

/** 外部API検索結果からのインポートリクエスト 🔵 backend POST /items/importより */
export interface ImportItemRequest {
  mediaType: MediaType;
  externalId: string;
  raw: Record<string, unknown>;
}

/** グループ作成リクエスト 🔵 backend CreateGroupRequestより */
export interface CreateGroupRequest {
  groupType: GroupType;
  groupName: string;
  number?: number;
  displayOrder?: number;
}

/** 話数作成リクエスト 🔵 backend CreateEpisodeRequestより */
export interface CreateEpisodeRequest {
  episodeNumber: number;
  title?: string;
  originalTitle?: string;
  airDate?: string;
  description?: string;
}

/** ファイルアップロードリクエスト（multipart/form-data） 🔵 backend POST /items/:id/files/uploadより */
export interface UploadItemFileRequest {
  file: File;
  fileType: FileType;
  label?: string;
}

/** インポート結果サマリー 🔵 backend ImportSummaryより */
export interface ImportSummary {
  successCount: number;
  failureCount: number;
  failures: ImportFailure[];
}

/** インポート失敗行の詳細 🟡 backend ImportFailure（EDGE-002）より */
export interface ImportFailure {
  rowNumber: number;
  reason: string;
}

/** 外部APIキー登録リクエスト 🔵 backend UpsertApiCredentialRequestより */
export interface UpsertApiCredentialRequest {
  apiKey: string;
}

// ========================================
// 統一APIレスポンス型 🟡 backend ApiOk/ApiError（既存実装の共通パターン）より
// ========================================

export interface ApiOkResponse<T> {
  success: true;
  data: T;
  pagination?: Pagination;
}

export interface ApiErrorResponse {
  success: false;
  error: {
    code: string;
    message: string;
  };
}

export type ApiResponse<T> = ApiOkResponse<T> | ApiErrorResponse;

export interface Pagination {
  page: number;
  limit: number;
  total: number;
}

/** apiClientがApiErrorResponseを検知した際にthrowする例外型 🟡 architecture.md API層設計から妥当な推測 */
export class ApiClientError extends Error {
  constructor(public code: string, message: string) {
    super(message);
    this.name = 'ApiClientError';
  }
}

// ========================================
// 信頼性レベルサマリー
// ========================================
/**
 * - 🔵 青信号: 32件 (76%)
 * - 🟡 黄信号: 10件 (24%)
 * - 🔴 赤信号: 0件 (0%)
 *
 * 品質評価: 高品質（backend types.rs・api-endpoints.mdとの対応が明確。
 * 統一レスポンス型・検索結果の生データラップ形式はバックエンド側も🟡推測のため、本書でも同水準の信頼性とした）
 */
