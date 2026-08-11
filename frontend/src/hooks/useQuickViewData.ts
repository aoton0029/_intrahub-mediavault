import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { apiFetch } from "@/lib/apiClient";

type ApiOk<T> = { success: boolean; data: T };
type ItemStatus = "not_started" | "in_progress" | "completed";
type MediaType = "anime" | "movie" | "drama" | "manga" | "novel" | "game" | "academic_book" | "paper";

type TagRef = { id: string; name: string };
type CategoryRef = { id: string; name: string };
type Mylist = { id: string; name: string; created_at: string };
type Tag = { id: string; name: string };
type Category = { id: string; name: string };

export type QuickViewItem = {
  id: string;
  media_type: MediaType;
  title: string;
  original_title: string | null;
  description: string | null;
  cover_image_url: string | null;
  release_date: string | null;
  homepage_url: string | null;
  status: ItemStatus;
  consumed_date: string | null;
  rating: number | null;
  is_favorite: boolean;
  source: "manual" | "api";
  updated_at: string;
  tags: TagRef[];
  categories: CategoryRef[];
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

async function createTag(name: string) {
  return fetchApi<Tag>("/tags", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ name }),
  });
}

async function createCategory(name: string) {
  return fetchApi<Category>("/categories", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ name }),
  });
}

/** Lightweight item detail + mutations for the media list quick-view sheet (no staff/cast/episode bundle). */
export function useQuickViewData(id: string | undefined) {
  const queryClient = useQueryClient();
  const queryKey = ["quick-view", id] as const;

  const itemQuery = useQuery({
    queryKey,
    enabled: Boolean(id),
    queryFn: () => fetchApi<QuickViewItem>(`/items/${id}`),
  });

  const mylistsQuery = useQuery({
    queryKey: ["quick-view-mylists", id],
    enabled: Boolean(id),
    queryFn: () => fetchApi<Mylist[]>(`/items/${id}/mylists`),
  });

  const invalidate = async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey }),
      queryClient.invalidateQueries({ queryKey: ["quick-view-mylists", id] }),
      queryClient.invalidateQueries({ queryKey: ["media-list"] }),
    ]);
  };

  const statusMutation = useMutation({
    mutationFn: async (status: ItemStatus) => {
      await fetchApi(`/items/${id}/status`, {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ status }),
      });
    },
    onSuccess: invalidate,
  });

  const patchItemMutation = useMutation({
    mutationFn: async (payload: Record<string, unknown>) => {
      await fetchApi(`/items/${id}`, {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });
    },
    onSuccess: invalidate,
  });

  const tagAddMutation = useMutation({
    mutationFn: async (name: string) => {
      const tag = await createTag(name);
      await apiFetch(`/items/${id}/tags/${tag.id}`, { method: "POST" }).then(parseJson);
    },
    onSuccess: invalidate,
  });

  const tagRemoveMutation = useMutation({
    mutationFn: async (tagId: string) => {
      await parseJson(await apiFetch(`/items/${id}/tags/${tagId}`, { method: "DELETE" }));
    },
    onSuccess: invalidate,
  });

  const categoryAddMutation = useMutation({
    mutationFn: async (name: string) => {
      const category = await createCategory(name);
      await apiFetch(`/items/${id}/categories/${category.id}`, { method: "POST" }).then(parseJson);
    },
    onSuccess: invalidate,
  });

  const categoryRemoveMutation = useMutation({
    mutationFn: async (categoryId: string) => {
      await parseJson(await apiFetch(`/items/${id}/categories/${categoryId}`, { method: "DELETE" }));
    },
    onSuccess: invalidate,
  });

  const mylistRemoveMutation = useMutation({
    mutationFn: async (mylistId: string) => {
      await parseJson(await apiFetch(`/mylists/${mylistId}/items/${id}`, { method: "DELETE" }));
    },
    onSuccess: invalidate,
  });

  const deleteItemMutation = useMutation({
    mutationFn: async () => {
      await parseJson(await apiFetch(`/items/${id}`, { method: "DELETE" }));
    },
    onSuccess: async () => {
      queryClient.removeQueries({ queryKey });
      await queryClient.invalidateQueries({ queryKey: ["media-list"] });
    },
  });

  return {
    item: itemQuery.data,
    mylists: mylistsQuery.data ?? [],
    isLoading: itemQuery.isLoading,
    isError: itemQuery.isError,
    updateStatus: (status: ItemStatus) => statusMutation.mutateAsync(status),
    updateRating: (rating: number) => patchItemMutation.mutateAsync({ rating }),
    updateFavorite: (isFavorite: boolean) => patchItemMutation.mutateAsync({ is_favorite: isFavorite }),
    addTag: (name: string) => tagAddMutation.mutateAsync(name),
    removeTag: (tagId: string) => tagRemoveMutation.mutateAsync(tagId),
    addCategory: (name: string) => categoryAddMutation.mutateAsync(name),
    removeCategory: (categoryId: string) => categoryRemoveMutation.mutateAsync(categoryId),
    removeMylist: (mylistId: string) => mylistRemoveMutation.mutateAsync(mylistId),
    deleteItem: () => deleteItemMutation.mutateAsync(),
  };
}

/** Status/favorite/delete mutations usable directly from the media list grid or table row menus. */
export function useMediaItemActions() {
  const queryClient = useQueryClient();

  const invalidateList = () => queryClient.invalidateQueries({ queryKey: ["media-list"] });

  const statusMutation = useMutation({
    mutationFn: async ({ id, status }: { id: string; status: ItemStatus }) => {
      await fetchApi(`/items/${id}/status`, {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ status }),
      });
    },
    onSuccess: invalidateList,
  });

  const favoriteMutation = useMutation({
    mutationFn: async ({ id, isFavorite }: { id: string; isFavorite: boolean }) => {
      await fetchApi(`/items/${id}`, {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ is_favorite: isFavorite }),
      });
    },
    onSuccess: invalidateList,
  });

  const deleteMutation = useMutation({
    mutationFn: async (id: string) => {
      await parseJson(await apiFetch(`/items/${id}`, { method: "DELETE" }));
    },
    onSuccess: invalidateList,
  });

  return {
    updateStatus: (id: string, status: ItemStatus) => statusMutation.mutateAsync({ id, status }),
    updateFavorite: (id: string, isFavorite: boolean) => favoriteMutation.mutateAsync({ id, isFavorite }),
    deleteItem: (id: string) => deleteMutation.mutateAsync(id),
  };
}
