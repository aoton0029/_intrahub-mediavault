import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen } from "@testing-library/react";
import { createMemoryRouter, RouterProvider, useLocation } from "react-router-dom";
import { AcademicBookListPage } from "./AcademicBookListPage";
import { useMediaListData } from "@/hooks/useMediaListData";

vi.mock("@/hooks/useMediaListData", () => ({
  useMediaListData: vi.fn(),
}));

const mockUseMediaListData = vi.mocked(useMediaListData);

function LocationProbe() {
  const location = useLocation();
  return <div data-testid="location">{location.search}</div>;
}

function renderWithRouter(initialEntry = "/academic-books") {
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
        path: "/academic-books",
        element: (
          <>
            <AcademicBookListPage />
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

describe("AcademicBookListPage", () => {
  beforeEach(() => {
    mockUseMediaListData.mockImplementation((filters) => ({
      items: [],
      mediaCards: [
        { title: "分散システム設計の原理", badge: "学術書", href: "/academic-books/1", variant: "compact", rating: 4.5 },
      ],
      hasNextPage: false,
      fetchNextPage: vi.fn(),
      isFetchingNextPage: false,
      isLoading: false,
      isError: false,
      tags: filters.tagId ? [{ id: filters.tagId, name: "積読", item_count: 1 }] : [],
      categories: filters.categoryId ? [{ id: filters.categoryId, name: "研究メモ", item_count: 1 }] : [],
    }));
  });

  afterEach(() => {
    mockUseMediaListData.mockReset();
  });

  it("renders the toolbar, compact grid, and completion sentinel without a media type select", () => {
    const { container } = renderWithRouter("/academic-books?tag_id=tag-1");

    expect(screen.queryByRole("combobox", { name: "種別" })).not.toBeInTheDocument();
    expect(screen.getByRole("combobox", { name: "並び順" })).toBeInTheDocument();
    expect(screen.getByRole("textbox", { name: "タイトル検索" })).toBeInTheDocument();
    expect(screen.getAllByText("分散システム設計の原理").length).toBeGreaterThan(0);
    expect(screen.getByText("すべて読み込みました")).toBeInTheDocument();
    expect(container.querySelector(".card-grid.is-compact")).not.toBeNull();
    expect(screen.getAllByText("# 積読").length).toBeGreaterThan(0);
  });

  it("calls useMediaListData with academic_book fixed", () => {
    renderWithRouter("/academic-books?title=設計");

    expect(mockUseMediaListData).toHaveBeenCalledWith(
      {
        categoryId: undefined,
        isFavorite: undefined,
        sort: undefined,
        tagId: undefined,
        title: "設計",
      },
      expect.objectContaining({
        mediaTypeOverride: "academic_book",
      }),
    );
  });

  it("does not include rating sort", () => {
    renderWithRouter("/academic-books");

    const options = screen.getAllByRole("option").map((option) => option.textContent);
    expect(options).not.toContain("評価順");
    expect(options).toEqual(["追加日順", "更新日順", "タイトル順", "発売日順"]);
  });

  it("toggles the favorite parameter from the chip", () => {
    renderWithRouter("/academic-books");

    fireEvent.click(screen.getByRole("button", { name: "お気に入り" }));
    expect(screen.getByTestId("location")).toHaveTextContent("?is_favorite=true");

    fireEvent.click(screen.getByRole("button", { name: "お気に入り" }));
    expect(screen.getByTestId("location")).toHaveTextContent("");
  });
});
