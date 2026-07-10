import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen } from "@testing-library/react";
import { createMemoryRouter, RouterProvider, useLocation } from "react-router-dom";
import { MediaListPage } from "./MediaListPage";
import { useMediaListData } from "@/hooks/useMediaListData";

vi.mock("@/hooks/useMediaListData", () => ({
  useMediaListData: vi.fn(),
}));

const mockUseMediaListData = vi.mocked(useMediaListData);

function LocationProbe() {
  const location = useLocation();
  return <div data-testid="location">{location.search}</div>;
}

function renderWithRouter(initialEntry = "/media") {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });

  const router = createMemoryRouter(
    [
      {
        path: "/media",
        element: (
          <>
            <MediaListPage />
            <LocationProbe />
          </>
        ),
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

describe("MediaListPage", () => {
  beforeEach(() => {
    mockUseMediaListData.mockImplementation((filters) => ({
      items: [],
      mediaCards: [
        { title: "Ghost in the Shell", badge: "Anime", href: "/media/1", variant: "compact", rating: 4.5 },
        { title: "Ring", badge: "Movie", href: "/media/2", variant: "compact" },
      ],
      hasNextPage: false,
      fetchNextPage: vi.fn(),
      isFetchingNextPage: false,
      isLoading: false,
      isError: false,
      tags: filters.tagId ? [{ id: filters.tagId, name: "SF", item_count: 1 }] : [],
      categories: filters.categoryId ? [{ id: filters.categoryId, name: "Weekend", item_count: 1 }] : [],
    }));
  });

  afterEach(() => {
    mockUseMediaListData.mockReset();
  });

  it("renders the toolbar, compact grid, and completion sentinel", () => {
    const { container } = renderWithRouter("/media?tag_id=tag-1");

    expect(screen.getByRole("combobox", { name: "種別" })).toBeInTheDocument();
    expect(screen.getByRole("combobox", { name: "並び順" })).toBeInTheDocument();
    expect(screen.getByRole("textbox", { name: "タイトル検索" })).toBeInTheDocument();
    expect(screen.getByText("Ghost in the Shell")).toBeInTheDocument();
    expect(screen.getByText("Ring")).toBeInTheDocument();
    expect(screen.getByText("All items loaded")).toBeInTheDocument();
    expect(container.querySelector(".card-grid.is-compact")).not.toBeNull();
    expect(screen.getByText("# SF")).toBeInTheDocument();
  });

  it("updates media_type in search params when the select changes", () => {
    renderWithRouter("/media");

    fireEvent.change(screen.getByRole("combobox", { name: "種別" }), { target: { value: "movie" } });

    expect(screen.getByTestId("location")).toHaveTextContent("?media_type=movie");
  });

  it("toggles the favorite parameter from the chip", () => {
    renderWithRouter("/media");

    fireEvent.click(screen.getByRole("button", { name: "Favorites" }));
    expect(screen.getByTestId("location")).toHaveTextContent("?is_favorite=true");

    fireEvent.click(screen.getByRole("button", { name: "Favorites" }));
    expect(screen.getByTestId("location")).toHaveTextContent("");
  });

  it("removes tag and category params from active chips", () => {
    renderWithRouter("/media?tag_id=tag-1&category_id=cat-1");

    fireEvent.click(screen.getByRole("button", { name: "# SFを解除" }));
    expect(screen.getByTestId("location")).toHaveTextContent("?category_id=cat-1");

    fireEvent.click(screen.getByRole("button", { name: "Category: Weekendを解除" }));
    expect(screen.getByTestId("location")).toHaveTextContent("");
  });
});
