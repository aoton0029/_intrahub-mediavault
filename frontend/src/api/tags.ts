import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from './client';
import type { Tag, Category } from '@/types';

// ========================================
// fetch関数群
// ========================================

export async function fetchTags(): Promise<{ data: Tag[] }> {
  return apiClient<Tag[]>('/tags');
}

export async function fetchCategories(): Promise<{ data: Category[] }> {
  return apiClient<Category[]>('/categories');
}

export async function createTag(name: string): Promise<{ data: Tag }> {
  return apiClient<Tag>('/tags', { method: 'POST', body: { name } });
}

export async function createCategory(name: string): Promise<{ data: Category }> {
  return apiClient<Category>('/categories', { method: 'POST', body: { name } });
}

export async function attachTag(itemId: string, tagId: string): Promise<{ data: null }> {
  return apiClient<null>(`/items/${itemId}/tags/${tagId}`, { method: 'POST' });
}

export async function detachTag(itemId: string, tagId: string): Promise<{ data: null }> {
  return apiClient<null>(`/items/${itemId}/tags/${tagId}`, { method: 'DELETE' });
}

// ========================================
// TanStack Queryフック群
// ========================================

export function useTagsQuery() {
  return useQuery({
    queryKey: ['tags'],
    queryFn: fetchTags,
  });
}

export function useCategoriesQuery() {
  return useQuery({
    queryKey: ['categories'],
    queryFn: fetchCategories,
  });
}

export function useCreateTagMutation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ name }: { name: string }) => createTag(name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['tags'] });
    },
  });
}

export function useCreateCategoryMutation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ name }: { name: string }) => createCategory(name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['categories'] });
    },
  });
}

export function useAttachTagMutation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ itemId, tagId }: { itemId: string; tagId: string }) =>
      attachTag(itemId, tagId),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ['items', 'detail', variables.itemId] });
    },
  });
}

export function useDetachTagMutation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ itemId, tagId }: { itemId: string; tagId: string }) =>
      detachTag(itemId, tagId),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ['items', 'detail', variables.itemId] });
    },
  });
}
