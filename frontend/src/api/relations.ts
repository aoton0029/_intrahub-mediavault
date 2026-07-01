import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from './client';
import type { ItemRelation, RelationType } from '@/types';

export interface CreateItemRelationRequest {
  relatedItemId: string;
  relationType: RelationType;
}

export async function fetchItemRelations(itemId: string): Promise<{ data: ItemRelation[] }> {
  return apiClient<ItemRelation[]>(`/items/${itemId}/relations`);
}

export async function createItemRelation(
  itemId: string,
  body: CreateItemRelationRequest
): Promise<{ data: ItemRelation }> {
  return apiClient<ItemRelation>('/item-relations', {
    method: 'POST',
    body: {
      item_id: itemId,
      related_item_id: body.relatedItemId,
      relation_type: body.relationType,
    },
  });
}

export async function deleteItemRelation(relationId: string): Promise<{ data: null }> {
  return apiClient<null>(`/item-relations/${relationId}`, { method: 'DELETE' });
}

export function useItemRelationsQuery(itemId: string) {
  return useQuery({
    queryKey: ['items', 'relations', itemId],
    queryFn: () => fetchItemRelations(itemId),
    enabled: !!itemId,
  });
}

export function useCreateItemRelationMutation(itemId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (body: CreateItemRelationRequest) => createItemRelation(itemId, body),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['items', itemId] });
      queryClient.invalidateQueries({ queryKey: ['items', 'relations', itemId] });
    },
  });
}

export function useDeleteItemRelationMutation(itemId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (relationId: string) => deleteItemRelation(relationId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['items', itemId] });
      queryClient.invalidateQueries({ queryKey: ['items', 'relations', itemId] });
    },
  });
}
