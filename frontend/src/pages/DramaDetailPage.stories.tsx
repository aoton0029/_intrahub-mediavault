import { useMemo } from "react";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryRouter, RouterProvider } from "react-router-dom";
import { DramaDetailPage } from "./DramaDetailPage";

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
      createMemoryRouter([{ path: "/media/:id", element: <DramaDetailPage /> }], {
        initialEntries: ["/media/drama-1"],
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
  title: "pages/DramaDetailPage",
  component: DetailPageHarness,
};

export default meta;
type Story = StoryObj<typeof DetailPageHarness>;

export const Default: Story = {
  render: () => {
    mockFetchByUrl({
      "/api/v1/items/drama-1": {
        success: true,
        data: {
          id: "drama-1",
          media_type: "drama",
          title: "深夜の約束",
          original_title: "Midnight Promise",
          description: "刑事と弁護士が過去の事件を追う連続ドラマ。",
          cover_image_url: null,
          release_date: "2025-01-10",
          homepage_url: "https://example.com",
          status: "in_progress",
          consumed_date: null,
          rating: 4,
          is_favorite: true,
          source: "api",
          external_id: "44231",
          created_at: "2026-07-01T12:00:00",
          updated_at: "2026-07-01T12:00:00",
          detail: {
            number_of_seasons: 2,
            number_of_episodes: 20,
            networks: ["フジテレビ"],
            status: "Returning Series",
            original_language: "ja",
            first_air_date: "2025-01-10",
            last_air_date: null,
            genres: ["サスペンス", "法廷"],
            rating: 7.6,
          },
          tags: [{ id: "tag-1", name: "一気見注意" }],
          categories: [{ id: "category-1", name: "2026年視聴" }],
          calibre_links: [],
          streaming_links: [],
        },
      },
      "/api/v1/items/drama-1/groups": {
        success: true,
        data: [
          {
            id: "group-1",
            item_id: "drama-1",
            parent_item_id: null,
            group_type: "season",
            group_name: "シーズン1",
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
            title: "沈黙の証言",
            original_title: null,
            air_date: null,
            description: null,
            created_at: "2026-07-01T12:00:00",
            updated_at: "2026-07-01T12:00:00",
          },
        ],
      },
      "/api/v1/items/drama-1/staff": {
        success: true,
        data: [
          {
            id: "staff-1",
            item_id: "drama-1",
            staff_id: "external-staff-1",
            role: "監督",
            character_name: null,
            staff_name: "村上 玲",
          },
        ],
      },
      "/api/v1/items/drama-1/cast": {
        success: true,
        data: [
          {
            id: "cast-1",
            item_id: "drama-1",
            cast_id: "external-cast-1",
            character_name: "高城 弁護士",
            cast_name: "北条 真央",
          },
        ],
      },
      "/api/v1/items/drama-1/relations": {
        success: true,
        data: [
          {
            id: "relation-1",
            item_id: "drama-1",
            related_item_id: "drama-2",
            relation_type: "reference",
            created_at: "2026-07-01T12:00:00",
            related_item_title: "深夜の約束 スペシャル",
          },
        ],
      },
      "/api/v1/items/drama-1/streaming-links": {
        success: true,
        data: [{ id: "streaming-1", item_id: "drama-1", platform: "netflix", url: "https://netflix.example.com", created_at: "2026-07-01T12:00:00" }],
      },
      "/api/v1/items/drama-1/mylists": {
        success: true,
        data: [{ id: "mylist-1", name: "土曜ドラマ", created_at: "2026-07-01T12:00:00" }],
      },
      "/api/v1/items/drama-1/links": {
        success: true,
        data: [{ id: "link-1", item_id: "drama-1", url: "https://example.com", label: "公式サイト", created_at: "2026-07-01T12:00:00" }],
      },
      "/api/v1/items/drama-1/files": {
        success: true,
        data: [{ id: "file-1", item_id: "drama-1", path: "/tmp/pamphlet.pdf", label: "パンフレットPDF", file_type: "pdf", calibre_book_id: null, created_at: "2026-07-01T12:00:00" }],
      },
      "/api/v1/items/drama-1/trailers": {
        success: true,
        data: [{ id: "trailer-1", item_id: "drama-1", url: "https://video.example.com", label: "本予告編", created_at: "2026-07-01T12:00:00" }],
      },
    });

    return <DetailPageHarness />;
  },
};

export const ManualEntry: Story = {
  render: () => {
    mockFetchByUrl({
      "/api/v1/items/drama-1": {
        success: true,
        data: {
          id: "drama-1",
          media_type: "drama",
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
      "/api/v1/items/drama-1/groups": { success: true, data: [] },
      "/api/v1/items/drama-1/staff": { success: true, data: [] },
      "/api/v1/items/drama-1/cast": { success: true, data: [] },
      "/api/v1/items/drama-1/relations": { success: true, data: [] },
      "/api/v1/items/drama-1/streaming-links": { success: true, data: [] },
      "/api/v1/items/drama-1/mylists": { success: true, data: [] },
      "/api/v1/items/drama-1/links": { success: true, data: [] },
      "/api/v1/items/drama-1/files": { success: true, data: [] },
      "/api/v1/items/drama-1/trailers": { success: true, data: [] },
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
