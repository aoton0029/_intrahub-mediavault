import { fireEvent, render, screen } from "@testing-library/react";
import { createMemoryRouter, RouterProvider } from "react-router-dom";
import { AppShell } from "@/components/layout/AppShell";
import { MovieDetailPage } from "./MovieDetailPage";
import { useMovieDetailData } from "@/hooks/useMovieDetailData";

vi.mock("@/hooks/useMovieDetailData", () => ({
  useMovieDetailData: vi.fn(),
}));

const mockUseMovieDetailData = vi.mocked(useMovieDetailData);

function createHookResult() {
  return {
    item: {
      id: "movie-1",
      media_type: "movie",
      title: "深海のオデッセイ",
      original_title: "The Odyssey of the Deep",
      description: "失われた海底都市を巡る、若き海洋学者たちの冒険譚。",
      cover_image_url: null,
      release_date: "2024-11-01",
      homepage_url: null,
      status: "in_progress",
      consumed_date: null,
      rating: 4,
      is_favorite: true,
      source: "api",
      external_id: "998821",
      created_at: "2026-07-01T12:00:00",
      updated_at: "2026-07-01T12:00:00",
      detail: {
        runtime_minutes: 132,
        original_language: "英語",
        production_companies: ["Meridian Pictures"],
        collection: "深海シリーズ",
        genres: ["SF", "冒険"],
        rating: 8.3,
        vote_count: 3204,
      },
      tags: [{ id: "tag-1", name: "映像美" }],
      categories: [{ id: "category-1", name: "2024年鑑賞" }],
      calibre_links: [],
      streaming_links: [],
    },
    propertyItems: [
      { key: "runtime_minutes", label: "上映時間", value: "132分", muted: false },
      { key: "original_language", label: "原語", value: "英語", muted: false },
      { key: "production_companies", label: "制作会社", value: "Meridian Pictures", muted: false },
      { key: "collection", label: "コレクション", value: "深海シリーズ", muted: false },
      { key: "genres", label: "ジャンル", value: "SF・冒険", muted: false },
      { key: "vote_count", label: "評価人数", value: "3204人", muted: false },
    ],
    staffList: [{ id: "staff-1", label: "レイラ・ハーモン", sub: "監督" }],
    relatedWorks: [{ id: "relation-1", title: "深海のオデッセイ2", relation: "reference" }],
    streaming: [{ id: "streaming-1", label: "Disney+", sub: "https://www.disneyplus.com/movies/xxxxx" }],
    resourceTabs: {
      links: [{ id: "link-1", label: "公式サイト", detail: "https://example.com" }],
      files: [{ id: "file-1", label: "パンフレット画像", detail: "image" }],
      trailers: [{ id: "trailer-1", label: "本予告編", detail: "https://video.example.com" }],
    },
    tags: [{ id: "tag-1", name: "映像美" }],
    categories: [{ id: "category-1", name: "2024年鑑賞" }],
    mylists: [{ id: "mylist-1", name: "映画館で観た作品", created_at: "2026-07-01T12:00:00" }],
    files: [{ id: "file-1", item_id: "movie-1", path: "/tmp/pamphlet.png", label: "パンフレット画像", file_type: "image", calibre_book_id: null, created_at: "2026-07-01T12:00:00" }],
    overview: "失われた海底都市を巡る、若き海洋学者たちの冒険譚。",
    actionLabel: "The Odyssey of the Deep ・ 2024年",
    isLoading: false,
    isError: false,
    refetch: vi.fn(),
    updateStatus: vi.fn().mockResolvedValue(undefined),
    updateRating: vi.fn().mockResolvedValue(undefined),
    updateFavorite: vi.fn().mockResolvedValue(undefined),
    addTag: vi.fn().mockResolvedValue(undefined),
    removeTag: vi.fn().mockResolvedValue(undefined),
    addCategory: vi.fn().mockResolvedValue(undefined),
    removeCategory: vi.fn().mockResolvedValue(undefined),
    removeMylist: vi.fn().mockResolvedValue(undefined),
    addStaff: vi.fn().mockResolvedValue(undefined),
    removeStaff: vi.fn().mockResolvedValue(undefined),
    addRelation: vi.fn().mockResolvedValue(undefined),
    removeRelation: vi.fn().mockResolvedValue(undefined),
    addStreamingLink: vi.fn().mockResolvedValue(undefined),
    removeStreamingLink: vi.fn().mockResolvedValue(undefined),
    addLink: vi.fn().mockResolvedValue(undefined),
    removeLink: vi.fn().mockResolvedValue(undefined),
    addFile: vi.fn().mockResolvedValue(undefined),
    removeFile: vi.fn().mockResolvedValue(undefined),
    addTrailer: vi.fn().mockResolvedValue(undefined),
    removeTrailer: vi.fn().mockResolvedValue(undefined),
    linkCalibre: vi.fn().mockResolvedValue(undefined),
  } as ReturnType<typeof useMovieDetailData>;
}

function renderWithRouter(initialEntry = "/media/movie-1") {
  const router = createMemoryRouter(
    [
      {
        path: "/",
        element: <AppShell />,
        children: [{ path: "media/:id", element: <MovieDetailPage /> }],
      },
    ],
    { initialEntries: [initialEntry] },
  );

  return render(<RouterProvider router={router} />);
}

describe("MovieDetailPage", () => {
  beforeEach(() => {
    mockUseMovieDetailData.mockReturnValue(createHookResult());
  });

  afterEach(() => {
    mockUseMovieDetailData.mockReset();
  });

  it("renders rail and main sections in movie order without groups", () => {
    renderWithRouter();

    expect(screen.getByText("深海のオデッセイ")).toBeInTheDocument();
    expect(screen.getByText("The Odyssey of the Deep ・ 2024年")).toBeInTheDocument();
    expect(screen.getByText("失われた海底都市を巡る、若き海洋学者たちの冒険譚。")).toBeInTheDocument();
    expect(screen.getByText("上映時間")).toBeInTheDocument();
    expect(screen.getByText("レイラ・ハーモン")).toBeInTheDocument();
    expect(screen.getByText("深海のオデッセイ2")).toBeInTheDocument();
    expect(screen.getByText("Disney+")).toBeInTheDocument();
    expect(screen.queryByText("シーズン構成")).not.toBeInTheDocument();
    expect(screen.getByRole("link", { name: "編集する" })).toHaveAttribute("href", "/media/movie-1/edit");
  });

  it("uses movie-specific status labels and sends completed rather than done", () => {
    const updateStatus = vi.fn().mockResolvedValue(undefined);
    mockUseMovieDetailData.mockReturnValue({
      ...createHookResult(),
      updateStatus,
    } as ReturnType<typeof useMovieDetailData>);

    renderWithRouter();

    fireEvent.click(screen.getByRole("button", { name: "視聴中" }));
    expect(screen.getByRole("button", { name: "視聴済" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "視聴済" }));

    expect(updateStatus).toHaveBeenCalledWith("completed");
    expect(updateStatus).not.toHaveBeenCalledWith("done");
  });
});
