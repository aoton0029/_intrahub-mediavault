import { useMemo } from "react";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryRouter, RouterProvider } from "react-router-dom";
import { MovieDetailPage } from "./MovieDetailPage";

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
      createMemoryRouter([{ path: "/media/:id", element: <MovieDetailPage /> }], {
        initialEntries: ["/media/movie-1"],
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
  title: "pages/MovieDetailPage",
  component: DetailPageHarness,
};

export default meta;
type Story = StoryObj<typeof DetailPageHarness>;

export const Default: Story = {
  render: () => {
    mockFetchByUrl({
      "/api/v1/items/movie-1": {
        success: true,
        data: {
          id: "movie-1",
          media_type: "movie",
          title: "夜明けのオーロラ",
          original_title: "Aurora at Dawn",
          description: "失われた記憶を辿る科学者の物語。",
          cover_image_url: null,
          release_date: "2025-11-14",
          homepage_url: "https://example.com",
          status: "completed",
          consumed_date: "2026-01-05",
          rating: 5,
          is_favorite: true,
          source: "api",
          external_id: "912345",
          created_at: "2026-07-01T12:00:00",
          updated_at: "2026-07-01T12:00:00",
          detail: {
            runtime_minutes: 128,
            original_language: "en",
            production_companies: ["Nova Pictures"],
            collection: "オーロラシリーズ",
            genres: ["SF", "ドラマ"],
            rating: 7.9,
            vote_count: 4210,
          },
          tags: [{ id: "tag-1", name: "号泣注意" }],
          categories: [{ id: "category-1", name: "映画館鑑賞" }],
          calibre_links: [],
          streaming_links: [],
        },
      },
      "/api/v1/items/movie-1/staff": {
        success: true,
        data: [
          {
            id: "staff-1",
            item_id: "movie-1",
            staff_id: "external-staff-1",
            role: "監督",
            character_name: null,
            staff_name: "リナ・ホルスト",
          },
        ],
      },
      "/api/v1/items/movie-1/cast": {
        success: true,
        data: [
          {
            id: "cast-1",
            item_id: "movie-1",
            cast_id: "external-cast-1",
            character_name: "エレナ",
            cast_name: "ソフィア・マレン",
          },
        ],
      },
      "/api/v1/items/movie-1/relations": {
        success: true,
        data: [
          {
            id: "relation-1",
            item_id: "movie-1",
            related_item_id: "movie-2",
            relation_type: "reference",
            created_at: "2026-07-01T12:00:00",
            related_item_title: "夜明けのオーロラ 前日譚",
          },
        ],
      },
      "/api/v1/items/movie-1/streaming-links": {
        success: true,
        data: [{ id: "streaming-1", item_id: "movie-1", platform: "disney_plus", url: "https://disneyplus.example.com", created_at: "2026-07-01T12:00:00" }],
      },
      "/api/v1/items/movie-1/mylists": {
        success: true,
        data: [{ id: "mylist-1", name: "泣ける映画", created_at: "2026-07-01T12:00:00" }],
      },
      "/api/v1/items/movie-1/links": {
        success: true,
        data: [{ id: "link-1", item_id: "movie-1", url: "https://example.com", label: "公式サイト", created_at: "2026-07-01T12:00:00" }],
      },
      "/api/v1/items/movie-1/files": {
        success: true,
        data: [{ id: "file-1", item_id: "movie-1", path: "/tmp/pamphlet.pdf", label: "パンフレットPDF", file_type: "pdf", calibre_book_id: null, created_at: "2026-07-01T12:00:00" }],
      },
      "/api/v1/items/movie-1/trailers": {
        success: true,
        data: [{ id: "trailer-1", item_id: "movie-1", url: "https://video.example.com", label: "本予告編", created_at: "2026-07-01T12:00:00" }],
      },
    });

    return <DetailPageHarness />;
  },
};

export const ManualEntry: Story = {
  render: () => {
    mockFetchByUrl({
      "/api/v1/items/movie-1": {
        success: true,
        data: {
          id: "movie-1",
          media_type: "movie",
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
      "/api/v1/items/movie-1/staff": { success: true, data: [] },
      "/api/v1/items/movie-1/cast": { success: true, data: [] },
      "/api/v1/items/movie-1/relations": { success: true, data: [] },
      "/api/v1/items/movie-1/streaming-links": { success: true, data: [] },
      "/api/v1/items/movie-1/mylists": { success: true, data: [] },
      "/api/v1/items/movie-1/links": { success: true, data: [] },
      "/api/v1/items/movie-1/files": { success: true, data: [] },
      "/api/v1/items/movie-1/trailers": { success: true, data: [] },
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
