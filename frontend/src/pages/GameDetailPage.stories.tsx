import { useMemo } from "react";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryRouter, RouterProvider } from "react-router-dom";
import { GameDetailPage } from "./GameDetailPage";

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

function DetailPageHarness() {
  const queryClient = useMemo(() => new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } }), []);
  const router = useMemo(
    () =>
      createMemoryRouter([{ path: "/media/:id", element: <GameDetailPage /> }], {
        initialEntries: ["/media/game-1"],
      }),
    [],
  );

  return (
    <QueryClientProvider client={queryClient}>
      <div className="content">
        <RouterProvider router={router} />
      </div>
    </QueryClientProvider>
  );
}

const meta: Meta<typeof DetailPageHarness> = {
  title: "pages/GameDetailPage",
  component: DetailPageHarness,
};

export default meta;
type Story = StoryObj<typeof DetailPageHarness>;

export const Default: Story = {
  render: () => {
    mockFetchByUrl({
      "/api/v1/items/game-1": {
        success: true,
        data: {
          id: "game-1",
          media_type: "game",
          title: "星海航路",
          original_title: "Stellar Route",
          description: "銀河を旅する交易商人シミュレーションゲーム。",
          cover_image_url: null,
          release_date: "2025-03-20",
          homepage_url: "https://example.com",
          status: "in_progress",
          consumed_date: null,
          rating: 4,
          is_favorite: true,
          source: "api",
          external_id: "1234560",
          created_at: "2026-07-01T12:00:00",
          updated_at: "2026-07-01T12:00:00",
          detail: {
            platforms: ["Windows", "macOS"],
            developers: ["Nova Interactive"],
            publishers: ["Nova Interactive"],
            screenshots: [],
            metacritic: 84,
            genres: ["シミュレーション", "アドベンチャー"],
          },
          tags: [{ id: "tag-1", name: "積みゲー" }],
          categories: [{ id: "category-1", name: "2026年購入" }],
          calibre_links: [],
          streaming_links: [],
        },
      },
      "/api/v1/items/game-1/staff": { success: true, data: [] },
      "/api/v1/items/game-1/cast": { success: true, data: [] },
      "/api/v1/items/game-1/relations": {
        success: true,
        data: [
          {
            id: "relation-1",
            item_id: "game-1",
            related_item_id: "game-2",
            relation_type: "dlc",
            created_at: "2026-07-01T12:00:00",
            related_item_title: "星海航路 追加航路パック",
          },
        ],
      },
      "/api/v1/items/game-1/streaming-links": { success: true, data: [] },
      "/api/v1/items/game-1/mylists": {
        success: true,
        data: [{ id: "mylist-1", name: "積みゲーリスト", created_at: "2026-07-01T12:00:00" }],
      },
      "/api/v1/items/game-1/links": {
        success: true,
        data: [{ id: "link-1", item_id: "game-1", url: "https://example.com", label: "公式サイト", created_at: "2026-07-01T12:00:00" }],
      },
      "/api/v1/items/game-1/files": {
        success: true,
        data: [{ id: "file-1", item_id: "game-1", path: "/tmp/screenshot.png", label: "スクリーンショット", file_type: "image", calibre_book_id: null, created_at: "2026-07-01T12:00:00" }],
      },
      "/api/v1/items/game-1/trailers": {
        success: true,
        data: [{ id: "trailer-1", item_id: "game-1", url: "https://video.example.com", label: "トレーラー", created_at: "2026-07-01T12:00:00" }],
      },
    });

    return <DetailPageHarness />;
  },
};

export const ManualEntry: Story = {
  render: () => {
    mockFetchByUrl({
      "/api/v1/items/game-1": {
        success: true,
        data: {
          id: "game-1",
          media_type: "game",
          title: "手動登録作品",
          original_title: null,
          description: null,
          cover_image_url: null,
          release_date: null,
          homepage_url: null,
          status: "not_started",
          consumed_date: null,
          rating: null,
          is_favorite: false,
          source: "manual",
          external_id: null,
          created_at: "2026-07-01T12:00:00",
          updated_at: "2026-07-01T12:00:00",
          detail: null,
          tags: [],
          categories: [],
          calibre_links: [],
          streaming_links: [],
        },
      },
      "/api/v1/items/game-1/staff": { success: true, data: [] },
      "/api/v1/items/game-1/cast": { success: true, data: [] },
      "/api/v1/items/game-1/relations": { success: true, data: [] },
      "/api/v1/items/game-1/streaming-links": { success: true, data: [] },
      "/api/v1/items/game-1/mylists": { success: true, data: [] },
      "/api/v1/items/game-1/links": { success: true, data: [] },
      "/api/v1/items/game-1/files": { success: true, data: [] },
      "/api/v1/items/game-1/trailers": { success: true, data: [] },
    });

    return <DetailPageHarness />;
  },
};

export const Loading: Story = {
  render: () => {
    neverResolvingFetch();
    return <DetailPageHarness />;
  },
};
