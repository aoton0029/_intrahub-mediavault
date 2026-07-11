import { useQuery } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { apiFetch } from "@/lib/apiClient";
import type { MediaCardProps } from "@/components/shared";
import type { HomeStats } from "@/components/home/StatGrid";

type ItemStatus = "not_started" | "in_progress" | "done" | "completed" | string;
type MediaType = "anime" | "movie" | "drama" | "manga" | "novel" | "game" | "academic_book" | "paper";

type ItemWithRefs = {
  id: string;
  media_type: MediaType;
  title: string;
  status: ItemStatus;
  rating: number | null;
  is_favorite: boolean;
};

type PaginatedItemsResponse = {
  success: boolean;
  data: ItemWithRefs[];
  pagination: {
    has_more: boolean;
    next_after_created_at: string | null;
    next_after_id: string | null;
  };
};

export type HomeData = {
  stats: HomeStats;
  recentItems: MediaCardProps[];
  inProgressItems: MediaCardProps[];
};

const MEDIA_TYPE_LABELS: Record<MediaType, string> = {
  anime: "アニメ",
  movie: "映画",
  drama: "ドラマ",
  manga: "漫画",
  novel: "小説",
  game: "ゲーム",
  academic_book: "専門書",
  paper: "論文",
};

function buildQueryString(params: Record<string, string | number | undefined | null>) {
  const searchParams = new URLSearchParams();

  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined && value !== null) {
      searchParams.set(key, String(value));
    }
  }

  const queryString = searchParams.toString();
  return queryString ? `?${queryString}` : "";
}

async function fetchItemsPage(params: Record<string, string | number | undefined | null>) {
  const response = await apiFetch(`/items${buildQueryString(params)}`);

  if (!response.ok) {
    throw new Error(`Failed to fetch items: ${response.status}`);
  }

  const json = (await response.json()) as PaginatedItemsResponse;
  return json;
}

async function fetchAllItems(params: Record<string, string | number | undefined | null> = {}) {
  const allItems: ItemWithRefs[] = [];
  let afterCreatedAt: string | null = null;
  let afterId: string | null = null;

  do {
    const page = await fetchItemsPage({
      ...params,
      limit: 100,
      after_created_at: afterCreatedAt,
      after_id: afterId,
    });

    allItems.push(...page.data);
    afterCreatedAt = page.pagination.has_more ? page.pagination.next_after_created_at : null;
    afterId = page.pagination.has_more ? page.pagination.next_after_id : null;
  } while (afterCreatedAt && afterId);

  return allItems;
}

function toStatusLabel(item: ItemWithRefs) {
  if (item.status !== "in_progress") {
    return undefined;
  }

  switch (item.media_type) {
    case "anime":
    case "movie":
    case "drama":
      return "視聴中";
    case "game":
      return "プレイ中";
    case "manga":
    case "novel":
    case "academic_book":
    case "paper":
      return "読書中";
  }
}

function createStatusMeta(statusLabel: string): ReactNode {
  return (
    <div className="prop-taglist" style={{ marginTop: 6 }}>
      <span className="tag-pill" style={{ color: "var(--status-progress)" }}>
        {statusLabel}
      </span>
    </div>
  );
}

export function mapItemToMediaCard(item: ItemWithRefs, options?: { includeStatus?: boolean }): MediaCardProps {
  const statusLabel = options?.includeStatus ? toStatusLabel(item) : undefined;

  return {
    title: item.title,
    badge: MEDIA_TYPE_LABELS[item.media_type],
    rating: item.rating ?? undefined,
    favorite: item.is_favorite,
    href: `/media/${item.id}`,
    meta: statusLabel ? createStatusMeta(statusLabel) : undefined,
  };
}

async function loadHomeData(): Promise<HomeData> {
  const [allItems, recentItemsPage, inProgressItemsPage] = await Promise.all([
    fetchAllItems(),
    fetchItemsPage({ limit: 6 }),
    fetchItemsPage({ status: "in_progress", limit: 6 }),
  ]);

  const stats: HomeStats = {
    totalCount: allItems.length,
    inProgressCount: allItems.filter((item) => item.status === "in_progress").length,
    doneCount: allItems.filter((item) => item.status === "done" || item.status === "completed").length,
    favoriteCount: allItems.filter((item) => item.is_favorite).length,
  };

  return {
    stats,
    recentItems: recentItemsPage.data.map((item) => mapItemToMediaCard(item)),
    inProgressItems: inProgressItemsPage.data.map((item) => mapItemToMediaCard(item, { includeStatus: true })),
  };
}

export function useHomeData() {
  return useQuery({
    queryKey: ["home-data"],
    queryFn: loadHomeData,
  });
}
