import { useQuery } from "@tanstack/react-query";
import { apiFetch } from "@/lib/apiClient";

type ApiOk<T> = { success: boolean; data: T };
type MediaType = "anime" | "movie" | "drama" | "manga" | "novel" | "game" | "academic_book" | "paper";

export type ItemSearchResult = {
  id: string;
  title: string;
  media_type: MediaType;
  release_date: string | null;
  cover_image_url: string | null;
};

async function parseJson<T>(response: Response) {
  if (!response.ok) {
    throw new Error(`Request failed: ${response.status}`);
  }
  return (await response.json()) as T;
}

async function searchItemsByTitle(title: string): Promise<ItemSearchResult[]> {
  const params = new URLSearchParams({ title, limit: "20" });
  const json = await parseJson<ApiOk<ItemSearchResult[]>>(await apiFetch(`/items?${params.toString()}`));
  return json.data;
}

export function useItemRelationSearch(title: string) {
  const query = useQuery({
    queryKey: ["item-relation-search", title],
    queryFn: () => searchItemsByTitle(title),
    enabled: title.trim().length > 0,
  });

  return {
    results: query.data ?? [],
    isLoading: query.isLoading,
    isError: query.isError,
  };
}
