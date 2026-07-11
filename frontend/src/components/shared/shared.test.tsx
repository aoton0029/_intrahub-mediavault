import { fireEvent, render, screen } from "@testing-library/react";
import { ApiKeyCard } from "./ApiKeyCard";
import { EmptyState } from "./EmptyState";
import { FavoriteToggle } from "./FavoriteToggle";
import { LoadMoreSentinel } from "./LoadMoreSentinel";
import { MediaCard } from "./MediaCard";
import { Modal } from "./Modal";
import { MylistCover } from "./MylistCover";
import { RatingStars } from "./RatingStars";
import { ResourceTabs } from "./ResourceTabs";
import { StatusSwitcher } from "./StatusSwitcher";
import { TagList } from "./TagList";
import { useInfiniteScroll } from "./useInfiniteScroll";

describe("shared components", () => {
  it("RatingStars previews hover and commits click", () => {
    const onChange = vi.fn();
    render(<RatingStars value={2} onChange={onChange} />);
    fireEvent.mouseEnter(screen.getAllByRole("button")[3]);
    expect(screen.getByText("4.0")).toBeInTheDocument();
    fireEvent.mouseLeave(screen.getByText("4.0").closest(".rating-stars")!);
    expect(screen.getByText("2.0")).toBeInTheDocument();
    fireEvent.click(screen.getAllByRole("button")[4]);
    expect(onChange).toHaveBeenCalledWith(5);
  });

  it("FavoriteToggle calls controlled onChange", () => {
    const onChange = vi.fn();
    render(<FavoriteToggle value={false} onChange={onChange} />);
    fireEvent.click(screen.getByRole("button"));
    expect(onChange).toHaveBeenCalledWith(true);
  });

  it("StatusSwitcher toggles, closes and changes value", () => {
    const onChange = vi.fn();
    render(<StatusSwitcher value="not_started" onChange={onChange} />);
    fireEvent.click(screen.getByRole("button"));
    fireEvent.click(screen.getByText("視聴済"));
    expect(onChange).toHaveBeenCalledWith("completed");
  });

  it("StatusSwitcher uses overridden labels", () => {
    const onChange = vi.fn();
    render(<StatusSwitcher value="in_progress" labels={{ in_progress: "読書中", completed: "読了" }} onChange={onChange} />);
    expect(screen.getByRole("button", { name: "読書中" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "読書中" }));
    expect(screen.getByText("読了")).toBeInTheDocument();
  });

  it("TagList supports add and remove", () => {
    const onAdd = vi.fn();
    const onRemove = vi.fn();
    render(<TagList kind="tag" items={[{ id: "1", name: "既存" }]} onAdd={onAdd} onRemove={onRemove} />);
    fireEvent.click(screen.getByText(/タグを追加/));
    const input = screen.getByPlaceholderText("タグ名を入力してEnter");
    fireEvent.change(input, { target: { value: "新規" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onAdd).toHaveBeenCalledWith("新規");
    fireEvent.click(screen.getByLabelText("削除"));
    expect(onRemove).toHaveBeenCalledWith("1");
  });

  it("Modal hides and closes from overlay", () => {
    const onClose = vi.fn();
    const { rerender } = render(<Modal open={false} onClose={onClose} title="確認">body</Modal>);
    expect(screen.queryByText("確認")).not.toBeInTheDocument();
    rerender(<Modal open onClose={onClose} title="確認">body</Modal>);
    fireEvent.click(screen.getByText("確認").closest(".modal-overlay")!);
    expect(onClose).toHaveBeenCalled();
  });

  it("ResourceTabs changes visible panel", () => {
    render(<ResourceTabs tabs={{ links: [{ id: "1", label: "公式", detail: "link" }], files: [{ id: "2", label: "PDF", detail: "pdf" }] }} />);
    fireEvent.click(screen.getByRole("button", { name: /ファイル/ }));
    expect(screen.getByText("PDF")).toBeInTheDocument();
  });

  it("MediaCard disables import button when already imported", () => {
    render(<MediaCard title="星屑" badge="アニメ" variant="search-result" imported />);
    expect(screen.getByRole("button", { name: "取り込み済み" })).toBeDisabled();
  });

  it("MylistCover applies count layout class", () => {
    const { container } = render(<MylistCover count={3} covers={["a", "b", "c"]} />);
    expect(container.firstChild).toHaveClass("n3");
  });

  it("LoadMoreSentinel hook reacts to intersection", () => {
    const onLoadMore = vi.fn();
    const observerState: { callback: ((entries: { isIntersecting: boolean }[]) => void) | null } = { callback: null };
    vi.stubGlobal("IntersectionObserver", class {
      constructor(callback: (entries: { isIntersecting: boolean }[]) => void) {
        observerState.callback = callback;
      }
      observe() {}
      disconnect() {}
    });

    function TestComponent() {
      const ref = useInfiniteScroll(onLoadMore);
      return <div ref={ref}><LoadMoreSentinel /></div>;
    }

    render(<TestComponent />);
    if (observerState.callback) {
      observerState.callback([{ isIntersecting: true }]);
    }
    expect(onLoadMore).toHaveBeenCalled();
  });

  it("EmptyState omits action when absent", () => {
    render(<EmptyState title="空" description="なし" />);
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });

  it("ApiKeyCard renders inline-save controls and saves the entered key", () => {
    const onSave = vi.fn();
    render(<ApiKeyCard provider="TMDB" keyMasked="provider: tmdb ・ 未設定" variant="inline-save" onSave={onSave} />);

    fireEvent.change(screen.getByLabelText("TMDB APIキー"), { target: { value: "secret-key" } });
    fireEvent.click(screen.getByRole("button", { name: "保存" }));

    expect(screen.getByPlaceholderText("APIキーを入力")).toHaveAttribute("type", "password");
    expect(onSave).toHaveBeenCalledWith("secret-key");
  });

  it("ApiKeyCard shows setup hint without input when key is not required", () => {
    render(<ApiKeyCard provider="Jikan" keyMasked="provider: jikan ・ APIキー不要(認証なしで利用可能)" variant="inline-save" requiresKey={false} />);

    expect(screen.queryByPlaceholderText("APIキーを入力")).not.toBeInTheDocument();
    expect(screen.getByText("設定不要")).toHaveClass("field-hint");
  });
});
