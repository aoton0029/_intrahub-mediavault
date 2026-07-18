import { useInfiniteQuery, useQuery, type InfiniteData } from "@tanstack/react-query";
import { apiFetch } from "@/lib/apiClient";
import {
  buildQueryString,
  mapItemToMediaCard,
  type MediaType,
  type PaginatedItemsResponse,
} from "@/hooks/useMediaListData";

export type YearDateField = "release" | "consumed" | "any";

export type YearlyMediaFilters = {
  dateField: YearDateField;
  mediaType?: MediaType;
  sort?: string;
};

export type MediaTypeCount = {
  media_type: MediaType;
  count: number;
};

export type YearCount = {
  year: number;
  count: number;
  /** メディア種別ごとの件数内訳（count降順、0件の種別は含まない） */
  media_types: MediaTypeCount[];
};

type YearsResponse = {
  success: boolean;
  data: YearCount[];
};

type Cursor = {
  afterCreatedAt: string | null;
  afterId: string | null;
  afterValue: string | null;
};

const YEAR_PAGE_LIMIT = 40;

async function fetchYears(filters: YearlyMediaFilters) {
  const response = await apiFetch(
    `/items/years${buildQueryString({
      date_field: filters.dateField,
      media_type: filters.mediaType,
    })}`,
  );

  if (!response.ok) {
    throw new Error(`Failed to fetch item years: ${response.status}`);
  }

  const json = (await response.json()) as YearsResponse;
  return json.data;
}

async function fetchYearItemsPage(filters: YearlyMediaFilters, year: number, pageParam: Cursor) {
  const response = await apiFetch(
    `/items${buildQueryString({
      year,
      date_field: filters.dateField,
      media_type: filters.mediaType,
      sort: filters.sort,
      limit: YEAR_PAGE_LIMIT,
      after_created_at: pageParam.afterCreatedAt,
      after_id: pageParam.afterId,
      after_value: pageParam.afterValue,
    })}`,
  );

  if (!response.ok) {
    throw new Error(`Failed to fetch items for year ${year}: ${response.status}`);
  }

  return (await response.json()) as PaginatedItemsResponse;
}

/** 年別コレクションページ用: 年（降順）と件数の一覧を取得する */
export function useYearlyMediaData(filters: YearlyMediaFilters) {
  const yearsQuery = useQuery({
    queryKey: ["item-years", filters],
    queryFn: () => fetchYears(filters),
  });

  return {
    years: yearsQuery.data ?? [],
    isLoading: yearsQuery.isLoading,
    isError: yearsQuery.isError,
  };
}

/** 指定年のアイテム一覧を取得する（年セクション単位で呼び出す） */
export function useYearItems(filters: YearlyMediaFilters, year: number) {
  const itemsQuery = useInfiniteQuery<
    PaginatedItemsResponse,
    Error,
    InfiniteData<PaginatedItemsResponse, Cursor>,
    [string, YearlyMediaFilters, number],
    Cursor
  >({
    queryKey: ["media-list-year", filters, year],
    queryFn: ({ pageParam }) => fetchYearItemsPage(filters, year, pageParam),
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

  const items = itemsQuery.data?.pages.flatMap((page) => page.data) ?? [];

  return {
    mediaCards: items.map((item) => mapItemToMediaCard(item)),
    hasNextPage: itemsQuery.hasNextPage,
    fetchNextPage: itemsQuery.fetchNextPage,
    isFetchingNextPage: itemsQuery.isFetchingNextPage,
    isLoading: itemsQuery.isLoading,
    isError: itemsQuery.isError,
  };
}
