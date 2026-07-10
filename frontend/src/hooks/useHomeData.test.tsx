import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { useHomeData } from "./useHomeData";

function TestComponent() {
  const { data, isSuccess } = useHomeData();

  if (!isSuccess || !data) {
    return <div>loading</div>;
  }

  return (
    <>
      <div data-testid="stats">{JSON.stringify(data.stats)}</div>
      <div data-testid="recent">{JSON.stringify(data.recentItems.map((item) => item.title))}</div>
      <div data-testid="progress">{JSON.stringify(data.inProgressItems.map((item) => item.title))}</div>
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

describe("useHomeData", () => {
  it("returns recent and in-progress items under the expected keys", async () => {
    vi.spyOn(global, "fetch").mockImplementation(async (input) => {
      const url = String(input);

      if (url === "/items?limit=100") {
        return new Response(JSON.stringify({
          success: true,
          data: [
            { id: "1", media_type: "anime", title: "最近1", status: "in_progress", rating: 4.5, is_favorite: true },
            { id: "2", media_type: "movie", title: "完了1", status: "done", rating: null, is_favorite: false },
          ],
          pagination: { has_more: false, next_after_created_at: null, next_after_id: null },
        }));
      }

      if (url === "/items?limit=6") {
        return new Response(JSON.stringify({
          success: true,
          data: [
            { id: "1", media_type: "anime", title: "最近1", status: "in_progress", rating: 4.5, is_favorite: true },
          ],
          pagination: { has_more: false, next_after_created_at: null, next_after_id: null },
        }));
      }

      if (url === "/items?status=in_progress&limit=6") {
        return new Response(JSON.stringify({
          success: true,
          data: [
            { id: "3", media_type: "manga", title: "進行中1", status: "in_progress", rating: null, is_favorite: false },
          ],
          pagination: { has_more: false, next_after_created_at: null, next_after_id: null },
        }));
      }

      throw new Error(`Unexpected fetch: ${url}`);
    });

    renderWithClient(<TestComponent />);

    await waitFor(() => expect(screen.getByTestId("stats")).toBeInTheDocument());

    expect(screen.getByTestId("stats")).toHaveTextContent('"totalCount":2');
    expect(screen.getByTestId("stats")).toHaveTextContent('"inProgressCount":1');
    expect(screen.getByTestId("stats")).toHaveTextContent('"doneCount":1');
    expect(screen.getByTestId("stats")).toHaveTextContent('"favoriteCount":1');
    expect(screen.getByTestId("recent")).toHaveTextContent("最近1");
    expect(screen.getByTestId("progress")).toHaveTextContent("進行中1");
  });
});
