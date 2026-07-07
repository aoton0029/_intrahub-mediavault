import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { apiFetch } from '@/lib/api-client';
import type {
  CreateItemEpisodeRequest,
  CreateItemFileRequest,
  CreateItemGroupRequest,
  CreateItemLinkRequest,
  CreateItemRelationRequest,
  CreateItemStaffRequest,
  CreateItemStreamingLinkRequest,
  CreateItemTrailerRequest,
  ItemDetail,
  ItemEpisode,
  ItemFile,
  ItemGroup,
  ItemLink,
  ItemRelation,
  ItemStaff,
  ItemStreamingLink,
  ItemTrailer,
  Mylist,
  Staff,
  UpdateCalibreLinkRequest,
  UpdateItemRequest,
  UpdateStatusRequest,
} from './types-detail';

function invalidateItemDetail(queryClient: ReturnType<typeof useQueryClient>, itemId: string) {
  queryClient.invalidateQueries({ queryKey: ['item', itemId] });
}

function invalidateItemListsToo(queryClient: ReturnType<typeof useQueryClient>) {
  queryClient.invalidateQueries({ queryKey: ['items'] });
  queryClient.invalidateQueries({ queryKey: ['items-counts-by-status'] });
}

// --- アイテム本体 ---

export function useItemDetailQuery(itemId: string) {
  return useQuery({
    queryKey: ['item', itemId],
    queryFn: async () => (await apiFetch<ItemDetail>(`/items/${itemId}`)).data,
    enabled: Boolean(itemId),
  });
}

export function useUpdateItemMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: UpdateItemRequest) =>
      apiFetch(`/items/${itemId}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      }),
    onSuccess: () => {
      invalidateItemDetail(queryClient, itemId);
      invalidateItemListsToo(queryClient);
    },
  });
}

export function useUpdateItemStatusMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: UpdateStatusRequest) =>
      apiFetch(`/items/${itemId}/status`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      }),
    onSuccess: () => {
      invalidateItemDetail(queryClient, itemId);
      invalidateItemListsToo(queryClient);
    },
  });
}

// --- タグ ---

export function useAddItemTagMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (tagId: string) => apiFetch(`/items/${itemId}/tags/${tagId}`, { method: 'POST' }),
    onSuccess: () => invalidateItemDetail(queryClient, itemId),
  });
}

export function useRemoveItemTagMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (tagId: string) =>
      apiFetch(`/items/${itemId}/tags/${tagId}`, { method: 'DELETE' }),
    onSuccess: () => invalidateItemDetail(queryClient, itemId),
  });
}

// --- カテゴリ ---

export function useAddItemCategoryMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (categoryId: string) =>
      apiFetch(`/items/${itemId}/categories/${categoryId}`, { method: 'POST' }),
    onSuccess: () => invalidateItemDetail(queryClient, itemId),
  });
}

export function useRemoveItemCategoryMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (categoryId: string) =>
      apiFetch(`/items/${itemId}/categories/${categoryId}`, { method: 'DELETE' }),
    onSuccess: () => invalidateItemDetail(queryClient, itemId),
  });
}

// --- シーズン/巻/章（groups）・話数（episodes） ---

export function useItemGroupsQuery(itemId: string) {
  return useQuery({
    queryKey: ['item', itemId, 'groups'],
    queryFn: async () => (await apiFetch<ItemGroup[]>(`/items/${itemId}/groups`)).data,
    enabled: Boolean(itemId),
  });
}

export function useCreateItemGroupMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: CreateItemGroupRequest) =>
      apiFetch<ItemGroup>(`/items/${itemId}/groups`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['item', itemId, 'groups'] });
    },
  });
}

export function useGroupEpisodesQuery(groupId: string) {
  return useQuery({
    queryKey: ['group', groupId, 'episodes'],
    queryFn: async () => (await apiFetch<ItemEpisode[]>(`/groups/${groupId}/episodes`)).data,
    enabled: Boolean(groupId),
  });
}

export function useCreateItemEpisodeMutation(groupId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: CreateItemEpisodeRequest) =>
      apiFetch<ItemEpisode>(`/groups/${groupId}/episodes`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['group', groupId, 'episodes'] });
    },
  });
}

// --- スタッフ ---

export function useStaffListQuery() {
  return useQuery({
    queryKey: ['staff'],
    queryFn: async () => (await apiFetch<Staff[]>('/staff')).data,
    retry: false,
  });
}

export function useItemStaffQuery(itemId: string) {
  return useQuery({
    queryKey: ['item', itemId, 'staff'],
    queryFn: async () => (await apiFetch<ItemStaff[]>(`/items/${itemId}/staff`)).data,
    enabled: Boolean(itemId),
  });
}

export function useCreateItemStaffMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: CreateItemStaffRequest) =>
      apiFetch<ItemStaff>(`/items/${itemId}/staff`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['item', itemId, 'staff'] }),
  });
}

export function useRemoveItemStaffMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (itemStaffId: string) =>
      apiFetch(`/items/${itemId}/staff/${itemStaffId}`, { method: 'DELETE' }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['item', itemId, 'staff'] }),
  });
}

// --- 関連作品（item-relations） ---

export function useItemRelationsQuery(itemId: string) {
  return useQuery({
    queryKey: ['item', itemId, 'relations'],
    queryFn: async () => (await apiFetch<ItemRelation[]>(`/items/${itemId}/relations`)).data,
    enabled: Boolean(itemId),
  });
}

export function useCreateItemRelationMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: CreateItemRelationRequest) =>
      apiFetch<ItemRelation>('/item-relations', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['item', itemId, 'relations'] }),
  });
}

export function useRemoveItemRelationMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (relationId: string) =>
      apiFetch(`/item-relations/${relationId}`, { method: 'DELETE' }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['item', itemId, 'relations'] }),
  });
}

// --- リンク ---

export function useItemLinksQuery(itemId: string) {
  return useQuery({
    queryKey: ['item', itemId, 'links'],
    queryFn: async () => (await apiFetch<ItemLink[]>(`/items/${itemId}/links`)).data,
    enabled: Boolean(itemId),
  });
}

export function useCreateItemLinkMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: CreateItemLinkRequest) =>
      apiFetch<ItemLink>(`/items/${itemId}/links`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['item', itemId, 'links'] }),
  });
}

export function useRemoveItemLinkMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (linkId: string) =>
      apiFetch(`/items/${itemId}/links/${linkId}`, { method: 'DELETE' }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['item', itemId, 'links'] }),
  });
}

// --- 配信URL ---

export function useItemStreamingLinksQuery(itemId: string) {
  return useQuery({
    queryKey: ['item', itemId, 'streaming-links'],
    queryFn: async () =>
      (await apiFetch<ItemStreamingLink[]>(`/items/${itemId}/streaming-links`)).data,
    enabled: Boolean(itemId),
  });
}

export function useCreateItemStreamingLinkMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: CreateItemStreamingLinkRequest) =>
      apiFetch<ItemStreamingLink>(`/items/${itemId}/streaming-links`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      }),
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: ['item', itemId, 'streaming-links'] }),
  });
}

export function useRemoveItemStreamingLinkMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (linkId: string) =>
      apiFetch(`/items/${itemId}/streaming-links/${linkId}`, { method: 'DELETE' }),
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: ['item', itemId, 'streaming-links'] }),
  });
}

// --- ファイル ---

export function useItemFilesQuery(itemId: string) {
  return useQuery({
    queryKey: ['item', itemId, 'files'],
    queryFn: async () => (await apiFetch<ItemFile[]>(`/items/${itemId}/files`)).data,
    enabled: Boolean(itemId),
  });
}

export function useCreateItemFileMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: CreateItemFileRequest) =>
      apiFetch<ItemFile>(`/items/${itemId}/files`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['item', itemId, 'files'] }),
  });
}

export function useRemoveItemFileMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (fileId: string) =>
      apiFetch(`/items/${itemId}/files/${fileId}`, { method: 'DELETE' }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['item', itemId, 'files'] }),
  });
}

export function useUpdateCalibreLinkMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ fileId, body }: { fileId: string; body: UpdateCalibreLinkRequest }) =>
      apiFetch<ItemFile>(`/items/${itemId}/files/${fileId}/calibre-link`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['item', itemId, 'files'] });
      invalidateItemDetail(queryClient, itemId);
    },
  });
}

// --- トレーラー ---

export function useItemTrailersQuery(itemId: string) {
  return useQuery({
    queryKey: ['item', itemId, 'trailers'],
    queryFn: async () => (await apiFetch<ItemTrailer[]>(`/items/${itemId}/trailers`)).data,
    enabled: Boolean(itemId),
  });
}

export function useCreateItemTrailerMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: CreateItemTrailerRequest) =>
      apiFetch<ItemTrailer>(`/items/${itemId}/trailers`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['item', itemId, 'trailers'] }),
  });
}

export function useRemoveItemTrailerMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (trailerId: string) =>
      apiFetch(`/items/${itemId}/trailers/${trailerId}`, { method: 'DELETE' }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['item', itemId, 'trailers'] }),
  });
}

// --- マイリスト ---

export function useMylistsQuery() {
  return useQuery({
    queryKey: ['mylists'],
    queryFn: async () => (await apiFetch<Mylist[]>('/mylists')).data,
    retry: false,
  });
}

export function useItemMylistsQuery(itemId: string) {
  return useQuery({
    queryKey: ['item', itemId, 'mylists'],
    queryFn: async () => (await apiFetch<Mylist[]>(`/items/${itemId}/mylists`)).data,
    enabled: Boolean(itemId),
  });
}

export function useAddItemToMylistMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (mylistId: string) =>
      apiFetch(`/mylists/${mylistId}/items`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ item_id: itemId }),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['item', itemId, 'mylists'] });
    },
  });
}

export function useRemoveItemFromMylistMutation(itemId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (mylistId: string) =>
      apiFetch(`/mylists/${mylistId}/items/${itemId}`, { method: 'DELETE' }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['item', itemId, 'mylists'] });
    },
  });
}
