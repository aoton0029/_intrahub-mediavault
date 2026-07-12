import { useMemo } from "react";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryRouter, RouterProvider } from "react-router-dom";
import { NovelDetailPage } from "./NovelDetailPage";

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
      createMemoryRouter([{ path: "/media/:id", element: <NovelDetailPage /> }], {
        initialEntries: ["/media/novel-1"],
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
  title: "pages/NovelDetailPage",
  component: DetailPageHarness,
};

export default meta;
type Story = StoryObj<typeof DetailPageHarness>;

export const Default: Story = {
  render: () => {
    mockFetchByUrl({
      "/api/v1/items/novel-1": {
        success: true,
        data: {
          id: "novel-1",
          media_type: "novel",
          title: "水底の図書館",
          original_title: null,
          description: "沈んだ都市に眠る図書館を巡る旅の物語。",
          cover_image_url: null,
          release_date: "2023-09-15",
          homepage_url: "https://example.com",
          status: "completed",
          consumed_date: "2026-02-01",
          rating: 5,
          is_favorite: true,
          source: "api",
          external_id: "9784041xxxxxx",
          created_at: "2026-07-01T12:00:00",
          updated_at: "2026-07-01T12:00:00",
          detail: {
            authors: "藤堂 千夏",
            publisher: "KADOKAWA",
            isbn: "9784041xxxxxx",
            series_name: "水底の図書館",
          },
          tags: [{ id: "tag-1", name: "泣ける" }],
          categories: [{ id: "category-1", name: "読了済み" }],
          calibre_links: [],
          streaming_links: [],
        },
      },
      "/api/v1/items/novel-1/groups": {
        success: true,
        data: [
          {
            id: "group-1",
            item_id: "novel-1",
            parent_item_id: null,
            group_type: "chapter",
            group_name: "第1章",
            number: 1,
            display_order: 1,
            created_at: "2026-07-01T12:00:00",
            updated_at: "2026-07-01T12:00:00",
          },
        ],
      },
      "/api/v1/groups/group-1/episodes": {
        success: true,
        data: [
          {
            id: "episode-1",
            group_id: "group-1",
            episode_number: 1,
            title: "沈んだ街",
            original_title: null,
            air_date: null,
            description: null,
            created_at: "2026-07-01T12:00:00",
            updated_at: "2026-07-01T12:00:00",
          },
        ],
      },
      "/api/v1/items/novel-1/staff": { success: true, data: [] },
      "/api/v1/items/novel-1/cast": { success: true, data: [] },
      "/api/v1/items/novel-1/relations": {
        success: true,
        data: [
          {
            id: "relation-1",
            item_id: "novel-1",
            related_item_id: "novel-2",
            relation_type: "reference",
            created_at: "2026-07-01T12:00:00",
            related_item_title: "水底の図書館 外伝",
          },
        ],
      },
      "/api/v1/items/novel-1/streaming-links": { success: true, data: [] },
      "/api/v1/items/novel-1/mylists": {
        success: true,
        data: [{ id: "mylist-1", name: "積読リスト", created_at: "2026-07-01T12:00:00" }],
      },
      "/api/v1/items/novel-1/links": {
        success: true,
        data: [{ id: "link-1", item_id: "novel-1", url: "https://example.com", label: "公式サイト", created_at: "2026-07-01T12:00:00" }],
      },
      "/api/v1/items/novel-1/files": {
        success: true,
        data: [{ id: "file-1", item_id: "novel-1", path: "/tmp/novel.pdf", label: "本文PDF", file_type: "pdf", calibre_book_id: null, created_at: "2026-07-01T12:00:00" }],
      },
      "/api/v1/items/novel-1/trailers": { success: true, data: [] },
    });

    return <DetailPageHarness />;
  },
};

export const ManualEntry: Story = {
  render: () => {
    mockFetchByUrl({
      "/api/v1/items/novel-1": {
        success: true,
        data: {
          id: "novel-1",
          media_type: "novel",
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
      "/api/v1/items/novel-1/groups": { success: true, data: [] },
      "/api/v1/items/novel-1/staff": { success: true, data: [] },
      "/api/v1/items/novel-1/cast": { success: true, data: [] },
      "/api/v1/items/novel-1/relations": { success: true, data: [] },
      "/api/v1/items/novel-1/streaming-links": { success: true, data: [] },
      "/api/v1/items/novel-1/mylists": { success: true, data: [] },
      "/api/v1/items/novel-1/links": { success: true, data: [] },
      "/api/v1/items/novel-1/files": { success: true, data: [] },
      "/api/v1/items/novel-1/trailers": { success: true, data: [] },
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
