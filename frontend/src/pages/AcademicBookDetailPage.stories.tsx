import { useMemo } from "react";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryRouter, RouterProvider } from "react-router-dom";
import { AcademicBookDetailPage } from "./AcademicBookDetailPage";

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
      createMemoryRouter([{ path: "/academic-books/:id", element: <AcademicBookDetailPage /> }], {
        initialEntries: ["/academic-books/book-1"],
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
  title: "pages/AcademicBookDetailPage",
  component: DetailPageHarness,
};

export default meta;
type Story = StoryObj<typeof DetailPageHarness>;

export const Default: Story = {
  render: () => {
    mockFetchByUrl({
      "/api/v1/items/book-1": {
        success: true,
        data: {
          id: "book-1",
          media_type: "academic_book",
          title: "位相幾何学入門",
          original_title: null,
          description: "位相空間論からホモロジーまでを平易に解説する入門書。",
          cover_image_url: null,
          release_date: "2022-04-10",
          homepage_url: "https://example.com",
          status: "in_progress",
          consumed_date: null,
          rating: 4,
          is_favorite: false,
          source: "api",
          external_id: "9784000000001",
          created_at: "2026-07-01T12:00:00",
          updated_at: "2026-07-01T12:00:00",
          detail: {
            authors: "高梨 一朗",
            publisher: "岩波書店",
            isbn: "9784000000001",
            series_name: null,
          },
          tags: [{ id: "tag-1", name: "数学" }],
          categories: [{ id: "category-1", name: "教科書" }],
          calibre_links: [],
          streaming_links: [],
        },
      },
      "/api/v1/items/book-1/staff": { success: true, data: [] },
      "/api/v1/items/book-1/cast": { success: true, data: [] },
      "/api/v1/items/book-1/relations": {
        success: true,
        data: [
          {
            id: "relation-1",
            item_id: "book-1",
            related_item_id: "book-2",
            relation_type: "reference",
            created_at: "2026-07-01T12:00:00",
            related_item_title: "位相幾何学演習",
          },
        ],
      },
      "/api/v1/items/book-1/streaming-links": { success: true, data: [] },
      "/api/v1/items/book-1/mylists": { success: true, data: [] },
      "/api/v1/items/book-1/links": {
        success: true,
        data: [{ id: "link-1", item_id: "book-1", url: "https://example.com", label: "出版社ページ", created_at: "2026-07-01T12:00:00" }],
      },
      "/api/v1/items/book-1/files": {
        success: true,
        data: [{ id: "file-1", item_id: "book-1", path: "/tmp/book.pdf", label: "本文PDF", file_type: "pdf", calibre_book_id: null, created_at: "2026-07-01T12:00:00" }],
      },
      "/api/v1/items/book-1/trailers": { success: true, data: [] },
    });

    return <DetailPageHarness />;
  },
};

export const ManualEntry: Story = {
  render: () => {
    mockFetchByUrl({
      "/api/v1/items/book-1": {
        success: true,
        data: {
          id: "book-1",
          media_type: "academic_book",
          title: "手動登録の専門書",
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
      "/api/v1/items/book-1/staff": { success: true, data: [] },
      "/api/v1/items/book-1/cast": { success: true, data: [] },
      "/api/v1/items/book-1/relations": { success: true, data: [] },
      "/api/v1/items/book-1/streaming-links": { success: true, data: [] },
      "/api/v1/items/book-1/mylists": { success: true, data: [] },
      "/api/v1/items/book-1/links": { success: true, data: [] },
      "/api/v1/items/book-1/files": { success: true, data: [] },
      "/api/v1/items/book-1/trailers": { success: true, data: [] },
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
