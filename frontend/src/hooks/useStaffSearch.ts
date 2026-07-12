import { useQuery } from "@tanstack/react-query";
import { apiFetch } from "@/lib/apiClient";

type ApiOk<T> = { success: boolean; data: T };

export type StaffSearchResult = {
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

async function searchStaffByName(q: string): Promise<StaffSearchResult[]> {
  const params = new URLSearchParams({ q });
  const json = await parseJson<ApiOk<StaffSearchResult[]>>(await apiFetch(`/staff?${params.toString()}`));
  return json.data;
}

export async function createStaff(input: { name: string; externalId?: string; imageUrl?: string }): Promise<StaffSearchResult> {
  const json = await parseJson<ApiOk<StaffSearchResult>>(
    await apiFetch("/staff", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name: input.name, external_id: input.externalId || undefined, image_url: input.imageUrl || undefined }),
    }),
  );
  return json.data;
}

export function useStaffSearch(q: string) {
  const query = useQuery({
    queryKey: ["staff-search", q],
    queryFn: () => searchStaffByName(q),
    enabled: q.trim().length > 0,
  });

  return {
    results: query.data ?? [],
    isLoading: query.isLoading,
    isError: query.isError,
  };
}
