import { fireEvent, render, screen } from "@testing-library/react";
import { createMemoryRouter, RouterProvider } from "react-router-dom";
import { AnimeDetailPage } from "./AnimeDetailPage";
import { useAnimeDetailData } from "@/hooks/useAnimeDetailData";

vi.mock("@/hooks/useAnimeDetailData", () => ({
  useAnimeDetailData: vi.fn(),
}));

const mockUseAnimeDetailData = vi.mocked(useAnimeDetailData);

function createHookResult() {
  return {
    item: {
      id: "anime-1",
      media_type: "anime",
      title: "星屑のシンフォニア",
      original_title: "Symphonia of Stardust",
      description: "記憶を失った少女が旅をする物語。",
      cover_image_url: null,
      release_date: "2025-04-05",
      homepage_url: null,
      status: "in_progress",
      consumed_date: null,
      rating: 4,
      is_favorite: true,
      source: "api",
      external_id: "58214",
      created_at: "2026-07-01T12:00:00",
      updated_at: "2026-07-01T12:00:00",
      detail: {
        episodes: 12,
        status: "Finished Airing",
        season: "2025-spring",
        year: 2025,
        studios: [],
        source: "Original",
        duration: null,
        trailer_url: null,
        genres: [],
        rating: 8.5,
        url: null,
        alternative_titles: [],
      },
      tags: [{ id: "tag-1", name: "神作画" }],
      categories: [{ id: "category-1", name: "2026年鑑賞予定" }],
      calibre_links: [],
      streaming_links: [],
      theme_songs: [],
    },
    groups: [{ id: "group-1", label: "シーズン1", episodes: [{ id: "episode-1", number: "01", title: "星が墜ちた夜" }] }],
    staffList: [{ id: "staff-1", label: "新津 明日香", sub: "監督" }],
    castList: [{ id: "cast-1", label: "結城 かなで", sub: "ルカ役" }],
    themeSongs: [
      {
        type: "op",
        label: "OP",
        songs: [
          {
            id: "its-1",
            title: "夜明けのアリア",
            sub: "高橋洋子",
            note: null,
            links: [{ id: "tsl-1", type: "youtube", url: "https://example.com/mv", label: "MV" }],
          },
        ],
      },
    ],
    relatedWorks: [{ id: "relation-1", relatedItemId: "item-2", title: "星屑のシンフォニア OVA", relation: "reference" }],
    streaming: [{ id: "streaming-1", label: "Netflix", sub: "https://netflix.example.com" }],
    images: [{ id: "image-1", url: "https://img.example.com/1.jpg", isCover: false }],
    resourceTabs: {
      links: [{ id: "link-1", label: "公式サイト", detail: "https://example.com" }],
      files: [{ id: "file-1", label: "パンフレットPDF", detail: "pdf" }],
      trailers: [{ id: "trailer-1", label: "本予告編", detail: "https://video.example.com" }],
    },
    tags: [{ id: "tag-1", name: "神作画" }],
    categories: [{ id: "category-1", name: "2026年鑑賞予定" }],
    mylists: [{ id: "mylist-1", name: "お気に入り原作", created_at: "2026-07-01T12:00:00" }],
    files: [{ id: "file-1", item_id: "anime-1", path: "/tmp/pamphlet.pdf", label: "パンフレットPDF", file_type: "pdf", calibre_book_id: null, created_at: "2026-07-01T12:00:00" }],
    citations: [],
    overview: "記憶を失った少女が旅をする物語。",
    actionLabel: "星屑のシンフォニア ・ 2025",
    isLoading: false,
    isError: false,
    refetch: vi.fn(),
    updateStatus: vi.fn().mockResolvedValue(undefined),
    updateRating: vi.fn().mockResolvedValue(undefined),
    updateFavorite: vi.fn().mockResolvedValue(undefined),
    updateConsumedDate: vi.fn().mockResolvedValue(undefined),
    updateDescription: vi.fn().mockResolvedValue(undefined),
    addTag: vi.fn().mockResolvedValue(undefined),
    removeTag: vi.fn().mockResolvedValue(undefined),
    addCategory: vi.fn().mockResolvedValue(undefined),
    removeCategory: vi.fn().mockResolvedValue(undefined),
    removeMylist: vi.fn().mockResolvedValue(undefined),
    addGroup: vi.fn().mockResolvedValue(undefined),
    addEpisode: vi.fn().mockResolvedValue(undefined),
    addStaff: vi.fn().mockResolvedValue(undefined),
    removeStaff: vi.fn().mockResolvedValue(undefined),
    addCast: vi.fn().mockResolvedValue(undefined),
    removeCast: vi.fn().mockResolvedValue(undefined),
    addThemeSong: vi.fn().mockResolvedValue(undefined),
    removeThemeSong: vi.fn().mockResolvedValue(undefined),
    addRelation: vi.fn().mockResolvedValue(undefined),
    removeRelation: vi.fn().mockResolvedValue(undefined),
    addStreamingLink: vi.fn().mockResolvedValue(undefined),
    removeStreamingLink: vi.fn().mockResolvedValue(undefined),
    addImage: vi.fn().mockResolvedValue(undefined),
    removeImage: vi.fn().mockResolvedValue(undefined),
    setCoverImage: vi.fn().mockResolvedValue(undefined),
    addLink: vi.fn().mockResolvedValue(undefined),
    removeLink: vi.fn().mockResolvedValue(undefined),
    addFile: vi.fn().mockResolvedValue(undefined),
    uploadFile: vi.fn().mockResolvedValue(undefined),
    removeFile: vi.fn().mockResolvedValue(undefined),
    addTrailer: vi.fn().mockResolvedValue(undefined),
    removeTrailer: vi.fn().mockResolvedValue(undefined),
    linkCalibre: vi.fn().mockResolvedValue(undefined),
    addCitation: vi.fn().mockResolvedValue(undefined),
    updateCitation: vi.fn().mockResolvedValue(undefined),
    removeCitation: vi.fn().mockResolvedValue(undefined),
    deleteItem: vi.fn().mockResolvedValue(undefined),
  } as ReturnType<typeof useAnimeDetailData>;
}

function renderWithRouter(initialEntry = "/media/anime-1") {
  const router = createMemoryRouter(
    [
      {
        path: "/media/:id",
        element: <AnimeDetailPage />,
      },
    ],
    { initialEntries: [initialEntry] },
  );

  return render(<RouterProvider router={router} />);
}

describe("AnimeDetailPage", () => {
  beforeEach(() => {
    mockUseAnimeDetailData.mockReturnValue(createHookResult());
  });

  afterEach(() => {
    mockUseAnimeDetailData.mockReset();
  });

  it("renders rail and main sections without the property list section", () => {
    renderWithRouter();

    expect(screen.getByText("Symphonia of Stardust")).toBeInTheDocument();
    expect(screen.getByText("星屑のシンフォニア ・ 2025")).toBeInTheDocument();
    expect(screen.getByText("記憶を失った少女が旅をする物語。")).toBeInTheDocument();
    expect(screen.getByText("シーズン1")).toBeInTheDocument();
    expect(screen.getByText("新津 明日香")).toBeInTheDocument();
    expect(screen.getByText("星屑のシンフォニア OVA")).toBeInTheDocument();
    expect(screen.getByText("Netflix")).toBeInTheDocument();
    expect(screen.queryByText("プロパティ")).not.toBeInTheDocument();
  });

  it("sends completed rather than done when the status is changed", () => {
    const updateStatus = vi.fn().mockResolvedValue(undefined);

    mockUseAnimeDetailData.mockReturnValue({
      ...createHookResult(),
      updateStatus,
    } as ReturnType<typeof useAnimeDetailData>);

    renderWithRouter();

    fireEvent.click(screen.getByRole("button", { name: "視聴中" }));
    fireEvent.click(screen.getByRole("button", { name: "視聴済" }));

    expect(updateStatus).toHaveBeenCalledWith("completed");
    expect(updateStatus).not.toHaveBeenCalledWith("done");
  });

  it("updates consumed_date through the date editor", () => {
    const updateConsumedDate = vi.fn().mockResolvedValue(undefined);

    mockUseAnimeDetailData.mockReturnValue({
      ...createHookResult(),
      updateConsumedDate,
    } as ReturnType<typeof useAnimeDetailData>);

    renderWithRouter();

    expect(screen.getByText("視聴日未登録")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "視聴日未登録" }));
    fireEvent.change(screen.getByDisplayValue(""), { target: { value: "2026-01-05" } });
    fireEvent.click(screen.getByRole("button", { name: "保存" }));

    expect(updateConsumedDate).toHaveBeenCalledWith("2026-01-05");
  });
});
