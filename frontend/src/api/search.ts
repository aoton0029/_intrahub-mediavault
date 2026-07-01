/**
 * TASK-0015: 外部API検索フック実装
 * 🔵 docs/tasks/frontend-collection-ui/TASK-0015.md より
 */
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import type { UseQueryResult } from '@tanstack/react-query';
import { apiClient } from './client';
import type { ExternalSearchQuery, ExternalSearchResultItem, ImportItemRequest, Item } from '@/types';

// ========================================
// fetch関数群
// ========================================

export async function searchExternalItems(
  query: ExternalSearchQuery
): Promise<ExternalSearchResultItem[]> {
  const params = new URLSearchParams({
    media_type: query.mediaType,
    q: query.q,
  });
  const { data } = await apiClient<ExternalSearchResultItem[]>(`/items/search?${params.toString()}`);
  return data;
}

export async function importItem(body: ImportItemRequest): Promise<Item> {
  const { data } = await apiClient<Item>('/items/import', { method: 'POST', body });
  return data;
}

// ========================================
// TanStack Queryフック群
// ========================================

export function useExternalSearchQuery(
  query: ExternalSearchQuery
): UseQueryResult<ExternalSearchResultItem[], Error> {
  return useQuery({
    queryKey: ['search', query],
    queryFn: () => searchExternalItems(query),
    enabled: !!query.q,
  });
}

export function useImportItemMutation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: ImportItemRequest) => importItem(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['items'] });
    },
  });
}
