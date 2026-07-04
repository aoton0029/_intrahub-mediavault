import { apiRequest } from '../../lib/apiClient';
import type {
  ApiOk,
  Item,
  ItemDetail,
  ListItemsQuery,
  PaginatedOk,
  UpdateStatusRequest,
} from './types';

/**
 * items feature APIクライアント関数群
 *
 * TASK-0004のapiRequestを利用し、TASK-0003の型定義に基づいてリクエストを組み立てる。
 * 関連: docs/tasks/collection-browsing/TASK-0005.md
 */

function buildListItemsQueryString(query: ListItemsQuery): string {
  const params = new URLSearchParams();

  if (query.media_type !== undefined) {
    params.set('media_type', query.media_type);
  }
  if (query.tag_id !== undefined && query.tag_id !== '') {
    params.set('tag_id', query.tag_id);
  }
  if (query.category_id !== undefined && query.category_id !== '') {
    params.set('category_id', query.category_id);
  }
  if (query.is_favorite !== undefined) {
    params.set('is_favorite', query.is_favorite ? 'true' : 'false');
  }
  if (query.status !== undefined) {
    params.set('status', query.status);
  }
  if (query.title !== undefined && query.title !== '') {
    params.set('title', query.title);
  }
  if (query.page !== undefined) {
    params.set('page', String(query.page));
  }
  if (query.limit !== undefined) {
    params.set('limit', String(query.limit));
  }

  const qs = params.toString();
  return qs === '' ? '/items' : `/items?${qs}`;
}

/**
 * GET /items — アイテム一覧を取得する。
 */
export async function listItems(query: ListItemsQuery): Promise<PaginatedOk<Item[]>> {
  return apiRequest<PaginatedOk<Item[]>>(buildListItemsQueryString(query));
}

/**
 * GET /items/{id} — アイテム詳細を取得する。
 */
export async function getItemDetail(id: string): Promise<ApiOk<ItemDetail>> {
  return apiRequest<ApiOk<ItemDetail>>(`/items/${id}`);
}

/**
 * PATCH /items/{id}/status — アイテムのステータスを更新する。
 */
export async function updateItemStatus(
  id: string,
  body: UpdateStatusRequest,
): Promise<ApiOk<Item>> {
  return apiRequest<ApiOk<Item>>(`/items/${id}/status`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
}

/**
 * DELETE /items/{id} — アイテムを削除する（204 No Content）。
 */
export async function deleteItem(id: string): Promise<void> {
  await apiRequest<void>(`/items/${id}`, { method: 'DELETE' });
}
