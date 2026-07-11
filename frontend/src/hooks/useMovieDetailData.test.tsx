import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { mapPropertyItems, useMovieDetailData } from "./useMovieDetailData";

function renderWithClient(ui: ReactNode) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  return render(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>);
}

function TestComponent() {
  const data = useMovieDetailData("movie-1");

  return (
    <>
      <div data-testid="properties">{JSON.stringify(data.propertyItems)}</div>
      <div data-testid="overview">{data.overview}</div>
    </>
  );
}

describe("useMovieDetailData", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("maps snake_case movie detail fields into six property items", async () => {
    const items = mapPropertyItems({
      runtime_minutes: 132,
      original_language: "英語",
      production_companies: ["Meridian Pictures", "Oceanic Works"],
      collection: "深海シリーズ",
      genres: ["SF", "冒険"],
      rating: 8.3,
      vote_count: 3204,
    });

    expect(items).toEqual([
      { key: "runtime_minutes", label: "上映時間", value: "132分", muted: false },
      { key: "original_language", label: "原語", value: "英語", muted: false },
      { key: "production_companies", label: "制作会社", value: "Meridian Pictures, Oceanic Works", muted: false },
      { key: "collection", label: "コレクション", value: "深海シリーズ", muted: false },
      { key: "genres", label: "ジャンル", value: "SF・冒険", muted: false },
      { key: "vote_count", label: "評価人数", value: "3204人", muted: false },
    ]);
  });

  it("handles manual items with detail=null without crashing", async () => {
    vi.spyOn(global, "fetch").mockImplementation(async (input) => {
      const url = String(input);
      const responses: Record<string, unknown> = {
        "/items/movie-1": {
          success: true,
          data: {
            id: "movie-1",
            media_type: "movie",
            title: "手動登録映画",
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
        "/items/movie-1/staff": { success: true, data: [] },
        "/items/movie-1/relations": { success: true, data: [] },
        "/items/movie-1/streaming-links": { success: true, data: [] },
        "/items/movie-1/mylists": { success: true, data: [] },
        "/items/movie-1/links": { success: true, data: [] },
        "/items/movie-1/files": { success: true, data: [] },
        "/items/movie-1/trailers": { success: true, data: [] },
      };

      const payload = responses[url];
      if (!payload) {
        throw new Error(`Unexpected fetch: ${url}`);
      }

      return new Response(JSON.stringify(payload));
    });

    renderWithClient(<TestComponent />);

    await waitFor(() => expect(screen.getByTestId("properties")).toHaveTextContent("上映時間"));
    expect(screen.getByTestId("properties")).toHaveTextContent("未登録");
    expect(screen.getByTestId("overview")).toHaveTextContent("");
  });
});
