import { useMemo } from "react";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryRouter, RouterProvider } from "react-router-dom";
import { MediaListPage } from "./MediaListPage";

function mockFetchByUrl(responses: Record<string, unknown>) {
  window.fetch = (async (input: RequestInfo | URL) => {
    const url = String(input);
    const payload = responses[url];
    if (!payload) {
      return new Response(JSON.stringify({ success: false, message: `Unmocked request: ${url}` }), { status: 404 });
    }
    return new Response(JSON.stringify(payload));
  }) as typeof window.fetch;
}

function neverResolvingFetch() {
  window.fetch = (() => new Promise(() => {})) as typeof window.fetch;
}

function makeItem(overrides: Record<string, unknown>) {
  return {
    id: "item-1",
    media_type: "anime",
    title: "作品",
    original_title: null,
    description: null,
    cover_image_url: null,
    release_date: "2025-04-05",
    homepage_url: null,
    status: "in_progress",
    consumed_date: null,
    rating: 4,
    is_favorite: false,
    source: "api",
    external_id: null,
    created_at: "2026-07-01T12:00:00",
    updated_at: "2026-07-01T12:00:00",
    tags: [],
    categories: [],
    ...overrides,
  };
}

function MediaListPageHarness({ initialEntry = "/media" }: { initialEntry?: string }) {
  const queryClient = useMemo(() => new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } }), []);
  const router = useMemo(
    () =>
      createMemoryRouter([{ path: "/media", element: <MediaListPage /> }], {
        initialEntries: [initialEntry],
      }),
    [initialEntry],
  );

  return (
    <QueryClientProvider client={queryClient}>
      <div className="content">
        <RouterProvider router={router} />
      </div>
    </QueryClientProvider>
  );
}

const meta: Meta<typeof MediaListPageHarness> = {
  title: "pages/MediaListPage",
  component: MediaListPageHarness,
};

export default meta;
type Story = StoryObj<typeof MediaListPageHarness>;

export const Default: Story = {
  render: () => {
    mockFetchByUrl({
      "/api/v1/items?limit=20": {
        success: true,
        data: [
          makeItem({
            id: "item-1",
            media_type: "anime",
            title: "星屑のシンフォニア",
            cover_image_url: "https://img.annict.com/anime-1.jpg",
            rating: 4.5,
            is_favorite: true,
            tags: [{ id: "tag-1", name: "神作画" }],
            categories: [{ id: "category-1", name: "2026年鑑賞予定" }],
          }),
          makeItem({
            id: "item-2",
            media_type: "movie",
            title: "深海のレクイエム",
            cover_image_url: "https://img.tmdb.org/movie-2.jpg",
            rating: 3.5,
            is_favorite: false,
          }),
          makeItem({
            id: "item-3",
            media_type: "manga",
            title: "月ノ森ダイアリー",
            cover_image_url: null,
            rating: null,
            status: "done",
          }),
          makeItem({
            id: "item-4",
            media_type: "game",
            title: "旅立ちのフィールド",
            cover_image_url: "https://cdn.akamai.steamstatic.com/game-4.jpg",
            rating: 5,
            is_favorite: true,
            status: "not_started",
          }),
        ],
        pagination: { limit: 20, has_more: false, next_after_created_at: null, next_after_id: null },
      },
      "/api/v1/tags": {
        success: true,
        data: [{ id: "tag-1", name: "神作画", item_count: 3 }],
      },
      "/api/v1/categories": {
        success: true,
        data: [{ id: "category-1", name: "2026年鑑賞予定", item_count: 2 }],
      },
    });

    return <MediaListPageHarness />;
  },
};

export const Empty: Story = {
  render: () => {
    mockFetchByUrl({
      "/api/v1/items?limit=20": {
        success: true,
        data: [],
        pagination: { limit: 20, has_more: false, next_after_created_at: null, next_after_id: null },
      },
      "/api/v1/tags": { success: true, data: [] },
      "/api/v1/categories": { success: true, data: [] },
    });

    return <MediaListPageHarness />;
  },
};

export const Loading: Story = {
  render: () => {
    neverResolvingFetch();
    return <MediaListPageHarness />;
  },
};
