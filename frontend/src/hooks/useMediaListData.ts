import { useInfiniteQuery, useQuery, type InfiniteData } from "@tanstack/react-query";
import type { MediaCardProps } from "@/components/shared";
import { apiFetch } from "@/lib/apiClient";

export type MediaType = "anime" | "movie" | "drama" | "manga" | "novel" | "game" | "academic_book";
type ItemStatus = "not_started" | "in_progress" | "done" | "completed" | string;

type TagRef = {
  id: string;
  name: string;
};

type CategoryRef = {
  id: string;
  name: string;
};

export type ItemWithRefs = {
  id: string;
  media_type: MediaType;
  title: string;
  original_title: string | null;
  status: ItemStatus;
  rating: number | null;
  is_favorite: boolean;
  cover_image_url: string | null;
  release_date: string | null;
  consumed_date: string | null;
  updated_at?: string | null;
  tags: TagRef[];
  categories: CategoryRef[];
};

export type PaginatedItemsResponse = {
  success: boolean;
  data: ItemWithRefs[];
  pagination: {
    has_more: boolean;
    next_after_created_at: string | null;
    next_after_id: string | null;
    next_after_value?: string | null;
  };
};

type TagOption = TagRef & { item_count: number };
type CategoryOption = CategoryRef & { item_count: number };

type TagResponse = {
  success: boolean;
  data: TagOption[];
};

type CategoryResponse = {
  success: boolean;
  data: CategoryOption[];
};

export type Cursor = {
  afterCreatedAt: string | null;
  afterId: string | null;
  afterValue: string | null;
};

export type MediaListFilters = {
  isFavorite?: boolean;
  mediaType?: MediaType;
  tagId?: string;
  categoryId?: string;
  title?: string;
  sort?: string;
  status?: string;
};

export const MEDIA_TYPE_LABELS: Record<MediaType, string> = {
  anime: "Anime",
  movie: "Movie",
  drama: "Drama",
  manga: "Manga",
  novel: "Novel",
  game: "Game",
  academic_book: "Academic Book",
};

type UseMediaListDataOptions = {
  mediaTypeOverride?: MediaType;
  getBadgeLabel?: (item: ItemWithRefs) => string;
  getItemHref?: (item: ItemWithRefs) => string;
};

export function buildQueryString(params: Record<string, string | number | boolean | undefined | null>) {
  const searchParams = new URLSearchParams();

  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined && value !== null && value !== "") {
      searchParams.set(key, String(value));
    }
  }

  const queryString = searchParams.toString();
  return queryString ? `?${queryString}` : "";
}

async function fetchItemsPage(filters: MediaListFilters, pageParam: Cursor) {
  const response = await apiFetch(
    `/items${buildQueryString({
      media_type: filters.mediaType,
      is_favorite: filters.isFavorite,
      tag_id: filters.tagId,
      category_id: filters.categoryId,
      title: filters.title,
      status: filters.status,
      sort: filters.sort,
      limit: 20,
      after_created_at: pageParam.afterCreatedAt,
      after_id: pageParam.afterId,
      after_value: pageParam.afterValue,
    })}`,
  );

  if (!response.ok) {
    throw new Error(`Failed to fetch items: ${response.status}`);
  }

  return (await response.json()) as PaginatedItemsResponse;
}

async function fetchTags() {
  const response = await apiFetch("/tags");

  if (!response.ok) {
    throw new Error(`Failed to fetch tags: ${response.status}`);
  }

  const json = (await response.json()) as TagResponse;
  return json.data;
}

async function fetchCategories() {
  const response = await apiFetch("/categories");

  if (!response.ok) {
    throw new Error(`Failed to fetch categories: ${response.status}`);
  }

  const json = (await response.json()) as CategoryResponse;
  return json.data;
}

export function mapItemToMediaCard(item: ItemWithRefs, options?: UseMediaListDataOptions): MediaCardProps {
  return {
    id: item.id,
    title: item.original_title || item.title,
    badge: options?.getBadgeLabel?.(item) ?? MEDIA_TYPE_LABELS[item.media_type],
    rating: item.rating ?? undefined,
    favorite: item.is_favorite,
    href: options?.getItemHref?.(item) ?? `/media/${item.id}`,
    imageUrl: item.cover_image_url,
    variant: "compact",
  };
}

export function useMediaListData(filters: MediaListFilters, options?: UseMediaListDataOptions) {
  const effectiveFilters = options?.mediaTypeOverride
    ? { ...filters, mediaType: options.mediaTypeOverride }
    : filters;

  const itemsQuery = useInfiniteQuery<PaginatedItemsResponse, Error, InfiniteData<PaginatedItemsResponse, Cursor>, [string, MediaListFilters], Cursor>({
    queryKey: ["media-list", effectiveFilters],
    queryFn: ({ pageParam }) => fetchItemsPage(effectiveFilters, pageParam),
    initialPageParam: { afterCreatedAt: null, afterId: null, afterValue: null },
    getNextPageParam: (lastPage) => {
      if (!lastPage.pagination.has_more) {
        return undefined;
      }

      return {
        afterCreatedAt: lastPage.pagination.next_after_created_at,
        afterId: lastPage.pagination.next_after_id,
        afterValue: lastPage.pagination.next_after_value ?? null,
      };
    },
  });

  const tagsQuery = useQuery({
    queryKey: ["tags"],
    queryFn: fetchTags,
  });

  const categoriesQuery = useQuery({
    queryKey: ["categories"],
    queryFn: fetchCategories,
  });

  const items = itemsQuery.data?.pages.flatMap((page: PaginatedItemsResponse) => page.data) ?? [];

  return {
    items,
    mediaCards: items.map((item) => mapItemToMediaCard(item, options)),
    hasNextPage: itemsQuery.hasNextPage,
    fetchNextPage: itemsQuery.fetchNextPage,
    isFetchingNextPage: itemsQuery.isFetchingNextPage,
    isLoading: itemsQuery.isLoading || tagsQuery.isLoading || categoriesQuery.isLoading,
    isError: itemsQuery.isError || tagsQuery.isError || categoriesQuery.isError,
    tags: tagsQuery.data ?? [],
    categories: categoriesQuery.data ?? [],
  };
}
