import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import type { UseQueryResult } from '@tanstack/react-query';
import { apiClient } from './client';
import { ApiClientError } from '@/types';
import type { ItemLink, ItemFile, ItemTrailer, Pagination } from '@/types';

// ========================================
// ItemLink CRUD
// ========================================

export async function fetchItemLinks(
  itemId: string
): Promise<{ data: ItemLink[]; pagination?: Pagination }> {
  return apiClient<ItemLink[]>(`/items/${itemId}/links`);
}

export async function createItemLink(
  itemId: string,
  body: { url: string; label: string }
): Promise<{ data: ItemLink; pagination?: Pagination }> {
  return apiClient<ItemLink>(`/items/${itemId}/links`, { method: 'POST', body });
}

export async function updateItemLink(
  itemId: string,
  linkId: string,
  body: { url?: string; label?: string }
): Promise<{ data: ItemLink; pagination?: Pagination }> {
  return apiClient<ItemLink>(`/items/${itemId}/links/${linkId}`, { method: 'PATCH', body });
}

export async function deleteItemLink(
  itemId: string,
  linkId: string
): Promise<{ data: null; pagination?: Pagination }> {
  return apiClient<null>(`/items/${itemId}/links/${linkId}`, { method: 'DELETE' });
}

export function useItemLinksQuery(
  itemId: string
): UseQueryResult<{ data: ItemLink[]; pagination?: Pagination }, Error> {
  return useQuery({
    queryKey: ['items', 'links', itemId],
    queryFn: () => fetchItemLinks(itemId),
    enabled: !!itemId,
  });
}

export function useCreateItemLinkMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: { url: string; label: string }) => createItemLink(itemId, body),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['items', 'links', itemId] });
    },
  });
}

export function useUpdateItemLinkMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ linkId, body }: { linkId: string; body: { url?: string; label?: string } }) =>
      updateItemLink(itemId, linkId, body),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['items', 'links', itemId] });
    },
  });
}

export function useDeleteItemLinkMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (linkId: string) => deleteItemLink(itemId, linkId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['items', 'links', itemId] });
    },
  });
}

// ========================================
// ItemFile CRUD
// ========================================

export async function fetchItemFiles(
  itemId: string
): Promise<{ data: ItemFile[]; pagination?: Pagination }> {
  return apiClient<ItemFile[]>(`/items/${itemId}/files`);
}

export async function deleteItemFile(
  itemId: string,
  fileId: string
): Promise<{ data: null; pagination?: Pagination }> {
  return apiClient<null>(`/items/${itemId}/files/${fileId}`, { method: 'DELETE' });
}

export function useItemFilesQuery(
  itemId: string
): UseQueryResult<{ data: ItemFile[]; pagination?: Pagination }, Error> {
  return useQuery({
    queryKey: ['items', 'files', itemId],
    queryFn: () => fetchItemFiles(itemId),
    enabled: !!itemId,
  });
}

export function useDeleteItemFileMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (fileId: string) => deleteItemFile(itemId, fileId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['items', 'files', itemId] });
    },
  });
}

export function uploadItemFile(
  itemId: string,
  req: { file: File; fileType: string; label?: string },
  onProgress?: (percent: number) => void
): Promise<{ data: ItemFile; pagination?: Pagination }> {
  return new Promise((resolve, reject) => {
    const BASE_URL = import.meta.env.VITE_API_BASE_URL ?? 'http://localhost:8080/api/v1';
    const formData = new FormData();
    formData.append('file', req.file);
    formData.append('fileType', req.fileType);
    if (req.label) formData.append('label', req.label);

    const xhr = new XMLHttpRequest();
    xhr.open('POST', `${BASE_URL}/items/${itemId}/files/upload`);

    xhr.upload.addEventListener('progress', (e) => {
      if (e.lengthComputable && onProgress) {
        onProgress(Math.round((e.loaded / e.total) * 100));
      }
    });

    xhr.addEventListener('load', () => {
      let json: { success: true; data: ItemFile; pagination?: Pagination } | { success: false; error: { code: string; message: string } };
      try {
        json = JSON.parse(xhr.responseText);
      } catch {
        reject(new ApiClientError('PARSE_ERROR', 'レスポンスの解析に失敗しました'));
        return;
      }
      if (!json.success) {
        reject(new ApiClientError(json.error.code, json.error.message));
        return;
      }
      resolve({ data: json.data, pagination: json.pagination });
    });

    xhr.addEventListener('error', () => {
      reject(new ApiClientError('NETWORK_ERROR', 'ネットワークエラーが発生しました'));
    });

    xhr.send(formData);
  });
}

export function useUploadItemFileMutation(
  itemId: string,
  onProgress?: (percent: number) => void
) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (req: { file: File; fileType: string; label?: string }) =>
      uploadItemFile(itemId, req, onProgress),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['items', 'files', itemId] });
    },
  });
}

// ========================================
// ItemTrailer CRUD
// ========================================

export async function fetchItemTrailers(
  itemId: string
): Promise<{ data: ItemTrailer[]; pagination?: Pagination }> {
  return apiClient<ItemTrailer[]>(`/items/${itemId}/trailers`);
}

export async function createItemTrailer(
  itemId: string,
  body: { url: string; label?: string }
): Promise<{ data: ItemTrailer; pagination?: Pagination }> {
  return apiClient<ItemTrailer>(`/items/${itemId}/trailers`, { method: 'POST', body });
}

export async function updateItemTrailer(
  itemId: string,
  trailerId: string,
  body: { url?: string; label?: string }
): Promise<{ data: ItemTrailer; pagination?: Pagination }> {
  return apiClient<ItemTrailer>(`/items/${itemId}/trailers/${trailerId}`, {
    method: 'PATCH',
    body,
  });
}

export async function deleteItemTrailer(
  itemId: string,
  trailerId: string
): Promise<{ data: null; pagination?: Pagination }> {
  return apiClient<null>(`/items/${itemId}/trailers/${trailerId}`, { method: 'DELETE' });
}

export function useItemTrailersQuery(
  itemId: string
): UseQueryResult<{ data: ItemTrailer[]; pagination?: Pagination }, Error> {
  return useQuery({
    queryKey: ['items', 'trailers', itemId],
    queryFn: () => fetchItemTrailers(itemId),
    enabled: !!itemId,
  });
}

export function useCreateItemTrailerMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: { url: string; label?: string }) => createItemTrailer(itemId, body),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['items', 'trailers', itemId] });
    },
  });
}

export function useUpdateItemTrailerMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({
      trailerId,
      body,
    }: {
      trailerId: string;
      body: { url?: string; label?: string };
    }) => updateItemTrailer(itemId, trailerId, body),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['items', 'trailers', itemId] });
    },
  });
}

export function useDeleteItemTrailerMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (trailerId: string) => deleteItemTrailer(itemId, trailerId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['items', 'trailers', itemId] });
    },
  });
}
