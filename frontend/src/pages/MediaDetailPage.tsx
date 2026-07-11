import { useMemo } from "react";
import { Link, useParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { usePageChrome } from "@/components/layout/usePageChrome";
import { EmptyState } from "@/components/shared";
import { AnimeDetailPage } from "./AnimeDetailPage";
import { MovieDetailPage } from "./MovieDetailPage";
import { apiFetch } from "@/lib/apiClient";

type MediaType = "anime" | "movie" | "drama" | "manga" | "novel" | "game" | "academic_book" | "paper";
type ItemDetail = {
  id: string;
  media_type: MediaType;
};
type ApiOk<T> = { success: boolean; data: T };

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

  return (await response.json()) as T;
}

async function fetchItem(id: string) {
  const json = await parseJson<ApiOk<ItemDetail>>(await apiFetch(`/items/${id}`));
  return json.data;
}

export function MediaDetailPage() {
  const { id } = useParams();
  const detailQuery = useQuery({
    queryKey: ["media-detail-dispatch", id],
    enabled: Boolean(id),
    queryFn: () => fetchItem(id!),
  });

  const mediaType = detailQuery.data?.media_type;
  const pageChrome = useMemo(() => ({
    breadcrumbs: mediaType
      ? [
        { label: "一般メディア", to: "/media" },
        { label: mediaType === "movie" ? "映画" : mediaType === "anime" ? "アニメ" : "詳細" },
      ]
      : [{ label: "一般メディア", to: "/media" }],
    actions: mediaType === "movie" && id ? <Link className="btn btn-accent" to={`/media/${id}/edit`}>編集する</Link> : undefined,
  }), [id, mediaType]);
  usePageChrome(pageChrome);

  if (!id) {
    return <EmptyState title="作品IDが見つかりません" description="URL を確認してからもう一度開いてください。" />;
  }

  if (detailQuery.isLoading) {
    return <EmptyState title="読み込み中です" description="詳細画面を準備しています。" />;
  }

  if (detailQuery.isError || !detailQuery.data) {
    return <EmptyState title="詳細を読み込めませんでした" description="時間をおいて再読み込みしてください。" />;
  }

  if (mediaType === "anime") {
    return <AnimeDetailPage />;
  }

  if (mediaType === "movie") {
    return <MovieDetailPage />;
  }

  return (
    <EmptyState
      title="未対応の種別です"
      description="この種別の詳細画面は未対応です。今後の対応をお待ちください。"
    />
  );
}
