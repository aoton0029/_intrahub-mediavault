import { useQuery } from "@tanstack/react-query";
import { apiFetch } from "@/lib/apiClient";

type ApiOk<T> = { success: boolean; data: T };

export type CastSearchResult = {
  id: string;
  external_id: string | null;
  name: string;
  image_url: string | null;
  created_at: string;
  linked_item_count: number;
};

async function parseJson<T>(response: Response) {
  if (!response.ok) {
    throw new Error(`Request failed: ${response.status}`);
  }
  return (await response.json()) as T;
}

async function searchCastByName(q: string): Promise<CastSearchResult[]> {
  const params = new URLSearchParams({ q });
  const json = await parseJson<ApiOk<CastSearchResult[]>>(await apiFetch(`/cast?${params.toString()}`));
  return json.data;
}

export async function createCast(input: { name: string; externalId?: string; imageUrl?: string }): Promise<CastSearchResult> {
  const json = await parseJson<ApiOk<CastSearchResult>>(
    await apiFetch("/cast", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name: input.name, external_id: input.externalId || undefined, image_url: input.imageUrl || undefined }),
    }),
  );
  return json.data;
}

export function useCastSearch(q: string) {
  const query = useQuery({
    queryKey: ["cast-search", q],
    queryFn: () => searchCastByName(q),
    enabled: q.trim().length > 0,
  });

  return {
    results: query.data ?? [],
    isLoading: query.isLoading,
    isError: query.isError,
  };
}
