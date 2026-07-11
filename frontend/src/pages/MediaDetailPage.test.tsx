import { render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryRouter, RouterProvider } from "react-router-dom";
import { AppShell } from "@/components/layout/AppShell";
import { MediaDetailPage } from "./MediaDetailPage";

vi.mock("./MovieDetailPage", () => ({
  MovieDetailPage: () => <div>Movie Detail Mock</div>,
}));

vi.mock("./AnimeDetailPage", () => ({
  AnimeDetailPage: () => <div>Anime Detail Mock</div>,
}));

function renderWithRouter(initialEntry = "/media/1") {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  const router = createMemoryRouter(
    [
      {
        path: "/",
        element: <AppShell />,
        children: [
          { path: "media/:id", element: <MediaDetailPage /> },
        ],
      },
    ],
    { initialEntries: [initialEntry] },
  );

  return render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
}

describe("MediaDetailPage", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("renders anime detail when media_type is anime", async () => {
    vi.spyOn(global, "fetch").mockResolvedValue(
      new Response(JSON.stringify({ success: true, data: { id: "1", media_type: "anime" } })),
    );

    renderWithRouter();

    await waitFor(() => expect(screen.getByText("Anime Detail Mock")).toBeInTheDocument());
    expect(document.querySelector(".breadcrumb")?.textContent).toContain("一般メディア / アニメ");
  });

  it("renders movie detail when media_type is movie", async () => {
    vi.spyOn(global, "fetch").mockResolvedValue(
      new Response(JSON.stringify({ success: true, data: { id: "1", media_type: "movie" } })),
    );

    renderWithRouter();

    await waitFor(() => expect(screen.getByText("Movie Detail Mock")).toBeInTheDocument());
    expect(screen.getByRole("link", { name: "編集する" })).toHaveAttribute("href", "/media/1/edit");
  });

  it("renders an empty state when media_type is unsupported", async () => {
    vi.spyOn(global, "fetch").mockResolvedValue(
      new Response(JSON.stringify({ success: true, data: { id: "1", media_type: "game" } })),
    );

    renderWithRouter();

    await waitFor(() => expect(screen.getByText("未対応の種別です")).toBeInTheDocument());
    expect(screen.getByText("この種別の詳細画面は未対応です。今後の対応をお待ちください。")).toBeInTheDocument();
  });
});
