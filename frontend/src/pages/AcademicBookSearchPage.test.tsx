import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";
import { createMemoryRouter, RouterProvider } from "react-router-dom";
import { AcademicBookSearchPage } from "./AcademicBookSearchPage";

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
        path: "/academic-books/search",
        element: <AcademicBookSearchPage />,
      },
      {
        path: "/academic-books/:id",
        element: <div>detail page</div>,
      },
      {
        path: "/settings",
        element: <div>settings</div>,
      },
    ],
    { initialEntries: ["/academic-books/search"] },
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

describe("AcademicBookSearchPage", () => {
  it("searches with media_type=academic_book and renders search-result cards", async () => {
    const searchRequests: string[] = [];

    server.use(
      http.get("/api/v1/items/search", ({ request }) => {
        const url = new URL(request.url);
        searchRequests.push(url.search);

        return HttpResponse.json({
          success: true,
          data: [
            {
              id: "9784000000001",
              media_type: "academic_book",
              provider: "rakuten_books",
              title: "位相幾何学入門",
              thumbnail_url: null,
            },
          ],
        });
      }),
    );

    renderWithRouter();

    const user = userEvent.setup();
    await user.type(screen.getByRole("textbox", { name: "作品名" }), "位相");
    await user.click(screen.getByRole("button", { name: "検索" }));

    await screen.findByText("位相幾何学入門");

    expect(searchRequests).toEqual(["?media_type=academic_book&q=%E4%BD%8D%E7%9B%B8"]);
    expect(document.querySelector(".media-card.search-result.is-compact")).not.toBeNull();
    expect(screen.getByText("学術書")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "取り込む" })).toBeInTheDocument();
  });

  it("navigates to the academic book detail page after a successful import response", async () => {
    const importBodies: Array<{ media_type: string; provider: string | null; external_id: string }> = [];

    server.use(
      http.get("/api/v1/items/search", () =>
        HttpResponse.json({
          success: true,
          data: [
            {
              id: "9784000000001",
              media_type: "academic_book",
              provider: "rakuten_books",
              title: "位相幾何学入門",
              thumbnail_url: null,
            },
          ],
        }),
      ),
      http.post("/api/v1/items/import", async ({ request }) => {
        importBodies.push((await request.json()) as { media_type: string; provider: string | null; external_id: string });
        return HttpResponse.json({ success: true, data: { id: "item-uuid-1" } }, { status: 201 });
      }),
    );

    renderWithRouter();

    const user = userEvent.setup();
    await user.type(screen.getByRole("textbox", { name: "作品名" }), "位相");
    await user.click(screen.getByRole("button", { name: "検索" }));
    await screen.findByText("位相幾何学入門");

    await user.click(screen.getByRole("button", { name: "取り込む" }));

    await screen.findByText("detail page");
    expect(importBodies).toEqual([{ media_type: "academic_book", provider: "rakuten_books", external_id: "9784000000001" }]);
  });

  it("marks an item as imported when the API returns ITEM_ALREADY_IMPORTED", async () => {
    server.use(
      http.get("/api/v1/items/search", () =>
        HttpResponse.json({
          success: true,
          data: [
            {
              id: "9784000000002",
              media_type: "academic_book",
              provider: "rakuten_books",
              title: "圏論の地平",
              thumbnail_url: null,
            },
          ],
        }),
      ),
      http.post("/api/v1/items/import", () =>
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
    await user.type(screen.getByRole("textbox", { name: "作品名" }), "圏論");
    await user.click(screen.getByRole("button", { name: "検索" }));
    await screen.findByText("圏論の地平");

    await user.click(screen.getByRole("button", { name: "取り込む" }));

    await waitFor(() => expect(screen.getByRole("button", { name: "取り込み済み" })).toBeDisabled());
  });

  it("shows only the API key empty state when search returns API_KEY_NOT_CONFIGURED", async () => {
    server.use(
      http.get("/api/v1/items/search", () =>
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
    await user.type(screen.getByRole("textbox", { name: "作品名" }), "位相");
    await user.click(screen.getByRole("button", { name: "検索" }));

    await screen.findByText("APIキーが設定されていません");

    expect(screen.getByText("学術書・専門書の検索には 楽天ブックス のAPIキーが必要です。設定画面から登録してください。")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "設定を開く" })).toHaveAttribute("href", "/settings?tab=api");
    expect(document.querySelector(".card-grid.is-compact")).toBeNull();
  });
});
