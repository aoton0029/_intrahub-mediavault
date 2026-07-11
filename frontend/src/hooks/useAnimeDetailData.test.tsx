import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { useAnimeDetailData } from "./useAnimeDetailData";

function renderWithClient(ui: ReactNode) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
      mutations: {
        retry: false,
      },
    },
  });

  return render(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>);
}

function TestComponent() {
  const data = useAnimeDetailData("anime-1");

  return (
    <>
      <div data-testid="groups">{JSON.stringify(data.groups)}</div>
      <div data-testid="overview">{data.overview}</div>
    </>
  );
}

describe("useAnimeDetailData", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("maps snake_case item groups and episodes into GroupList data", async () => {
    vi.spyOn(global, "fetch").mockImplementation(async (input) => {
      const url = String(input);

      const responses: Record<string, unknown> = {
        "/items/anime-1": {
          success: true,
          data: {
            id: "anime-1",
            media_type: "anime",
            title: "星屑のシンフォニア",
            original_title: "Symphonia of Stardust",
            description: "概要",
            cover_image_url: null,
            release_date: "2025-04-05",
            homepage_url: null,
            status: "in_progress",
            consumed_date: null,
            rating: 4,
            is_favorite: true,
            source: "api",
            external_id: "58214",
            created_at: "2026-07-01T12:00:00",
            updated_at: "2026-07-01T12:00:00",
            detail: { year: 2025, episodes: 12, status: "Finished Airing", season: "2025-spring", studios: [], source: "Original", duration: null, trailer_url: null, genres: [], rating: 8.5, url: null, alternative_titles: [] },
            tags: [],
            categories: [],
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
        "/items/anime-1/staff": { success: true, data: [] },
        "/items/anime-1/relations": { success: true, data: [] },
        "/items/anime-1/streaming-links": { success: true, data: [] },
        "/items/anime-1/mylists": { success: true, data: [] },
        "/items/anime-1/links": { success: true, data: [] },
        "/items/anime-1/files": { success: true, data: [] },
        "/items/anime-1/trailers": { success: true, data: [] },
      };

      const payload = responses[url];
      if (!payload) {
        throw new Error(`Unexpected fetch: ${url}`);
      }

      return new Response(JSON.stringify(payload));
    });

    renderWithClient(<TestComponent />);

    await waitFor(() => expect(screen.getByTestId("groups")).toHaveTextContent('"number":"01"'));
    expect(screen.getByTestId("groups")).toHaveTextContent('"number":"12"');
    expect(screen.getByTestId("groups")).toHaveTextContent("シーズン1");
  });

  it("handles manual items with detail=null without crashing", async () => {
    vi.spyOn(global, "fetch").mockImplementation(async (input) => {
      const url = String(input);

      const responses: Record<string, unknown> = {
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
      };

      const payload = responses[url];
      if (!payload) {
        throw new Error(`Unexpected fetch: ${url}`);
      }

      return new Response(JSON.stringify(payload));
    });

    renderWithClient(<TestComponent />);

    await waitFor(() => expect(screen.getByTestId("groups")).toHaveTextContent("[]"));
    expect(screen.getByTestId("overview")).toHaveTextContent("");
  });
});
