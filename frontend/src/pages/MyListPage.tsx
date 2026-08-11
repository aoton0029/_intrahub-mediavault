import { useState } from "react";
import { Link } from "react-router-dom";
import { toast } from "sonner";
import { FiBookmark, FiMoreVertical } from "react-icons/fi";
import { EmptyState, MylistCover } from "@/components/shared";
import { useOutsideClick } from "@/hooks/useOutsideClick";
import { useMylistsData, type MylistSummary } from "@/hooks/useMylistsData";

type MenuTarget = {
  id: string;
  name: string;
  position: { top: number; right: number };
};

function anchorToMenuPosition(anchor: HTMLElement) {
  const rect = anchor.getBoundingClientRect();
  return { top: rect.bottom + 6, right: window.innerWidth - rect.right };
}

function coverCount(covers: string[]): 1 | 2 | 3 | 4 {
  return (Math.min(Math.max(covers.length, 1), 4) as 1 | 2 | 3 | 4);
}

export function MyListPage() {
  const { mylists, isLoading, isError, createMylist, renameMylist, deleteMylist } = useMylistsData();
  const [menuTarget, setMenuTarget] = useState<MenuTarget | null>(null);
  const menuRef = useOutsideClick<HTMLDivElement>(() => setMenuTarget(null), Boolean(menuTarget));

  async function handleCreate() {
    const name = window.prompt("新しいマイリスト名を入力してください");
    if (!name || !name.trim()) return;
    try {
      await createMylist(name.trim());
      toast.success("マイリストを作成しました。");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "マイリストの作成に失敗しました。");
    }
  }

  async function handleRename(target: MenuTarget) {
    const next = window.prompt("マイリスト名を変更", target.name);
    if (!next || !next.trim()) return;
    try {
      await renameMylist(target.id, next.trim());
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "名前の変更に失敗しました。");
    }
  }

  async function handleDelete(target: MenuTarget) {
    if (!window.confirm(`「${target.name}」を削除しますか？`)) return;
    try {
      await deleteMylist(target.id);
      toast.success("マイリストを削除しました。");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "削除に失敗しました。");
    }
  }

  if (isLoading) {
    return <EmptyState title="読み込み中…" description="マイリストを取得しています。" />;
  }

  if (isError) {
    return <EmptyState title="読み込みに失敗しました" description="時間をおいて再度お試しください。" />;
  }

  return (
    <>
      <div className="titlebar-inline-actions" style={{ display: "flex", justifyContent: "flex-end", padding: "0 0 12px" }}>
        <button type="button" className="btn btn-accent" onClick={() => void handleCreate()}>
          ＋ 新規マイリスト
        </button>
      </div>

      {mylists.length === 0 ? (
        <EmptyState
          title="まだマイリストがありません"
          description="「＋ 新規マイリスト」から作成してください。"
        />
      ) : (
        <div className="mylist-grid">
          {mylists.map((mylist: MylistSummary) => (
            <Link className="mylist-card" key={mylist.id} to={`/mylists/${mylist.id}`}>
              {mylist.cover_urls.length === 0 ? (
                <div className="mylist-cover n1">
                  <div className="mylist-cover-empty">
                    <FiBookmark className="icon" />
                  </div>
                </div>
              ) : (
                <MylistCover count={coverCount(mylist.cover_urls)} covers={mylist.cover_urls} />
              )}
              <button
                type="button"
                className="mylist-card-menu-btn"
                title="メニュー"
                onClick={(event) => {
                  event.preventDefault();
                  event.stopPropagation();
                  setMenuTarget({ id: mylist.id, name: mylist.name, position: anchorToMenuPosition(event.currentTarget) });
                }}
              >
                <FiMoreVertical className="icon" />
              </button>
              <div className="mylist-card-body">
                <p className="mylist-card-name">{mylist.name}</p>
                <p className="mylist-card-meta">{mylist.item_count}件</p>
              </div>
            </Link>
          ))}
        </div>
      )}

      {menuTarget ? (
        <div
          className="card-menu open"
          style={{ top: menuTarget.position.top, right: menuTarget.position.right }}
          ref={menuRef}
        >
          <button
            type="button"
            className="dropdown-item"
            onClick={() => {
              void handleRename(menuTarget);
              setMenuTarget(null);
            }}
          >
            名前を変更
          </button>
          <button
            type="button"
            className="dropdown-item danger"
            onClick={() => {
              void handleDelete(menuTarget);
              setMenuTarget(null);
            }}
          >
            削除する
          </button>
        </div>
      ) : null}
    </>
  );
}
