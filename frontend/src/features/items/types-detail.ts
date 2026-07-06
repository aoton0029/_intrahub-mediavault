/**
 * アイテム詳細ページ（/items/:id）向けの型定義。
 * 一覧向けの `./types` の `Item`（軽量型）とは別に、GET /items/{id} の
 * ItemDetail 及び配下リソース（groups/episodes/staff/relations/links/files/trailers/mylists）
 * のバックエンド仕様（docs/backend/mediavault-api/data-model.md）に忠実な型をここに定義する。
 */
import type { CategoryRef, ItemStatus, MediaType, TagRef } from './types';

export type ItemSource = 'api' | 'manual';
export type GroupType = 'season' | 'volume' | 'chapter';
export type RelationType = 'reference' | 'dlc';
export type FileType = 'pdf' | 'image' | 'other';

export interface CalibreWebLinkInfo {
  file_id: string;
  calibre_book_id: string;
}

export interface ItemDetail {
  id: string;
  media_type: MediaType;
  title: string;
  original_title: string | null;
  description: string | null;
  cover_image_url: string | null;
  release_date: string | null;
  homepage_url: string | null;
  status: ItemStatus;
  consumed_date: string | null;
  rating: number | null;
  is_favorite: boolean;
  source: ItemSource;
  external_id: string | null;
  created_at: string;
  updated_at: string;
  /** メディア別詳細（media_typeに応じたキー集合）。本ページのv1では専用UIを持たないため緩い型に留める */
  detail: Record<string, unknown> | null;
  tags: TagRef[];
  categories: CategoryRef[];
  calibre_links: CalibreWebLinkInfo[];
}

export interface ItemGroup {
  id: string;
  item_id: string;
  parent_item_id: string | null;
  group_type: GroupType;
  group_name: string;
  number: number | null;
  display_order: number;
  created_at: string;
  updated_at: string;
}

export interface ItemEpisode {
  id: string;
  group_id: string;
  episode_number: number;
  title: string | null;
  original_title: string | null;
  air_date: string | null;
  description: string | null;
  created_at: string;
  updated_at: string;
}

export interface Staff {
  id: string;
  external_id: string | null;
  name: string;
  image_url: string | null;
  created_at: string;
}

export interface ItemStaff {
  id: string;
  item_id: string;
  staff_id: string;
  role: string;
  character_name: string | null;
}

export interface ItemRelation {
  id: string;
  item_id: string;
  related_item_id: string;
  relation_type: RelationType;
  created_at: string;
}

export interface ItemLink {
  id: string;
  item_id: string;
  url: string;
  label: string;
  created_at: string;
}

export interface ItemTrailer {
  id: string;
  item_id: string;
  url: string;
  label: string | null;
  created_at: string;
}

export interface ItemFile {
  id: string;
  item_id: string;
  path: string;
  label: string | null;
  file_type: FileType;
  calibre_book_id: string | null;
  created_at: string;
}

export interface Mylist {
  id: string;
  name: string;
  created_at: string;
}

// --- リクエストボディ ---

export interface UpdateItemRequest {
  title?: string;
  original_title?: string;
  description?: string;
  cover_image_url?: string;
  release_date?: string;
  homepage_url?: string;
  status?: ItemStatus;
  consumed_date?: string;
  rating?: number;
  is_favorite?: boolean;
}

export interface UpdateStatusRequest {
  status: ItemStatus;
  consumed_date?: string;
}

export interface CreateItemGroupRequest {
  group_type: GroupType;
  group_name: string;
  number?: number;
  display_order?: number;
  parent_item_id?: string;
}

export interface CreateItemEpisodeRequest {
  episode_number: number;
  title?: string;
  original_title?: string;
  air_date?: string;
  description?: string;
}

export interface CreateItemStaffRequest {
  staff_id: string;
  role: string;
  character_name?: string;
}

export interface CreateItemRelationRequest {
  item_id: string;
  related_item_id: string;
  relation_type: RelationType;
}

export interface CreateItemLinkRequest {
  url: string;
  label: string;
}

export interface CreateItemTrailerRequest {
  url: string;
  label?: string;
}

export interface CreateItemFileRequest {
  path: string;
  label?: string;
  file_type: FileType;
}

export interface UpdateCalibreLinkRequest {
  calibre_book_id: string;
}
