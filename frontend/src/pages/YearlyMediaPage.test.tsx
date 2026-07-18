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
        { year: 2026, count: 2 },
        { year: 2024, count: 1 },
      ],
      isLoading: false,
      isError: false,
    });
    mockUseYearItems.mockImplementation((_filters, year) => ({
      mediaCards:
        year === 2026
          ? [
              { title: "Ghost in the Shell", badge: "Anime", href: "/media/1", variant: "compact" },
              { title: "Ring", badge: "Movie", href: "/media/2", variant: "compact" },
            ]
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

  it("renders year sections in descending order with counts and item cards", () => {
    renderWithRouter();

    const headings = screen.getAllByRole("heading", { level: 2 });
    expect(headings[0]).toHaveTextContent("2026年");
    expect(headings[0]).toHaveTextContent("(2)");
    expect(headings[1]).toHaveTextContent("2024年");
    expect(screen.getByText("Ghost in the Shell")).toBeInTheDocument();
    expect(screen.getByText("Dune")).toBeInTheDocument();
  });

  it("updates date_field in search params when the axis chip is clicked", () => {
    renderWithRouter();

    fireEvent.click(screen.getByRole("button", { name: "視聴・読了年" }));
    expect(screen.getByTestId("location")).toHaveTextContent("?date_field=consumed");

    fireEvent.click(screen.getByRole("button", { name: "リリース年" }));
    expect(screen.getByTestId("location")).not.toHaveTextContent("date_field");
  });

  it("passes the consumed date field from the URL to the data hooks", () => {
    renderWithRouter("/collection/yearly?date_field=consumed");

    expect(mockUseYearlyMediaData).toHaveBeenCalledWith({ dateField: "consumed", mediaType: undefined, sort: "rating" });
  });

  it("updates media_type in search params when a dropdown option is selected", () => {
    renderWithRouter();

    fireEvent.click(screen.getByRole("button", { name: "メディア種別" }));
    fireEvent.click(screen.getByRole("option", { name: "映画" }));

    expect(screen.getByTestId("location")).toHaveTextContent("?media_type=movie");
  });

  it("shows an empty state when no years exist", () => {
    mockUseYearlyMediaData.mockReturnValue({ years: [], isLoading: false, isError: false });

    renderWithRouter();

    expect(screen.getByText("表示できる作品がありません")).toBeInTheDocument();
  });
});
