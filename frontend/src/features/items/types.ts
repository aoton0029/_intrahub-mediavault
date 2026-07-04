/**
 * コレクション一覧・詳細画面（コア機能） 型定義
 *
 * 作成日: 2026-07-04
 * 関連設計: docs/design/collection-browsing/architecture.md, dataflow.md, interfaces.ts
 * 元ファイル: docs/design/collection-browsing/interfaces.ts
 *
 * TASK-0002突合結果を反映済み（backend/mediavault-api/src/models/item.rs を正とする）:
 * - ItemStatus: 実値は "not_started" | "in_progress" | "completed" の3値（確定）
 * - ItemDetail: 実際にネストされるのは tags / categories / calibre_links / detail のみ（確定）。
 *   groups/links/files/trailers/relations/staff/mylists はネストされないため、
 *   ItemDetailからは除外し、必要な場合は個別APIエンドポイントを呼び出す設計とする。
 *   （詳細: docs/spec/collection-browsing/note.md「TASK-0002確認結果」参照）
 *
 * 信頼性レベル:
 * - 🔵 青信号: EARS要件定義書・設計文書・既存API仕様(docs/backend/mediavault-api)・バックエンド実装を参考にした確実な型定義
 * - 🟡 黄信号: EARS要件定義書・設計文書・既存API仕様から妥当な推測による型定義
 * - 🔴 赤信号: EARS要件定義書・設計文書・既存API仕様にない推測による型定義
 */

// ========================================
// Enum定義
// ========================================

/**
 * メディア種別
 * 🔵 信頼性: index.md 主要Enumより
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

/**
 * アイテムステータス
 * 🔵 信頼性: TASK-0002確認結果（backend/mediavault-api/src/models/item.rs L27-34）より確定。
 * `#[serde(rename_all = "snake_case")]` の Rust enum { NotStarted, InProgress, Completed } に対応。
 */
export type ItemStatus = 'not_started' | 'in_progress' | 'completed';

/**
 * グループ種別（シーズン/巻/章）
 * 🔵 信頼性: index.md 主要Enumより
 */
export type GroupType = 'season' | 'volume' | 'chapter';

/**
 * アイテム関連種別
 * 🔵 信頼性: index.md 主要Enumより
 */
export type RelationType = 'reference' | 'dlc';

// ========================================
// エンティティ定義（一覧・詳細共通）
// ========================================

/**
 * アイテム（一覧表示用）
 * 🔵 信頼性: items.md GET /items・01_home.htmlのカード表示項目より
 */
export interface Item {
  id: string; // 🔵 items.mdパスパラメータ仕様より
  media_type: MediaType; // 🔵 items.mdクエリパラメータより
  title: string; // 🔵 REQ-002・01_home.htmlより
  status: ItemStatus; // 🔵 REQ-002より
  is_favorite: boolean; // 🔵 items.mdクエリパラメータ is_favorite より
  cover_image_url?: string | null; // 🟡 doc-cover/media-card .cover のビジュアル要素から妥当な推測
  season_label?: string | null; // 🟡 01_home.htmlの「S2」表示から妥当な推測（現行シーズンのラベル）
  created_at: string; // 🟡 一般的なエンティティ共通項目から妥当な推測
  updated_at: string; // 🟡 一般的なエンティティ共通項目から妥当な推測
}

/**
 * タグ
 * 🔵 信頼性: tags.mdより
 */
export interface Tag {
  id: string;
  name: string;
}

/**
 * カテゴリ
 * 🔵 信頼性: categories.md（タグと構造同一）より
 */
export interface Category {
  id: string;
  name: string;
}

/**
 * マイリスト
 * 🔵 信頼性: mylists.mdより
 * 備考: TASK-0002確認によりItemDetailにはネストされない。GET /mylists 等の個別APIで取得する。
 */
export interface Mylist {
  id: string;
  name: string;
}

/**
 * アイテム関連（詳細画面「関連付け」セクション）
 * 🔵 信頼性: item-relations.md・02_item_detail.htmlより
 * 備考: TASK-0002確認によりItemDetailにはネストされない。個別APIで取得する。
 */
export interface ItemRelationView {
  id: string;
  related_item_id: string;
  related_item_title: string; // 🟡 表示用に結合された値。API仕様書に明記なし、妥当な推測
  relation_type: RelationType;
}

/**
 * アイテムに紐づくスタッフ
 * 🔵 信頼性: staff.md CreateItemStaffRequest・02_item_detail.htmlより
 * 備考: TASK-0002確認によりItemDetailにはネストされない。個別APIで取得する。
 */
export interface ItemStaffView {
  item_staff_id: string; // 🔵 DELETE /items/{id}/staff/{item_staff_id} のパスパラメータより
  staff_id: string;
  name: string;
  role: string;
  character_name?: string | null;
}

/**
 * エピソード（話数）
 * 🔵 信頼性: item-episodes.mdより
 */
export interface ItemEpisode {
  id: string;
  episode_number: number;
  title?: string | null;
  original_title?: string | null;
  air_date?: string | null;
  description?: string | null;
}

/**
 * グループ（シーズン/巻/章）
 * 🔵 信頼性: item-groups.mdより
 * 備考: TASK-0002確認によりItemDetailにはネストされない。GET /items/{id}/groups で個別取得する。
 */
export interface ItemGroup {
  id: string;
  group_type: GroupType;
  group_name: string;
  number?: number | null;
  display_order: number;
  parent_item_id?: string | null;
  episodes?: ItemEpisode[]; // 🟡 詳細取得時にネストして返る想定。API仕様書に明記なし、02_item_detail.htmlの表示構造から妥当な推測
}

/**
 * 外部リンク
 * 🔵 信頼性: item-links.mdより
 * 備考: TASK-0002確認によりItemDetailにはネストされない。個別APIで取得する。
 */
export interface ItemLink {
  id: string;
  url: string;
  label: string;
}

/**
 * 予告編リンク
 * 🔵 信頼性: item-trailers.mdより
 * 備考: TASK-0002確認によりItemDetailにはネストされない。個別APIで取得する。
 */
export interface ItemTrailer {
  id: string;
  url: string;
  label?: string | null;
}

/**
 * アイテムファイル
 * 🔵 信頼性: item-files.mdより（本要件では表示のみ、アップロード等の操作は対象外）
 * 備考: TASK-0002確認によりItemDetailにはネストされない。個別APIで取得する。
 */
export interface ItemFile {
  id: string;
  path: string;
  label?: string | null;
  file_type: 'pdf' | 'image' | 'other'; // 🔵 index.md file_type Enumより
  calibre_book_id?: string | null;
}

/**
 * Calibre-Web連携情報（ItemDetail.calibre_links の要素）
 * 🔵 信頼性: TASK-0002確認結果（backend/mediavault-api/src/models/item.rs CalibreWebLinkInfo）より確定
 * PDF item_filesでcalibre_book_idが設定済みの場合のみ含まれる。
 */
export interface CalibreWebLinkInfo {
  item_file_id: string;
  calibre_book_id: string;
  calibre_web_url?: string | null;
}

/**
 * アイテム詳細（GET /items/{id} レスポンス）
 * 🔵 信頼性: TASK-0002確認結果（backend/mediavault-api/src/models/item.rs L246-270 struct ItemDetail）より確定
 *
 * 重要: 設計時の想定（groups/links/files/trailers/relations/staff/mylistsをネスト）は誤りと判明。
 * 実際にネストされるのは tags / categories / calibre_links / detail のみ。
 * groups・links・files・trailers・relations・staff・mylistsは、詳細画面表示に必要な場合、
 * それぞれ個別のAPIエンドポイント（例: GET /items/{id}/groups, item-links.md 等）を
 * 別途呼び出して取得すること。上記の型（ItemGroup, ItemLink, ItemTrailer, ItemFile,
 * ItemRelationView, ItemStaffView, Mylist）はそれら個別エンドポイントのレスポンス型として
 * 引き続き使用する。
 */
export interface ItemDetail extends Item {
  original_title?: string | null; // 🔵 02_item_detail.htmlの「原題」表示より
  description?: string | null; // 🔵 02_item_detail.htmlの「概要」セクションより
  consumed_date?: string | null; // 🔵 items.md UpdateStatusRequest・02_item_detail.html Propertiesより
  rating?: number | null; // 🔵 02_item_detail.html Properties「rating」より
  source: string; // 🔵 index.md item_source Enum・02_item_detail.html「source」より（例: "api (Jikan)"相当の内部コード値）
  release_date?: string | null; // 🔵 02_item_detail.html Properties「release_date」より
  studio?: string | null; // 🟡 02_item_detail.htmlの「studio」項目はanime固有属性の可能性。media_type別詳細フィールドの一部として妥当な推測

  tags: Tag[]; // 🔵 TASK-0002確認結果より実際にネストされることを確認
  categories: Category[]; // 🔵 TASK-0002確認結果より実際にネストされることを確認
  calibre_links: CalibreWebLinkInfo[]; // 🔵 TASK-0002確認結果より（PDF item_filesでcalibre_book_id設定済みの場合のみ、serde(default)）
  detail?: unknown; // 🔵 TASK-0002確認結果より。media_type別詳細情報のJSONブロブ（serde_json::Value）。具体的な構造は未確定のためunknown。
}

// ========================================
// APIリクエスト/レスポンス
// ========================================

/**
 * GET /items クエリパラメータ
 * 🔵 信頼性: items.md ListItemsQueryより
 */
export interface ListItemsQuery {
  media_type?: MediaType;
  tag_id?: string;
  category_id?: string;
  is_favorite?: boolean;
  status?: ItemStatus;
  title?: string;
  page?: number; // default 1
  limit?: number; // default 20, max 100
}

/**
 * PATCH /items/{id}/status リクエストボディ
 * 🔵 信頼性: items.md UpdateStatusRequestより
 */
export interface UpdateStatusRequest {
  status: ItemStatus;
  consumed_date?: string | null;
}

/**
 * ページネーション情報
 * 🔵 信頼性: index.md PaginatedOk<T>より
 */
export interface Pagination {
  page: number;
  limit: number;
  total: number;
}

/**
 * 成功レスポンス（単一データ）
 * 🔵 信頼性: index.md ApiOk<T>より
 */
export interface ApiOk<T> {
  success: true;
  data: T;
}

/**
 * 成功レスポンス（ページネーション付き）
 * 🔵 信頼性: index.md PaginatedOk<T>より
 */
export interface PaginatedOk<T> {
  success: true;
  data: T[];
  pagination: Pagination;
}

/**
 * エラーレスポンス
 * 🔵 信頼性: index.md ApiErrorより
 */
export interface ApiError {
  success: false;
  error: {
    code: ApiErrorCode;
    message: string;
  };
}

/**
 * エラーコード（本機能が扱いうる範囲）
 * 🔵 信頼性: index.md エラーコード一覧より
 */
export type ApiErrorCode =
  | 'VALIDATION_ERROR'
  | 'ITEM_NOT_FOUND'
  | 'UNPROCESSABLE_ENTITY'
  | 'INTERNAL_ERROR';

// ========================================
// フロントエンド内部の状態型
// ========================================

/**
 * 一覧フィルタ状態（UI・URLクエリ同期用）
 * 🔵 信頼性: REQ-101・ヒアリング（useSearchParams採用）より
 */
export interface ItemFilterState {
  mediaTypes: MediaType[]; // 🟡 モックはチップ単一表示だが複合AND要件のため配列とする（要件から妥当な推測）
  tagIds: string[];
  categoryIds: string[];
  isFavorite: boolean;
  statuses: ItemStatus[];
  title: string;
}

/**
 * 無限スクロール1ページ分のレスポンス（useInfiniteQueryのpages配列要素）
 * 🔵 信頼性: TanStack Query useInfiniteQuery標準パターンより
 */
export interface ItemsPage {
  items: Item[];
  pagination: Pagination;
  nextPage: number | null; // pagination.totalとページ内累計件数から算出、最終ページならnull
}

// ========================================
// 信頼性レベルサマリー
// ========================================
/**
 * TASK-0003にてTASK-0002の確認結果（ItemStatus実値・ItemDetailネスト構造）を反映済み。
 * - ItemStatus: 🔵 確定（'not_started' | 'in_progress' | 'completed'）
 * - ItemDetail: 🔵 確定（tags/categories/calibre_links/detailのみネスト。他は個別API参照）
 * - その他の型: interfaces.ts（設計文書オリジナル）を踏襲
 */
