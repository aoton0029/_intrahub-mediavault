/**
 * TASK-0009: items APIフック実装
 * 🔵 docs/tasks/frontend-collection-ui/TASK-0009.md より
 */
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import type { UseQueryResult } from '@tanstack/react-query';
import { apiClient } from './client';
import type { Item, ItemListFilters, UpdateItemStatusRequest, Pagination } from '@/types';

// ========================================
// URLSearchParams変換ユーティリティ
// ========================================

/**
 * 【機能概要】: ItemListFiltersをURLSearchParamsに変換する
 * 【実装方針】: undefinedフィールドをスキップし、キャメルケース→スネークケース変換を行う
 * 【テスト対応】: TC-IQ-N-02, TC-IQ-N-03, TC-IQ-N-11
 * 🔵 TASK-0009.md 注意事項・requirements.md URLクエリパラメータ変換ルールより
 */
function filtersToSearchParams(filters: ItemListFilters): URLSearchParams {
  const params = new URLSearchParams();

  // 【フィールド変換】: 各フィールドをスネークケースのクエリパラメータに変換
  // undefinedの場合はパラメータを追加しない
  if (filters.mediaType !== undefined) {
    params.set('media_type', filters.mediaType); // 🔵 mediaType → media_type
  }
  if (filters.tagId !== undefined) {
    params.set('tag_id', filters.tagId); // 🔵 tagId → tag_id
  }
  if (filters.categoryId !== undefined) {
    params.set('category_id', filters.categoryId); // 🔵 categoryId → category_id
  }
  if (filters.isFavorite !== undefined) {
    params.set('is_favorite', String(filters.isFavorite)); // 🔵 isFavorite → is_favorite
  }
  if (filters.status !== undefined) {
    params.set('status', filters.status); // 🔵 status → status
  }
  if (filters.page !== undefined) {
    params.set('page', String(filters.page)); // 🔵 page → page
  }
  if (filters.limit !== undefined) {
    params.set('limit', String(filters.limit)); // 🔵 limit → limit
  }

  return params;
}

// ========================================
// fetch関数群
// ========================================

/**
 * 【機能概要】: アイテム一覧をフィルタ条件付きで取得する
 * 【実装方針】: filtersをURLSearchParamsに変換してGET /itemsに送信
 * 【テスト対応】: TC-IQ-N-01, TC-IQ-N-02, TC-IQ-N-03
 * 🔵 TASK-0009.md 実装詳細1・requirements.md 2-1より
 */
export async function fetchItems(
  filters: ItemListFilters
): Promise<{ data: Item[]; pagination?: Pagination }> {
  const params = filtersToSearchParams(filters);
  const queryString = params.toString();
  // 【URLパス構築】: クエリパラメータが空の場合は ? を付けない
  const path = queryString ? `/items?${queryString}` : '/items';
  return apiClient<Item[]>(path);
}

/**
 * 【機能概要】: アイテム詳細を1件取得する
 * 【実装方針】: GET /items/:id を呼び出す
 * 【テスト対応】: TC-IQ-N-05
 * 🔵 TASK-0009.md 完了条件・requirements.md 2-2より
 */
export async function fetchItem(id: string): Promise<{ data: Item; pagination?: Pagination }> {
  return apiClient<Item>(`/items/${id}`);
}

/**
 * 【機能概要】: アイテムを削除する
 * 【実装方針】: DELETE /items/:id を呼び出す（204 No Content想定）
 * 【テスト対応】: TC-IQ-N-08, TC-IQ-B-04, TC-IQ-E-04
 * 🔵 TASK-0009.md 完了条件・requirements.md 2-3より
 */
export async function deleteItem(id: string): Promise<{ data: null; pagination?: Pagination }> {
  return apiClient<null>(`/items/${id}`, { method: 'DELETE' });
}

/**
 * 【機能概要】: アイテムのステータスを更新する
 * 【実装方針】: PATCH /items/:id/status にUpdateItemStatusRequestを送信
 * 【テスト対応】: TC-IQ-N-09, TC-IQ-N-10, TC-IQ-E-03
 * 🔵 TASK-0009.md 完了条件・requirements.md 2-4より
 */
export async function updateItemStatus(
  id: string,
  body: UpdateItemStatusRequest
): Promise<{ data: Item; pagination?: Pagination }> {
  return apiClient<Item>(`/items/${id}/status`, { method: 'PATCH', body });
}

// ========================================
// TanStack Queryフック群
// ========================================

/**
 * 【機能概要】: アイテム一覧取得クエリフック
 * 【実装方針】: queryKey=['items', filters]でフィルタごとにキャッシュを分離
 * 【テスト対応】: TC-IQ-N-01, TC-IQ-N-02, TC-IQ-N-03, TC-IQ-N-06, TC-IQ-E-01
 * 🔵 TASK-0009.md 実装詳細2・dataflow.md「機能1: 全体一覧の閲覧・絞り込み」より
 */
export function useItemsQuery(
  filters: ItemListFilters
): UseQueryResult<{ data: Item[]; pagination?: Pagination }, Error> {
  return useQuery({
    // 【キャッシュキー】: filtersオブジェクト全体をキーに含め、フィルタ変更で自動再取得
    queryKey: ['items', filters],
    queryFn: () => fetchItems(filters),
  });
}

/**
 * 【機能概要】: アイテム詳細取得クエリフック
 * 【実装方針】: queryKey=['items','detail',id]で一覧と区別してキャッシュ。id未指定時はスキップ
 * 【テスト対応】: TC-IQ-N-05, TC-IQ-N-07, TC-IQ-B-01, TC-IQ-E-02, TC-IQ-E-05
 * 🔵 TASK-0009.md 実装詳細2・完了条件より
 */
export function useItemQuery(
  id: string
): UseQueryResult<{ data: Item; pagination?: Pagination }, Error> {
  return useQuery({
    // 【キャッシュキー】: 一覧クエリと区別するため 'detail' を挟む
    queryKey: ['items', 'detail', id],
    queryFn: () => fetchItem(id),
    // 【クエリ無効化】: idが空文字/undefinedの場合はfetchしない 🔵 TASK-0009.md「enabled: !!id」より
    enabled: !!id,
  });
}

/**
 * 【機能概要】: アイテム削除ミューテーションフック
 * 【実装方針】: 成功時に['items']配下の全クエリを無効化して一覧を自動更新
 * 【テスト対応】: TC-IQ-N-08, TC-IQ-B-04, TC-IQ-E-04
 * 🔵 TASK-0009.md 実装詳細3・dataflow.md「データ整合性の保証」より
 */
export function useDeleteItemMutation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => deleteItem(id),
    onSuccess: () => {
      // 【キャッシュ無効化】: 削除後に['items']配下の全クエリを無効化し、一覧を再取得させる
      // 🔵 TASK-0009.md「成功時に['items']配下の一覧クエリをinvalidateQueriesで無効化する」より
      queryClient.invalidateQueries({ queryKey: ['items'] });
    },
  });
}

/**
 * 【機能概要】: アイテムステータス更新ミューテーションフック
 * 【実装方針】: 成功時に一覧クエリと対象の詳細クエリを両方無効化
 * 【テスト対応】: TC-IQ-N-09, TC-IQ-N-10, TC-IQ-E-03
 * 🔵 TASK-0009.md 実装詳細3・完了条件より
 */
export function useUpdateItemStatusMutation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, body }: { id: string; body: UpdateItemStatusRequest }) =>
      updateItemStatus(id, body),
    onSuccess: (_data, variables) => {
      // 【一覧キャッシュ無効化】: ステータス変更後に全一覧クエリを無効化
      // 🔵 TASK-0009.md「成功時に対象アイテムの詳細クエリと一覧クエリを無効化する」より
      queryClient.invalidateQueries({ queryKey: ['items'] });
      // 【詳細キャッシュ無効化】: 対象アイテムの詳細キャッシュも無効化
      queryClient.invalidateQueries({ queryKey: ['items', 'detail', variables.id] });
    },
  });
}
