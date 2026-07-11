import { useMemo } from "react";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryRouter, RouterProvider } from "react-router-dom";
import { AnimeDetailPage } from "./AnimeDetailPage";

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
      createMemoryRouter([{ path: "/media/:id", element: <AnimeDetailPage /> }], {
        initialEntries: ["/media/anime-1"],
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
  title: "pages/AnimeDetailPage",
  component: DetailPageHarness,
};

export default meta;
type Story = StoryObj<typeof DetailPageHarness>;

export const Default: Story = {
  render: () => {
    mockFetchByUrl({
      "/items/anime-1": {
        success: true,
        data: {
          id: "anime-1",
          media_type: "anime",
          title: "星屑のシンフォニア",
          original_title: "Symphonia of Stardust",
          description: "記憶を失った少女が旅をする物語。",
          cover_image_url: null,
          release_date: "2025-04-05",
          homepage_url: "https://example.com",
          status: "in_progress",
          consumed_date: null,
          rating: 4,
          is_favorite: true,
          source: "api",
          external_id: "58214",
          created_at: "2026-07-01T12:00:00",
          updated_at: "2026-07-01T12:00:00",
          detail: {
            episodes: 12,
            status: "Finished Airing",
            season: "2025-spring",
            year: 2025,
            studios: [],
            source: "Original",
            duration: null,
            trailer_url: null,
            genres: [],
            rating: 8.5,
            url: null,
            alternative_titles: [],
          },
          tags: [{ id: "tag-1", name: "神作画" }],
          categories: [{ id: "category-1", name: "2026年鑑賞予定" }],
          calibre_links: [],
          streaming_links: [],
        },
      },
      "/items/anime-1/groups": {
        success: true,
        data: [
          {
            id: "group-1",
            item_id: "anime-1",
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
      "/groups/group-1/episodes": {
        success: true,
        data: [
          {
            id: "episode-1",
            group_id: "group-1",
            episode_number: 1,
            title: "星が墜ちた夜",
            original_title: null,
            air_date: null,
            description: null,
            created_at: "2026-07-01T12:00:00",
            updated_at: "2026-07-01T12:00:00",
          },
          {
            id: "episode-2",
            group_id: "group-1",
            episode_number: 12,
            title: "記憶の在処",
            original_title: null,
            air_date: null,
            description: null,
            created_at: "2026-07-01T12:00:00",
            updated_at: "2026-07-01T12:00:00",
          },
        ],
      },
      "/items/anime-1/staff": {
        success: true,
        data: [
          {
            id: "staff-1",
            item_id: "anime-1",
            staff_id: "external-staff-1",
            role: "監督",
            character_name: null,
            staff: { id: "external-staff-1", external_id: null, name: "新津 明日香", image_url: null, created_at: "2026-07-01T12:00:00" },
          },
        ],
      },
      "/items/anime-1/relations": {
        success: true,
        data: [
          {
            id: "relation-1",
            item_id: "anime-1",
            related_item_id: "anime-2",
            relation_type: "reference",
            created_at: "2026-07-01T12:00:00",
            related_item_title: "星屑のシンフォニア OVA",
          },
        ],
      },
      "/items/anime-1/streaming-links": {
        success: true,
        data: [{ id: "streaming-1", item_id: "anime-1", platform: "netflix", url: "https://netflix.example.com", created_at: "2026-07-01T12:00:00" }],
      },
      "/items/anime-1/mylists": {
        success: true,
        data: [{ id: "mylist-1", name: "お気に入り原作", created_at: "2026-07-01T12:00:00" }],
      },
      "/items/anime-1/links": {
        success: true,
        data: [{ id: "link-1", item_id: "anime-1", url: "https://example.com", label: "公式サイト", created_at: "2026-07-01T12:00:00" }],
      },
      "/items/anime-1/files": {
        success: true,
        data: [{ id: "file-1", item_id: "anime-1", path: "/tmp/pamphlet.pdf", label: "パンフレットPDF", file_type: "pdf", calibre_book_id: null, created_at: "2026-07-01T12:00:00" }],
      },
      "/items/anime-1/trailers": {
        success: true,
        data: [{ id: "trailer-1", item_id: "anime-1", url: "https://video.example.com", label: "本予告編", created_at: "2026-07-01T12:00:00" }],
      },
    });

    return <DetailPageHarness />;
  },
};

export const ManualEntry: Story = {
  render: () => {
    mockFetchByUrl({
      "/items/anime-1": {
        success: true,
        data: {
          id: "anime-1",
          media_type: "anime",
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
      "/items/anime-1/groups": { success: true, data: [] },
      "/items/anime-1/staff": { success: true, data: [] },
      "/items/anime-1/relations": { success: true, data: [] },
      "/items/anime-1/streaming-links": { success: true, data: [] },
      "/items/anime-1/mylists": { success: true, data: [] },
      "/items/anime-1/links": { success: true, data: [] },
      "/items/anime-1/files": { success: true, data: [] },
      "/items/anime-1/trailers": { success: true, data: [] },
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
