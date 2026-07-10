import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { useMediaListData } from "./useMediaListData";

function TestComponent({ filters }: { filters: Parameters<typeof useMediaListData>[0] }) {
  const { mediaCards, hasNextPage, fetchNextPage } = useMediaListData(filters);

  return (
    <>
      <div data-testid="titles">{JSON.stringify(mediaCards.map((item: { title: string }) => item.title))}</div>
      <div data-testid="has-next-page">{String(Boolean(hasNextPage))}</div>
      <button type="button" onClick={() => void fetchNextPage()}>
        next
      </button>
    </>
  );
}

function renderWithClient(ui: ReactNode) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });

  return render(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>);
}

describe("useMediaListData", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("requests items with filter query parameters", async () => {
    vi.spyOn(global, "fetch").mockImplementation(async (input) => {
      const url = String(input);

      if (url === "/tags") {
        return new Response(JSON.stringify({ success: true, data: [] }));
      }

      if (url === "/categories") {
        return new Response(JSON.stringify({ success: true, data: [] }));
      }

      if (url === "/items?media_type=anime&is_favorite=true&tag_id=tag-1&category_id=cat-1&title=eva&limit=20") {
        return new Response(JSON.stringify({
          success: true,
          data: [{ id: "1", media_type: "anime", title: "EVA", status: "done", rating: 4.5, is_favorite: true, tags: [], categories: [] }],
          pagination: { has_more: false, next_after_created_at: null, next_after_id: null },
        }));
      }

      throw new Error(`Unexpected fetch: ${url}`);
    });

    renderWithClient(<TestComponent filters={{ mediaType: "anime", isFavorite: true, tagId: "tag-1", categoryId: "cat-1", title: "eva" }} />);

    await waitFor(() => expect(screen.getByTestId("titles")).toHaveTextContent("EVA"));
  });

  it("uses keyset pagination cursors when fetching the next page", async () => {
    vi.spyOn(global, "fetch").mockImplementation(async (input) => {
      const url = String(input);

      if (url === "/tags") {
        return new Response(JSON.stringify({ success: true, data: [] }));
      }

      if (url === "/categories") {
        return new Response(JSON.stringify({ success: true, data: [] }));
      }

      if (url === "/items?limit=20") {
        return new Response(JSON.stringify({
          success: true,
          data: [{ id: "1", media_type: "movie", title: "First", status: "done", rating: null, is_favorite: false, tags: [], categories: [] }],
          pagination: { has_more: true, next_after_created_at: "2026-07-01T00:00:00", next_after_id: "1" },
        }));
      }

      if (url === "/items?limit=20&after_created_at=2026-07-01T00%3A00%3A00&after_id=1") {
        return new Response(JSON.stringify({
          success: true,
          data: [{ id: "2", media_type: "movie", title: "Second", status: "done", rating: null, is_favorite: false, tags: [], categories: [] }],
          pagination: { has_more: false, next_after_created_at: null, next_after_id: null },
        }));
      }

      throw new Error(`Unexpected fetch: ${url}`);
    });

    renderWithClient(<TestComponent filters={{}} />);

    await waitFor(() => expect(screen.getByTestId("titles")).toHaveTextContent("First"));

    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "next" }));
    });

    await waitFor(() => expect(screen.getByTestId("titles")).toHaveTextContent("Second"));
  });

  it("stops pagination when the API reports has_more=false", async () => {
    vi.spyOn(global, "fetch").mockImplementation(async (input) => {
      const url = String(input);

      if (url === "/tags") {
        return new Response(JSON.stringify({ success: true, data: [] }));
      }

      if (url === "/categories") {
        return new Response(JSON.stringify({ success: true, data: [] }));
      }

      if (url === "/items?limit=20") {
        return new Response(JSON.stringify({
          success: true,
          data: [{ id: "1", media_type: "game", title: "Done", status: "done", rating: null, is_favorite: false, tags: [], categories: [] }],
          pagination: { has_more: false, next_after_created_at: null, next_after_id: null },
        }));
      }

      throw new Error(`Unexpected fetch: ${url}`);
    });

    renderWithClient(<TestComponent filters={{}} />);

    await waitFor(() => expect(screen.getByTestId("has-next-page")).toHaveTextContent("false"));
  });
});
