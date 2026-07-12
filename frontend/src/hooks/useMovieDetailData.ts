import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { ImageItem, StaffMember, StreamingLinkItem } from "@/components/detail";
import type { PropertyItem, RelatedWork, ResourceTabKey, TagListItem } from "@/components/shared";
import { STREAMING_PLATFORM_LABELS } from "./useAnimeDetailData";
import { apiFetch } from "@/lib/apiClient";

type ApiOk<T> = { success: boolean; data: T };
type ItemStatus = "not_started" | "in_progress" | "completed";
type SourceType = "manual" | "api";
type MediaType = "anime" | "movie" | "drama" | "manga" | "novel" | "game" | "academic_book" | "paper";
type StreamingPlatform = "netflix" | "amazon_prime" | "disney_plus" | "dmm_tv" | "apple_tv";
type TagRef = TagListItem;
type CategoryRef = TagListItem;
type Mylist = { id: string; name: string; created_at: string };
type Tag = { id: string; name: string };
type Category = { id: string; name: string };
type MovieDetail = {
  runtime_minutes: number;
  original_language: string;
  production_companies: string[];
  collection?: string;
  genres: string[];
  rating: number | null;
  vote_count: number;
};
type CalibreLink = { file_id: string; calibre_book_id: number };
type ItemStreamingLink = { id: string; item_id: string; platform: StreamingPlatform; url: string; created_at: string };
type ItemImageRecord = { id: string; item_id: string; url: string; created_at: string };
type ItemLink = { id: string; item_id: string; url: string; label: string; created_at: string };
type ItemFile = {
  id: string;
  item_id: string;
  path: string;
  label: string | null;
  file_type: "pdf" | "image" | "other";
  calibre_book_id: number | null;
  created_at: string;
};
type ItemTrailer = { id: string; item_id: string; url: string; label: string | null; created_at: string };
type ItemStaff = {
  id: string;
  item_id: string;
  staff_id: string;
  role: string;
  character_name: string | null;
  staff_name: string;
};
type ItemCast = {
  id: string;
  item_id: string;
  cast_id: string;
  character_name: string | null;
  cast_name: string;
};
type ItemRelation = {
  id: string;
  item_id: string;
  related_item_id: string;
  relation_type: "reference" | "dlc";
  created_at: string;
  related_item_title?: string | null;
  related_title?: string | null;
  title?: string | null;
};
type ItemDetail = {
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
  source: SourceType;
  external_id: string | null;
  created_at: string;
  updated_at: string;
  detail: MovieDetail | null;
  tags: TagRef[];
  categories: CategoryRef[];
  calibre_links: CalibreLink[];
  streaming_links: ItemStreamingLink[];
};
type MovieDetailBundle = {
  item: ItemDetail;
  staff: ItemStaff[];
  cast: ItemCast[];
  relations: ItemRelation[];
  streamingLinks: ItemStreamingLink[];
  images: ItemImageRecord[];
  mylists: Mylist[];
  links: ItemLink[];
  files: ItemFile[];
  trailers: ItemTrailer[];
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

function formatValue(value: string | number | null | undefined, fallback = "未登録") {
  if (value === null || value === undefined || value === "") {
    return fallback;
  }
  return String(value);
}

function mapPropertyItems(detail: MovieDetail | null): PropertyItem[] {
  return [
    { key: "runtime_minutes", label: "上映時間", value: detail?.runtime_minutes ? `${detail.runtime_minutes}分` : "未登録", muted: !detail?.runtime_minutes },
    { key: "original_language", label: "原語", value: formatValue(detail?.original_language), muted: !detail?.original_language },
    {
      key: "production_companies",
      label: "制作会社",
      value: detail?.production_companies?.length ? detail.production_companies.join(", ") : "未登録",
      muted: !detail?.production_companies?.length,
    },
    { key: "collection", label: "コレクション", value: formatValue(detail?.collection), muted: !detail?.collection },
    { key: "genres", label: "ジャンル", value: detail?.genres?.length ? detail.genres.join("・") : "未登録", muted: !detail?.genres?.length },
    { key: "vote_count", label: "評価人数", value: detail?.vote_count !== undefined && detail?.vote_count !== null ? `${detail.vote_count}人` : "未登録", muted: detail?.vote_count === undefined || detail?.vote_count === null },
  ];
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

async function fetchMovieDetailBundle(id: string): Promise<MovieDetailBundle> {
  const [item, staff, cast, relations, streamingLinks, images, mylists, links, files, trailers] = await Promise.all([
    fetchApi<ItemDetail>(`/items/${id}`),
    fetchApi<ItemStaff[]>(`/items/${id}/staff`),
    fetchApi<ItemCast[]>(`/items/${id}/cast`),
    fetchApi<ItemRelation[]>(`/items/${id}/relations`),
    fetchApi<ItemStreamingLink[]>(`/items/${id}/streaming-links`),
    fetchApi<ItemImageRecord[]>(`/items/${id}/images`),
    fetchApi<Mylist[]>(`/items/${id}/mylists`),
    fetchApi<ItemLink[]>(`/items/${id}/links`),
    fetchApi<ItemFile[]>(`/items/${id}/files`),
    fetchApi<ItemTrailer[]>(`/items/${id}/trailers`),
  ]);

  return { item, staff, cast, relations, streamingLinks, images, mylists, links, files, trailers };
}

function buildActionLabel(item: ItemDetail) {
  const labels = [item.original_title, item.release_date?.slice(0, 4) ? `${item.release_date.slice(0, 4)}年` : null].filter(Boolean);
  return labels.join(" ・ ");
}

export function useMovieDetailData(id: string | undefined) {
  const queryClient = useQueryClient();
  const queryKey = ["movie-detail", id] as const;

  const detailQuery = useQuery({
    queryKey,
    enabled: Boolean(id),
    queryFn: () => fetchMovieDetailBundle(id!),
  });

  const invalidate = async () => {
    await queryClient.invalidateQueries({ queryKey });
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

  const staffAddMutation = useMutation({
    mutationFn: async ({ staffId, role, characterName }: { staffId: string; role: string; characterName?: string }) => {
      await fetchApi(`/items/${id}/staff`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ staff_id: staffId, role, character_name: characterName || undefined }),
      });
    },
    onSuccess: invalidate,
  });

  const staffRemoveMutation = useMutation({
    mutationFn: async (itemStaffId: string) => {
      await parseJson(await apiFetch(`/items/${id}/staff/${itemStaffId}`, { method: "DELETE" }));
    },
    onSuccess: invalidate,
  });

  const castAddMutation = useMutation({
    mutationFn: async ({ castId, characterName }: { castId: string; characterName?: string }) => {
      await fetchApi(`/items/${id}/cast`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ cast_id: castId, character_name: characterName || undefined }),
      });
    },
    onSuccess: invalidate,
  });

  const castRemoveMutation = useMutation({
    mutationFn: async (itemCastId: string) => {
      await parseJson(await apiFetch(`/items/${id}/cast/${itemCastId}`, { method: "DELETE" }));
    },
    onSuccess: invalidate,
  });

  const relationAddMutation = useMutation({
    mutationFn: async ({ relatedItemId, relationType }: { relatedItemId: string; relationType: "reference" | "dlc" }) => {
      await fetchApi("/item-relations", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ item_id: id, related_item_id: relatedItemId, relation_type: relationType }),
      });
    },
    onSuccess: invalidate,
  });

  const relationRemoveMutation = useMutation({
    mutationFn: async (relationId: string) => {
      await parseJson(await apiFetch(`/item-relations/${relationId}`, { method: "DELETE" }));
    },
    onSuccess: invalidate,
  });

  const streamingAddMutation = useMutation({
    mutationFn: async ({ platform, url }: { platform: StreamingPlatform; url: string }) => {
      await fetchApi(`/items/${id}/streaming-links`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ platform, url }),
      });
    },
    onSuccess: invalidate,
  });

  const streamingRemoveMutation = useMutation({
    mutationFn: async (linkId: string) => {
      await parseJson(await apiFetch(`/items/${id}/streaming-links/${linkId}`, { method: "DELETE" }));
    },
    onSuccess: invalidate,
  });

  const imageAddMutation = useMutation({
    mutationFn: async (url: string) => {
      await fetchApi(`/items/${id}/images`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ url }),
      });
    },
    onSuccess: invalidate,
  });

  const imageRemoveMutation = useMutation({
    mutationFn: async (imageId: string) => {
      await parseJson(await apiFetch(`/items/${id}/images/${imageId}`, { method: "DELETE" }));
    },
    onSuccess: invalidate,
  });

  const linkAddMutation = useMutation({
    mutationFn: async ({ label, url }: { label: string; url: string }) => {
      await fetchApi(`/items/${id}/links`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ label, url }),
      });
    },
    onSuccess: invalidate,
  });

  const linkRemoveMutation = useMutation({
    mutationFn: async (linkId: string) => {
      await parseJson(await apiFetch(`/items/${id}/links/${linkId}`, { method: "DELETE" }));
    },
    onSuccess: invalidate,
  });

  const fileAddMutation = useMutation({
    mutationFn: async ({ path, label, fileType }: { path: string; label?: string; fileType: ItemFile["file_type"] }) => {
      await fetchApi(`/items/${id}/files`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ path, label, file_type: fileType }),
      });
    },
    onSuccess: invalidate,
  });

  const fileRemoveMutation = useMutation({
    mutationFn: async (fileId: string) => {
      await parseJson(await apiFetch(`/items/${id}/files/${fileId}`, { method: "DELETE" }));
    },
    onSuccess: invalidate,
  });

  const trailerAddMutation = useMutation({
    mutationFn: async ({ url, label }: { url: string; label?: string }) => {
      await fetchApi(`/items/${id}/trailers`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ url, label }),
      });
    },
    onSuccess: invalidate,
  });

  const trailerRemoveMutation = useMutation({
    mutationFn: async (trailerId: string) => {
      await parseJson(await apiFetch(`/items/${id}/trailers/${trailerId}`, { method: "DELETE" }));
    },
    onSuccess: invalidate,
  });

  const calibreLinkMutation = useMutation({
    mutationFn: async ({ fileId, calibreBookId }: { fileId: string; calibreBookId: number }) => {
      await fetchApi(`/items/${id}/files/${fileId}/calibre-link`, {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ calibre_book_id: calibreBookId }),
      });
    },
    onSuccess: invalidate,
  });

  const deleteItemMutation = useMutation({
    mutationFn: async () => {
      await parseJson(await apiFetch(`/items/${id}`, { method: "DELETE" }));
    },
    onSuccess: () => {
      queryClient.removeQueries({ queryKey });
    },
  });

  const bundle = detailQuery.data;
  const item = bundle?.item;
  const propertyItems = mapPropertyItems(item?.detail ?? null);
  const staffList: StaffMember[] = bundle?.staff.map((entry) => ({
    id: entry.id,
    label: entry.staff_name,
    sub: entry.character_name ? `${entry.role}(${entry.character_name}役)` : entry.role,
  })) ?? [];
  const castList: StaffMember[] = bundle?.cast.map((entry) => ({
    id: entry.id,
    label: entry.cast_name,
    sub: entry.character_name ? `${entry.character_name}役` : "役名未登録",
  })) ?? [];
  const relatedWorks: RelatedWork[] = bundle?.relations.map((relation) => ({
    id: relation.id,
    relatedItemId: relation.related_item_id,
    title: relation.related_item_title ?? relation.related_title ?? relation.title ?? relation.related_item_id,
    relation: relation.relation_type,
  })) ?? [];
  const streaming: StreamingLinkItem[] = bundle?.streamingLinks.map((link) => ({
    id: link.id,
    label: STREAMING_PLATFORM_LABELS[link.platform],
    sub: link.url,
    platform: link.platform,
  })) ?? [];
  const images: ImageItem[] = bundle?.images.map((image) => ({
    id: image.id,
    url: image.url,
    isCover: image.url === item?.cover_image_url,
  })) ?? [];
  const resourceTabs: Partial<Record<ResourceTabKey, { id: string; label: string; detail: string }[]>> = {
    links: bundle?.links.map((link) => ({ id: link.id, label: link.label, detail: link.url })) ?? [],
    files: bundle?.files.map((file) => ({
      id: file.id,
      label: file.label ?? file.path,
      detail: file.calibre_book_id ? `${file.file_type} / Calibre #${file.calibre_book_id}` : file.file_type,
    })) ?? [],
    trailers: bundle?.trailers.map((trailer) => ({
      id: trailer.id,
      label: trailer.label ?? "トレーラー",
      detail: trailer.url,
    })) ?? [],
  };

  return {
    item,
    propertyItems,
    staffList,
    castList,
    relatedWorks,
    streaming,
    images,
    resourceTabs,
    tags: item?.tags ?? [],
    categories: item?.categories ?? [],
    mylists: bundle?.mylists ?? [],
    files: bundle?.files ?? [],
    overview: item?.description ?? "",
    actionLabel: item ? buildActionLabel(item) : "",
    isLoading: detailQuery.isLoading,
    isError: detailQuery.isError,
    refetch: detailQuery.refetch,
    updateStatus: (status: ItemStatus) => statusMutation.mutateAsync(status),
    updateRating: (rating: number) => patchItemMutation.mutateAsync({ rating }),
    updateFavorite: (isFavorite: boolean) => patchItemMutation.mutateAsync({ is_favorite: isFavorite }),
    updateConsumedDate: (date: string | null) => patchItemMutation.mutateAsync({ consumed_date: date }),
    updateDescription: (description: string) => patchItemMutation.mutateAsync({ description }),
    addTag: (name: string) => tagAddMutation.mutateAsync(name),
    removeTag: (tagId: string) => tagRemoveMutation.mutateAsync(tagId),
    addCategory: (name: string) => categoryAddMutation.mutateAsync(name),
    removeCategory: (categoryId: string) => categoryRemoveMutation.mutateAsync(categoryId),
    removeMylist: (mylistId: string) => mylistRemoveMutation.mutateAsync(mylistId),
    addStaff: (staffId: string, role: string, characterName?: string) => staffAddMutation.mutateAsync({ staffId, role, characterName }),
    removeStaff: (itemStaffId: string) => staffRemoveMutation.mutateAsync(itemStaffId),
    addCast: (castId: string, characterName?: string) => castAddMutation.mutateAsync({ castId, characterName }),
    removeCast: (itemCastId: string) => castRemoveMutation.mutateAsync(itemCastId),
    addRelation: (relatedItemId: string, relationType: "reference" | "dlc") => relationAddMutation.mutateAsync({ relatedItemId, relationType }),
    removeRelation: (relationId: string) => relationRemoveMutation.mutateAsync(relationId),
    addStreamingLink: (platform: StreamingPlatform, url: string) => streamingAddMutation.mutateAsync({ platform, url }),
    removeStreamingLink: (linkId: string) => streamingRemoveMutation.mutateAsync(linkId),
    addImage: (url: string) => imageAddMutation.mutateAsync(url),
    removeImage: (imageId: string) => imageRemoveMutation.mutateAsync(imageId),
    setCoverImage: (url: string) => patchItemMutation.mutateAsync({ cover_image_url: url }),
    addLink: (label: string, url: string) => linkAddMutation.mutateAsync({ label, url }),
    removeLink: (linkId: string) => linkRemoveMutation.mutateAsync(linkId),
    addFile: (path: string, label: string | undefined, fileType: ItemFile["file_type"]) => fileAddMutation.mutateAsync({ path, label, fileType }),
    removeFile: (fileId: string) => fileRemoveMutation.mutateAsync(fileId),
    addTrailer: (url: string, label?: string) => trailerAddMutation.mutateAsync({ url, label }),
    removeTrailer: (trailerId: string) => trailerRemoveMutation.mutateAsync(trailerId),
    linkCalibre: (fileId: string, calibreBookId: number) => calibreLinkMutation.mutateAsync({ fileId, calibreBookId }),
    deleteItem: () => deleteItemMutation.mutateAsync(),
  };
}

export type UseMovieDetailDataResult = ReturnType<typeof useMovieDetailData>;
export { mapPropertyItems };
