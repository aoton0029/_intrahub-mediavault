import { fireEvent, render, screen } from "@testing-library/react";
import { detailSectionMatrix } from "@/config/detailSections";
import { DetailLayout, DetailMain, DetailRail } from "./DetailLayout";

describe("detail components", () => {
  it("renders rail and main slots", () => {
    render(<DetailLayout rail={<div>rail</div>} main={<div>main</div>} />);
    expect(screen.getByText("rail")).toBeInTheDocument();
    expect(screen.getByText("main")).toBeInTheDocument();
  });

  it("renders optional sections in canonical order", () => {
    render(<DetailMain overview="概要" groups={[{ id: "g1", label: "S1", episodes: [{ id: "e1", number: "01", title: "ep" }] }]} relatedWorks={[{ id: "r1", title: "関連", relation: "reference" }]} />);
    const headings = screen.getAllByRole("heading", { level: 3 }).map((node) => node.textContent);
    expect(headings).toEqual(["概要", "構成", "関連作品"]);
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

  it("matches detail section matrix", () => {
    expect(detailSectionMatrix.anime).toEqual({ propertyList: false, groupList: true, staffList: true, streaming: true });
    expect(detailSectionMatrix.movie).toEqual({ propertyList: true, groupList: false, staffList: true, streaming: true });
    expect(detailSectionMatrix.drama).toEqual({ propertyList: true, groupList: true, staffList: true, streaming: true });
    expect(detailSectionMatrix.manga).toEqual({ propertyList: true, groupList: true, staffList: false, streaming: false });
    expect(detailSectionMatrix.novel).toEqual({ propertyList: true, groupList: true, staffList: false, streaming: false });
    expect(detailSectionMatrix.game).toEqual({ propertyList: true, groupList: false, staffList: false, streaming: false });
    expect(detailSectionMatrix.academic_book).toEqual({ propertyList: true, groupList: false, staffList: false, streaming: false });
    expect(detailSectionMatrix.paper).toEqual({ propertyList: true, groupList: false, staffList: false, streaming: false });
  });
});
