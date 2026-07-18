import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen } from "@testing-library/react";
import { createMemoryRouter, RouterProvider, useLocation } from "react-router-dom";
import { YearlyMediaPage } from "./YearlyMediaPage";
import { useYearItems, useYearlyMediaData } from "@/hooks/useYearlyMediaData";

vi.mock("@/hooks/useYearlyMediaData", () => ({
  useYearlyMediaData: vi.fn(),
  useYearItems: vi.fn(),
}));

const mockUseYearlyMediaData = vi.mocked(useYearlyMediaData);
const mockUseYearItems = vi.mocked(useYearItems);

function LocationProbe() {
  const location = useLocation();
  return <div data-testid="location">{location.search}</div>;
}

function renderWithRouter(initialEntry = "/collection/yearly") {
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
        path: "/collection/yearly",
        element: (
          <>
            <YearlyMediaPage />
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

describe("YearlyMediaPage", () => {
  beforeEach(() => {
    mockUseYearlyMediaData.mockReturnValue({
      years: [
        {
          year: 2026,
          count: 3,
          media_types: [
            { media_type: "anime", count: 2 },
            { media_type: "academic_book", count: 1 },
          ],
        },
        {
          year: 2024,
          count: 1,
          media_types: [{ media_type: "movie", count: 1 }],
        },
      ],
      isLoading: false,
      isError: false,
    });
    mockUseYearItems.mockImplementation((filters, year) => ({
      mediaCards:
        year === 2026
          ? filters.mediaType === "anime"
            ? [
                { title: "Ghost in the Shell", badge: "Anime", href: "/media/1", variant: "compact" },
                { title: "Akira", badge: "Anime", href: "/media/4", variant: "compact" },
              ]
            : [{ title: "SICP", badge: "Academic Book", href: "/media/5", variant: "compact" }]
          : [{ title: "Dune", badge: "Movie", href: "/media/3", variant: "compact" }],
      hasNextPage: false,
      fetchNextPage: vi.fn(),
      isFetchingNextPage: false,
      isLoading: false,
      isError: false,
    }));
  });

  afterEach(() => {
    mockUseYearlyMediaData.mockReset();
    mockUseYearItems.mockReset();
  });

  it("renders media type sections for the latest year with counts and item cards", () => {
    renderWithRouter();

    const headings = screen.getAllByRole("heading", { level: 2 });
    expect(headings).toHaveLength(2);
    expect(headings[0]).toHaveTextContent("アニメ");
    expect(headings[0]).toHaveTextContent("(2)");
    expect(headings[1]).toHaveTextContent("学術書");
    expect(headings[1]).toHaveTextContent("(1)");
    expect(screen.getByText("Ghost in the Shell")).toBeInTheDocument();
    expect(screen.getByText("SICP")).toBeInTheDocument();
    expect(screen.queryByText("Dune")).not.toBeInTheDocument();
  });

  it("shows the year from the URL and renders that year's sections", () => {
    renderWithRouter("/collection/yearly?year=2024");

    const headings = screen.getAllByRole("heading", { level: 2 });
    expect(headings).toHaveLength(1);
    expect(headings[0]).toHaveTextContent("映画");
    expect(screen.getByText("Dune")).toBeInTheDocument();
  });

  it("updates year in search params when a year is selected from the dropdown", () => {
    renderWithRouter();

    fireEvent.click(screen.getByRole("button", { name: "年" }));
    fireEvent.click(screen.getByRole("option", { name: "2024年 (1)" }));

    expect(screen.getByTestId("location")).toHaveTextContent("?year=2024");
  });

  it("defaults to both date fields selected and passes 'any' to the years hook", () => {
    renderWithRouter();

    expect(mockUseYearlyMediaData).toHaveBeenCalledWith({ dateField: "any" });
  });

  it("narrows to a single date field when the other chip is toggled off", () => {
    renderWithRouter();

    // 両方ON状態で「視聴・読了年」をOFF → releaseのみ
    fireEvent.click(screen.getByRole("button", { name: "視聴・読了年" }));
    expect(screen.getByTestId("location")).toHaveTextContent("?date_field=release");

    // 「視聴・読了年」を再度ON → 両方（date_field削除）
    fireEvent.click(screen.getByRole("button", { name: "視聴・読了年" }));
    expect(screen.getByTestId("location")).not.toHaveTextContent("date_field");
  });

  it("keeps at least one date field selected", () => {
    renderWithRouter("/collection/yearly?date_field=release");

    // 単独選択中のチップはOFFにできない
    fireEvent.click(screen.getByRole("button", { name: "リリース年" }));
    expect(screen.getByTestId("location")).toHaveTextContent("?date_field=release");
  });

  it("passes the consumed date field from the URL to the years hook", () => {
    renderWithRouter("/collection/yearly?date_field=consumed");

    expect(mockUseYearlyMediaData).toHaveBeenCalledWith({ dateField: "consumed" });
  });

  it("shows an empty state when no years exist", () => {
    mockUseYearlyMediaData.mockReturnValue({ years: [], isLoading: false, isError: false });

    renderWithRouter();

    expect(screen.getByText("表示できる作品がありません")).toBeInTheDocument();
  });
});
