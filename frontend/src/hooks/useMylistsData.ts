import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { apiFetch } from "@/lib/apiClient";

type ApiOk<T> = { success: boolean; data: T };

export type MylistSummary = {
  id: string;
  name: string;
  created_at: string;
  item_count: number;
  cover_urls: string[];
};

async function parseJson<T>(response: Response) {
  if (!response.ok) {
    let message = `Request failed: ${response.status}`;
    try {
      const errorJson = (await response.json()) as { message?: string };
      if (errorJson.message) {
        message = errorJson.message;
      }
    } catch {
      // Ignore non-JSON errors.
    }
    throw new Error(message);
  }

  if (response.status === 204) {
    return null as T;
  }

  return (await response.json()) as T;
}

async function fetchApi<T>(input: RequestInfo, init?: RequestInit) {
  const json = await parseJson<ApiOk<T>>(await apiFetch(input, init));
  return json.data;
}

export function useMylistsData() {
  const queryClient = useQueryClient();
  const queryKey = ["mylists"] as const;

  const mylistsQuery = useQuery({
    queryKey,
    queryFn: () => fetchApi<MylistSummary[]>("/mylists"),
  });

  const invalidate = () => queryClient.invalidateQueries({ queryKey });

  const createMutation = useMutation({
    mutationFn: (name: string) =>
      fetchApi<MylistSummary>("/mylists", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name }),
      }),
    onSuccess: invalidate,
  });

  const renameMutation = useMutation({
    mutationFn: ({ id, name }: { id: string; name: string }) =>
      fetchApi(`/mylists/${id}`, {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name }),
      }),
    onSuccess: invalidate,
  });

  const deleteMutation = useMutation({
    mutationFn: async (id: string) => parseJson(await apiFetch(`/mylists/${id}`, { method: "DELETE" })),
    onSuccess: invalidate,
  });

  return {
    mylists: mylistsQuery.data ?? [],
    isLoading: mylistsQuery.isLoading,
    isError: mylistsQuery.isError,
    createMylist: (name: string) => createMutation.mutateAsync(name),
    renameMylist: (id: string, name: string) => renameMutation.mutateAsync({ id, name }),
    deleteMylist: (id: string) => deleteMutation.mutateAsync(id),
  };
}
