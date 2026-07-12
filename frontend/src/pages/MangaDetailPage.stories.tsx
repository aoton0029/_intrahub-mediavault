import { useMemo } from "react";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryRouter, RouterProvider } from "react-router-dom";
import { MangaDetailPage } from "./MangaDetailPage";

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
      createMemoryRouter([{ path: "/media/:id", element: <MangaDetailPage /> }], {
        initialEntries: ["/media/manga-1"],
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
  title: "pages/MangaDetailPage",
  component: DetailPageHarness,
};

export default meta;
type Story = StoryObj<typeof DetailPageHarness>;

export const Default: Story = {
  render: () => {
    mockFetchByUrl({
      "/api/v1/items/manga-1": {
        success: true,
        data: {
          id: "manga-1",
          media_type: "manga",
          title: "月夜の刃",
          original_title: null,
          description: "剣士が月の呪いを解くため旅をする物語。",
          cover_image_url: null,
          release_date: "2024-06-20",
          homepage_url: "https://example.com",
          status: "in_progress",
          consumed_date: null,
          rating: 4,
          is_favorite: true,
          source: "api",
          external_id: "9784088xxxxxx",
          created_at: "2026-07-01T12:00:00",
          updated_at: "2026-07-01T12:00:00",
          detail: {
            authors: "青柳 廉",
            publisher: "集英社",
            isbn: "9784088xxxxxx",
            series_name: "月夜の刃",
          },
          tags: [{ id: "tag-1", name: "神作画" }],
          categories: [{ id: "category-1", name: "2026年購入" }],
          calibre_links: [],
          streaming_links: [],
        },
      },
      "/api/v1/items/manga-1/groups": {
        success: true,
        data: [
          {
            id: "group-1",
            item_id: "manga-1",
            parent_item_id: null,
            group_type: "volume",
            group_name: "1巻",
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
            title: "月が満ちる夜",
            original_title: null,
            air_date: null,
            description: null,
            created_at: "2026-07-01T12:00:00",
            updated_at: "2026-07-01T12:00:00",
          },
        ],
      },
      "/api/v1/items/manga-1/staff": { success: true, data: [] },
      "/api/v1/items/manga-1/cast": { success: true, data: [] },
      "/api/v1/items/manga-1/relations": {
        success: true,
        data: [
          {
            id: "relation-1",
            item_id: "manga-1",
            related_item_id: "manga-2",
            relation_type: "reference",
            created_at: "2026-07-01T12:00:00",
            related_item_title: "月夜の刃 外伝",
          },
        ],
      },
      "/api/v1/items/manga-1/streaming-links": { success: true, data: [] },
      "/api/v1/items/manga-1/mylists": {
        success: true,
        data: [{ id: "mylist-1", name: "積読リスト", created_at: "2026-07-01T12:00:00" }],
      },
      "/api/v1/items/manga-1/links": {
        success: true,
        data: [{ id: "link-1", item_id: "manga-1", url: "https://example.com", label: "公式サイト", created_at: "2026-07-01T12:00:00" }],
      },
      "/api/v1/items/manga-1/files": {
        success: true,
        data: [{ id: "file-1", item_id: "manga-1", path: "/tmp/vol1.pdf", label: "1巻PDF", file_type: "pdf", calibre_book_id: null, created_at: "2026-07-01T12:00:00" }],
      },
      "/api/v1/items/manga-1/trailers": { success: true, data: [] },
    });

    return <DetailPageHarness />;
  },
};

export const ManualEntry: Story = {
  render: () => {
    mockFetchByUrl({
      "/api/v1/items/manga-1": {
        success: true,
        data: {
          id: "manga-1",
          media_type: "manga",
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
      "/api/v1/items/manga-1/groups": { success: true, data: [] },
      "/api/v1/items/manga-1/staff": { success: true, data: [] },
      "/api/v1/items/manga-1/cast": { success: true, data: [] },
      "/api/v1/items/manga-1/relations": { success: true, data: [] },
      "/api/v1/items/manga-1/streaming-links": { success: true, data: [] },
      "/api/v1/items/manga-1/mylists": { success: true, data: [] },
      "/api/v1/items/manga-1/links": { success: true, data: [] },
      "/api/v1/items/manga-1/files": { success: true, data: [] },
      "/api/v1/items/manga-1/trailers": { success: true, data: [] },
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
