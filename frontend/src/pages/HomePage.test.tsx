import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { MemoryRouter } from "react-router-dom";
import { HomePage } from "./HomePage";
import { SectionHeading } from "@/components/home/SectionHeading";
import { StatCard } from "@/components/home/StatCard";
import { useHomeData } from "@/hooks/useHomeData";

vi.mock("@/hooks/useHomeData", () => ({
  useHomeData: vi.fn(),
}));

const mockUseHomeData = vi.mocked(useHomeData);

function renderWithProviders(ui: ReactNode) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter>{ui}</MemoryRouter>
    </QueryClientProvider>,
  );
}

describe("HomePage", () => {
  beforeEach(() => {
    mockUseHomeData.mockReturnValue({
      data: {
        stats: {
          totalCount: 128,
          inProgressCount: 17,
          doneCount: 94,
          favoriteCount: 22,
        },
        recentItems: [
          { title: "星屑のシンフォニア", badge: "アニメ", rating: 4.5, favorite: true, href: "/media/1" },
          { title: "塩の記憶", badge: "小説", href: "/media/2" },
        ],
        inProgressItems: [
          {
            title: "緋色の境界、青の余白",
            badge: "アニメ",
            href: "/media/3",
            meta: (
              <div className="prop-taglist" style={{ marginTop: 6 }}>
                <span className="tag-pill" style={{ color: "var(--status-progress)" }}>
                  視聴中
                </span>
              </div>
            ),
          },
        ],
      },
      isLoading: false,
      isError: false,
      error: null,
      isSuccess: true,
      status: "success",
      fetchStatus: "idle",
      dataUpdatedAt: 0,
      errorUpdatedAt: 0,
      failureCount: 0,
      failureReason: null,
      errorUpdateCount: 0,
      isFetched: true,
      isFetchedAfterMount: true,
      isFetching: false,
      isInitialLoading: false,
      isLoadingError: false,
      isPaused: false,
      isPending: false,
      isPlaceholderData: false,
      isRefetchError: false,
      isRefetching: false,
      isStale: false,
      isEnabled: true,
      promise: Promise.resolve(undefined),
      refetch: vi.fn(),
    } as unknown as ReturnType<typeof useHomeData>);
  });

  afterEach(() => {
    mockUseHomeData.mockReset();
  });

  it("applies favorite styling on StatCard", () => {
    const { container } = renderWithProviders(<StatCard label="❤ お気に入り" value={22} isFavorite />);
    expect(container.firstChild).toHaveClass("stat-card", "is-favorite");
  });

  it("passes seeAllHref to Link", () => {
    renderWithProviders(<SectionHeading title="最近追加した作品" seeAllHref="/media" />);
    expect(screen.getByRole("link", { name: /すべて見る/ })).toHaveAttribute("href", "/media");
  });

  it("renders stats, section headings, grids, progress tag, hidden rating meta, and see-all links", () => {
    const { container } = renderWithProviders(<HomePage />);
    const seeAllLinks = screen.getAllByRole("link", { name: /すべて見る/ });

    expect(screen.getByText("128")).toBeInTheDocument();
    expect(screen.getByText("17")).toBeInTheDocument();
    expect(screen.getByText("94")).toBeInTheDocument();
    expect(screen.getByText("22")).toBeInTheDocument();
    expect(screen.getByText("最近追加した作品")).toBeInTheDocument();
    expect(screen.getAllByText("進行中")).toHaveLength(2);
    expect(screen.getByText("星屑のシンフォニア")).toBeInTheDocument();
    expect(screen.getByText("緋色の境界、青の余白")).toBeInTheDocument();
    expect(screen.getByText("視聴中")).toHaveClass("tag-pill");
    expect(seeAllLinks[0]).toHaveAttribute("href", "/media");
    expect(seeAllLinks[1]).toHaveAttribute("href", "/media?status=in_progress");

    const novelCard = screen.getByText("塩の記憶").closest(".media-card");
    expect(novelCard?.querySelector(".meta")).toBeNull();
    expect(container.querySelectorAll(".card-grid")).toHaveLength(2);
    expect(screen.getByText("星屑のシンフォニア").closest("a")).toHaveAttribute("href", "/media/1");
  });

  it("renders the in-progress see-all link with status query", () => {
    renderWithProviders(<HomePage />);
    const links = screen.getAllByRole("link", { name: /すべて見る/ });
    expect(links[1]).toHaveAttribute("href", "/media?status=in_progress");
  });
});
