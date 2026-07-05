import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { apiFetch, apiFetchPaginated } from '@/lib/api-client';
import type {
  CategoryRef,
  ExternalSearchResult,
  GeneralMediaType,
  ImportItemRequest,
  Item,
  MediaType,
  TagRef,
} from './types';

export interface ListItemsParams {
  media_type?: MediaType;
  tag_id?: string;
  category_id?: string;
  is_favorite?: boolean;
  title?: string;
  limit?: number;
  after_created_at?: string;
  after_id?: string;
}

function buildItemsQueryString(params: ListItemsParams): string {
  const search = new URLSearchParams();
  if (params.media_type) search.set('media_type', params.media_type);
  if (params.tag_id) search.set('tag_id', params.tag_id);
  if (params.category_id) search.set('category_id', params.category_id);
  if (params.is_favorite) search.set('is_favorite', 'true');
  // 空文字titleはパラメータ自体を省略する（TC-101-B02）
  if (params.title) search.set('title', params.title);
  if (params.after_created_at) search.set('after_created_at', params.after_created_at);
  if (params.after_id) search.set('after_id', params.after_id);
  search.set('limit', String(params.limit ?? 20));
  return search.toString();
}

interface ItemsPagination {
  limit: number;
  has_more: boolean;
  next_after_created_at: string | null;
  next_after_id: string | null;
}

export async function fetchItems(
  params: ListItemsParams,
): Promise<{ data: Item[]; pagination: ItemsPagination }> {
  const qs = buildItemsQueryString(params);
  return apiFetchPaginated<Item[]>(`/items?${qs}`);
}

export type ItemFilters = Omit<ListItemsParams, 'after_created_at' | 'after_id' | 'limit'>;

type ItemsCursor = { after_created_at: string; after_id: string } | null;

export function useInfiniteItemsQuery(filters: ItemFilters) {
  return useInfiniteQuery({
    queryKey: ['items', filters],
    queryFn: ({ pageParam }: { pageParam: ItemsCursor }) =>
      fetchItems({ ...filters, ...(pageParam ?? {}), limit: 20 }),
    initialPageParam: null as ItemsCursor,
    getNextPageParam: (lastPage): ItemsCursor | undefined =>
      lastPage.pagination.has_more &&
      lastPage.pagination.next_after_created_at &&
      lastPage.pagination.next_after_id
        ? {
            after_created_at: lastPage.pagination.next_after_created_at,
            after_id: lastPage.pagination.next_after_id,
          }
        : undefined,
  });
}

export interface TagWithCount extends TagRef {
  item_count: number;
}

export interface CategoryWithCount extends CategoryRef {
  item_count: number;
}

export function useTagsQuery() {
  return useQuery({
    queryKey: ['tags'],
    queryFn: async () => (await apiFetch<TagWithCount[]>('/tags')).data,
    retry: false,
  });
}

export function useCategoriesQuery() {
  return useQuery({
    queryKey: ['categories'],
    queryFn: async () => (await apiFetch<CategoryWithCount[]>('/categories')).data,
    retry: false,
  });
}

export interface MediaTypeCounts {
  anime: number;
  movie: number;
  drama: number;
  manga: number;
  novel: number;
  game: number;
  academic_book: number;
  paper: number;
  total: number;
}

export function useMediaTypeCountsQuery() {
  return useQuery({
    queryKey: ['items-counts-by-media-type'],
    queryFn: async () => (await apiFetch<MediaTypeCounts>('/items/counts-by-media-type')).data,
    retry: false,
  });
}

export async function searchExternalItems(params: {
  media_type: GeneralMediaType;
  q: string;
}): Promise<ExternalSearchResult[]> {
  const search = new URLSearchParams({ media_type: params.media_type, q: params.q });
  return (await apiFetch<ExternalSearchResult[]>(`/items/search?${search.toString()}`)).data;
}

export function useExternalSearchQuery(mediaType: GeneralMediaType, q: string) {
  const trimmed = q.trim();
  return useQuery({
    queryKey: ['items-search', mediaType, trimmed],
    queryFn: () => searchExternalItems({ media_type: mediaType, q: trimmed }),
    enabled: trimmed.length > 0,
    retry: false,
  });
}

export async function importItem(body: ImportItemRequest): Promise<Item> {
  return (
    await apiFetch<Item>('/items/import', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    })
  ).data;
}

export function useImportItemMutation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: importItem,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['items'] });
      queryClient.invalidateQueries({ queryKey: ['items-counts-by-media-type'] });
    },
  });
}
