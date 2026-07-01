import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from './client';
import type { Staff, ItemStaff } from '@/types';

export async function fetchStaffList(): Promise<{ data: Staff[] }> {
  return apiClient<Staff[]>('/staff');
}

export async function createStaff(name: string): Promise<{ data: Staff }> {
  return apiClient<Staff>('/staff', { method: 'POST', body: { name } });
}

export async function attachStaff(
  itemId: string,
  body: { staffId: string; role: string; characterName?: string }
): Promise<{ data: ItemStaff }> {
  return apiClient<ItemStaff>(`/items/${itemId}/staff`, { method: 'POST', body });
}

export async function detachStaff(itemId: string, itemStaffId: string): Promise<{ data: null }> {
  return apiClient<null>(`/items/${itemId}/staff/${itemStaffId}`, { method: 'DELETE' });
}

export function useStaffListQuery() {
  return useQuery({
    queryKey: ['staff'],
    queryFn: fetchStaffList,
  });
}

export function useCreateStaffMutation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ name }: { name: string }) => createStaff(name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['staff'] });
    },
  });
}

export function useAttachStaffMutation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({
      itemId,
      staffId,
      role,
      characterName,
    }: {
      itemId: string;
      staffId: string;
      role: string;
      characterName?: string;
    }) => attachStaff(itemId, { staffId, role, characterName }),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ['items', 'detail', variables.itemId] });
    },
  });
}

export function useDetachStaffMutation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ itemId, itemStaffId }: { itemId: string; itemStaffId: string }) =>
      detachStaff(itemId, itemStaffId),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ['items', 'detail', variables.itemId] });
    },
  });
}
