import { fireEvent, render, screen } from "@testing-library/react";
import { detailSectionMatrix } from "@/config/detailSections";
import { DetailLayout, DetailMain, DetailRail, ThemeSongList } from "./DetailLayout";

describe("detail components", () => {
  it("renders rail and main slots", () => {
    render(<DetailLayout rail={<div>rail</div>} main={<div>main</div>} />);
    expect(screen.getByText("rail")).toBeInTheDocument();
    expect(screen.getByText("main")).toBeInTheDocument();
  });

  it("renders optional sections as tabs in canonical order", () => {
    render(<DetailMain overview="概要" groups={[{ id: "g1", label: "S1", episodes: [{ id: "e1", number: "01", title: "ep" }] }]} relatedWorks={[{ id: "r1", relatedItemId: "item-r1", title: "関連", relation: "reference" }]} />);
    const visibleHeadings = () => screen.getAllByRole("heading", { level: 3 }).map((node) => node.textContent);

    expect(visibleHeadings()).toEqual(["概要", "構成"]);

    fireEvent.click(screen.getByRole("button", { name: "関連作品" }));

    expect(visibleHeadings()).toEqual(["概要", "関連作品"]);
  });

  it("renders group episode counts from data", () => {
    render(<DetailMain overview="概要" groups={[{ id: "g1", label: "S1", episodes: [{ id: "e1", number: "01", title: "ep1" }, { id: "e2", number: "02", title: "ep2" }] }]} />);
    expect(screen.getByText("ep1")).toBeInTheDocument();
    expect(screen.getByText("ep2")).toBeInTheDocument();
  });

  it("removes mylist item via rail section callback", () => {
    const onRemove = vi.fn();
    render(<DetailRail title="作品" facts={[<span key="fact">fact</span>]} mylists={[{ id: "m1", label: "お気に入り", actionLabel: "解除" }]} onRemoveMylist={onRemove} />);
    fireEvent.click(screen.getByRole("button", { name: "解除" }));
    expect(onRemove).toHaveBeenCalledWith("m1");
  });

  it("groups theme songs by type and renders streaming links", () => {
    render(
      <ThemeSongList
        groups={[
          {
            type: "op",
            label: "OP",
            songs: [
              {
                id: "its-1",
                title: "残酷な天使のテーゼ",
                sub: "高橋洋子 ・ 作曲: 佐藤英敏",
                note: null,
                links: [{ id: "l1", type: "youtube", url: "https://example.com/mv", label: "MV" }],
              },
            ],
          },
          {
            type: "ed",
            label: "ED",
            songs: [
              { id: "its-2", title: "FLY ME TO THE MOON", sub: "CLAIRE", note: "TVサイズ", links: [] },
            ],
          },
        ]}
      />,
    );

    expect(screen.getByRole("heading", { level: 4, name: "OP" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { level: 4, name: "ED" })).toBeInTheDocument();
    expect(screen.getByText("残酷な天使のテーゼ")).toBeInTheDocument();
    expect(screen.getByText("高橋洋子 ・ 作曲: 佐藤英敏")).toBeInTheDocument();
    expect(screen.getByText("TVサイズ")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "MV" })).toHaveAttribute("href", "https://example.com/mv");
  });

  it("falls back to the link type label when a link has no label", () => {
    render(
      <ThemeSongList
        groups={[
          {
            type: "op",
            label: "OP",
            songs: [
              {
                id: "its-1",
                title: "曲名",
                sub: "",
                note: null,
                links: [{ id: "l1", type: "apple_music", url: "https://example.com/am", label: null }],
              },
            ],
          },
        ]}
      />,
    );

    expect(screen.getByRole("link", { name: "Apple Music" })).toHaveAttribute("href", "https://example.com/am");
  });

  it("removes a theme song via the entry action callback", () => {
    const onAction = vi.fn();
    render(
      <ThemeSongList
        groups={[
          {
            type: "op",
            label: "OP",
            songs: [{ id: "its-1", title: "曲名", sub: "", note: null, links: [], actionLabel: "解除", onAction }],
          },
        ]}
        footerAction={<button type="button">テーマソングを追加</button>}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "解除" }));
    expect(onAction).toHaveBeenCalledWith("its-1");
    expect(screen.getByRole("button", { name: "テーマソングを追加" })).toBeInTheDocument();
  });

  it("renders the theme song tab from DetailMain", () => {
    render(
      <DetailMain
        overview="概要"
        themeSongs={[{ type: "op", label: "OP", songs: [{ id: "its-1", title: "残酷な天使のテーゼ", sub: "高橋洋子", note: null, links: [] }] }]}
      />,
    );

    expect(screen.getByRole("button", { name: "テーマソング" })).toBeInTheDocument();
    expect(screen.getByText("残酷な天使のテーゼ")).toBeInTheDocument();
  });

  it("matches detail section matrix", () => {
    expect(detailSectionMatrix.anime).toEqual({ propertyList: false, groupList: true, staffList: true, castList: true, themeSongs: true, streaming: true, images: true });
    expect(detailSectionMatrix.movie).toEqual({ propertyList: true, groupList: false, staffList: true, castList: true, themeSongs: true, streaming: true, images: true });
    expect(detailSectionMatrix.drama).toEqual({ propertyList: true, groupList: true, staffList: true, castList: true, themeSongs: true, streaming: true, images: true });
    expect(detailSectionMatrix.manga).toEqual({ propertyList: true, groupList: true, staffList: false, castList: false, themeSongs: false, streaming: false, images: true });
    expect(detailSectionMatrix.novel).toEqual({ propertyList: true, groupList: true, staffList: false, castList: false, themeSongs: false, streaming: false, images: true });
    expect(detailSectionMatrix.game).toEqual({ propertyList: true, groupList: false, staffList: false, castList: false, themeSongs: false, streaming: false, images: true });
    expect(detailSectionMatrix.academic_book).toEqual({ propertyList: true, groupList: false, staffList: false, castList: false, themeSongs: false, streaming: false, images: true });
    expect(detailSectionMatrix.paper).toEqual({ propertyList: true, groupList: false, staffList: false, castList: false, themeSongs: false, streaming: false, images: true });
  });
});
