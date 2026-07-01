import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from './client';
import type { MyList, Item } from '@/types';

export async function fetchMyLists(): Promise<{ data: MyList[] }> {
  return apiClient<MyList[]>('/mylists');
}

export async function fetchMyList(id: string): Promise<{ data: MyList & { items: Item[] } }> {
  return apiClient<MyList & { items: Item[] }>(`/mylists/${id}`);
}

export async function createMyList(name: string): Promise<{ data: MyList }> {
  return apiClient<MyList>('/mylists', { method: 'POST', body: { name } });
}

export async function addItemToMyList(listId: string, itemId: string): Promise<{ data: null }> {
  return apiClient<null>(`/mylists/${listId}/items`, { method: 'POST', body: { itemId } });
}

export async function removeItemFromMyList(listId: string, itemId: string): Promise<{ data: null }> {
  return apiClient<null>(`/mylists/${listId}/items/${itemId}`, { method: 'DELETE' });
}

export function useMyListsQuery() {
  return useQuery({
    queryKey: ['mylists'],
    queryFn: fetchMyLists,
  });
}

export function useMyListQuery(id: string) {
  return useQuery({
    queryKey: ['mylists', 'detail', id],
    queryFn: () => fetchMyList(id),
    enabled: !!id,
  });
}

export function useCreateMyListMutation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (name: string) => createMyList(name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['mylists'] });
    },
  });
}

export function useAddItemToMyListMutation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ listId, itemId }: { listId: string; itemId: string }) =>
      addItemToMyList(listId, itemId),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ['mylists', 'detail', variables.listId] });
    },
  });
}

export function useRemoveItemFromMyListMutation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ listId, itemId }: { listId: string; itemId: string }) =>
      removeItemFromMyList(listId, itemId),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ['mylists', 'detail', variables.listId] });
    },
  });
}
