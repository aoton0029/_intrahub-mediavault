import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";
import { createMemoryRouter, RouterProvider } from "react-router-dom";
import { MediaSearchPage } from "./MediaSearchPage";

const server = setupServer();

function renderWithRouter() {
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

  const router = createMemoryRouter(
    [
      {
        path: "/media/search",
        element: <MediaSearchPage />,
      },
      {
        path: "/settings",
        element: <div>settings</div>,
      },
    ],
    { initialEntries: ["/media/search"] },
  );

  return render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
}

beforeAll(() => {
  server.listen({ onUnhandledRequest: "error" });
});

afterEach(() => {
  server.resetHandlers();
});

afterAll(() => {
  server.close();
});

describe("MediaSearchPage", () => {
  it("searches with the selected media type and query, then renders search-result cards", async () => {
    const searchRequests: string[] = [];

    server.use(
      http.get("/items/search", ({ request }) => {
        const url = new URL(request.url);
        searchRequests.push(url.search);

        return HttpResponse.json({
          success: true,
          data: [
            {
              id: "anime-1",
              media_type: "anime",
              provider: "annict",
              title: "星屑のシンフォニア",
              thumbnail_url: null,
            },
          ],
        });
      }),
    );

    renderWithRouter();

    const user = userEvent.setup();
    await user.type(screen.getByRole("textbox", { name: "作品名" }), "星屑");
    await user.click(screen.getByRole("button", { name: "検索" }));

    await screen.findByText("星屑のシンフォニア");

    expect(searchRequests).toEqual(["?media_type=anime&q=%E6%98%9F%E5%B1%91"]);
    expect(document.querySelector(".card-grid.is-compact")).not.toBeNull();
    expect(document.querySelector(".media-card.search-result.is-compact")).not.toBeNull();
    expect(screen.getAllByText("アニメ")).toHaveLength(2);
    expect(screen.getByRole("button", { name: "取り込む" })).toBeInTheDocument();
  });

  it("marks an item as imported after a successful import response", async () => {
    const importBodies: Array<{ media_type: string; provider: string | null; external_id: string }> = [];

    server.use(
      http.get("/items/search", () =>
        HttpResponse.json({
          success: true,
          data: [
            {
              id: "anime-1",
              media_type: "anime",
              provider: "annict",
              title: "星屑のシンフォニア",
              thumbnail_url: null,
            },
          ],
        }),
      ),
      http.post("/items/import", async ({ request }) => {
        importBodies.push((await request.json()) as { media_type: string; provider: string | null; external_id: string });
        return HttpResponse.json({ success: true }, { status: 201 });
      }),
    );

    renderWithRouter();

    const user = userEvent.setup();
    await user.type(screen.getByRole("textbox", { name: "作品名" }), "星屑");
    await user.click(screen.getByRole("button", { name: "検索" }));
    await screen.findByText("星屑のシンフォニア");

    await user.click(screen.getByRole("button", { name: "取り込む" }));

    await waitFor(() => expect(screen.getByRole("button", { name: "取り込み済み" })).toBeDisabled());
    expect(importBodies).toEqual([{ media_type: "anime", provider: "annict", external_id: "anime-1" }]);
  });

  it("marks an item as imported when the API returns ITEM_ALREADY_IMPORTED", async () => {
    server.use(
      http.get("/items/search", () =>
        HttpResponse.json({
          success: true,
          data: [
            {
              id: "anime-2",
              media_type: "anime",
              provider: "annict",
              title: "緋色の境界、青の余白",
              thumbnail_url: null,
            },
          ],
        }),
      ),
      http.post("/items/import", () =>
        HttpResponse.json(
          {
            code: "ITEM_ALREADY_IMPORTED",
            message: "Already imported",
          },
          { status: 409 },
        ),
      ),
    );

    renderWithRouter();

    const user = userEvent.setup();
    await user.type(screen.getByRole("textbox", { name: "作品名" }), "境界");
    await user.click(screen.getByRole("button", { name: "検索" }));
    await screen.findByText("緋色の境界、青の余白");

    await user.click(screen.getByRole("button", { name: "取り込む" }));

    await waitFor(() => expect(screen.getByRole("button", { name: "取り込み済み" })).toBeDisabled());
  });

  it("shows only the API key empty state when search returns API_KEY_NOT_CONFIGURED", async () => {
    server.use(
      http.get("/items/search", () =>
        HttpResponse.json(
          {
            code: "API_KEY_NOT_CONFIGURED",
            message: "Missing API key",
          },
          { status: 422 },
        ),
      ),
    );

    renderWithRouter();

    const user = userEvent.setup();
    await user.selectOptions(screen.getByRole("combobox", { name: "種別" }), "movie");
    await user.type(screen.getByRole("textbox", { name: "作品名" }), "インセプション");
    await user.click(screen.getByRole("button", { name: "検索" }));

    await screen.findByText("APIキーが設定されていません");

    expect(screen.getByText("この種別の検索には TMDb のAPIキーが必要です。設定画面から登録してください。")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "設定を開く" })).toHaveAttribute("href", "/settings?tab=api");
    expect(document.querySelector(".card-grid.is-compact")).toBeNull();
  });
});
