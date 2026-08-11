import { Link } from "react-router-dom";
import { toast } from "sonner";
import { FiHeart, FiStar, FiX } from "react-icons/fi";
import { cn } from "@/lib/cn";
import { useQuickViewData } from "@/hooks/useQuickViewData";
import { TagList } from "./TagList";

type QuickViewStatus = "not_started" | "in_progress" | "completed";

const STATUS_LABELS: Record<QuickViewStatus, string> = {
  not_started: "未着手",
  in_progress: "視聴中",
  completed: "視聴済",
};

const MEDIA_TYPE_LABELS: Record<string, string> = {
  anime: "Anime",
  movie: "Movie",
  drama: "Drama",
  manga: "Manga",
  novel: "Novel",
  game: "Game",
  academic_book: "Academic Book",
  paper: "Paper",
};

function formatDate(value: string | null | undefined) {
  if (!value) return null;
  return value.slice(0, 10);
}

export function QuickViewSheet({
  itemId,
  onClose,
  onDeleted,
}: {
  itemId: string | null;
  onClose: () => void;
  onDeleted: () => void;
}) {
  const quickView = useQuickViewData(itemId ?? undefined);
  const open = Boolean(itemId);
  const item = quickView.item;

  async function handleDelete() {
    if (!itemId) return;
    if (!window.confirm("この作品を削除しますか？この操作は取り消せません。")) return;
    try {
      await quickView.deleteItem();
      toast.success("作品を削除しました。");
      onDeleted();
      onClose();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "削除に失敗しました。");
    }
  }

  return (
    <>
      <div className={cn("sheet-backdrop", open && "open")} onClick={onClose} />
      <aside className={cn("quick-view-sheet", open && "open")} aria-hidden={!open}>
        <button type="button" className="qv-close" aria-label="閉じる" onClick={onClose}>
          <FiX className="icon" />
        </button>
        {item ? (
          <>
            <div className="qv-header">
              {item.cover_image_url ? <img src={item.cover_image_url} alt="" /> : null}
              <div className="qv-header-info">
                <span className="qv-pill">{MEDIA_TYPE_LABELS[item.media_type] ?? item.media_type}</span>
                <h2>{item.title}</h2>
                <p className="qv-original">{item.original_title || "原題未登録"}</p>
              </div>
            </div>
            <div className="qv-body">
              <div className="qv-facts">
                <div className="qv-fact">
                  <span className="qv-fact-label">ステータス</span>
                  <div className="qv-status-group" role="group" aria-label="ステータス">
                    {(Object.keys(STATUS_LABELS) as QuickViewStatus[]).map((status) => (
                      <button
                        key={status}
                        type="button"
                        className={cn("qv-status-chip", item.status === status && "is-active")}
                        onClick={() => void quickView.updateStatus(status)}
                      >
                        {STATUS_LABELS[status]}
                      </button>
                    ))}
                  </div>
                </div>
                <div className="qv-fact">
                  <span className="qv-fact-label">評価</span>
                  <div className="qv-rating" data-value={item.rating ?? 0}>
                    {[1, 2, 3, 4, 5].map((star) => (
                      <button
                        key={star}
                        type="button"
                        className="qv-star"
                        onClick={() => void quickView.updateRating(star)}
                      >
                        <FiStar className={cn("icon", (item.rating ?? 0) >= star && "is-full")} />
                      </button>
                    ))}
                  </div>
                </div>
                <div className="qv-fact">
                  <span className="qv-fact-label">視聴日</span>
                  <span className="qv-fact-value">{formatDate(item.consumed_date) ?? "視聴日未登録"}</span>
                </div>
                <div className="qv-fact">
                  <span className="qv-fact-label">お気に入り</span>
                  <button
                    type="button"
                    className="qv-fav-toggle"
                    aria-pressed={item.is_favorite}
                    onClick={() => void quickView.updateFavorite(!item.is_favorite)}
                  >
                    <FiHeart className="icon" />
                    <span>{item.is_favorite ? "登録済み" : "未登録"}</span>
                  </button>
                </div>
                <div className="qv-fact">
                  <span className="qv-fact-label">公開日</span>
                  <span className="qv-fact-value">{formatDate(item.release_date) ?? "公開日未登録"}</span>
                </div>
                <div className="qv-fact">
                  <span className="qv-fact-label">ソース</span>
                  <span className="qv-fact-value">{item.source === "api" ? "自動取得" : "手動登録"}</span>
                </div>
              </div>

              <div>
                <p className="qv-section-title">タグ</p>
                <TagList
                  kind="tag"
                  items={item.tags}
                  onAdd={(name) => void quickView.addTag(name)}
                  onRemove={(tagId) => void quickView.removeTag(tagId)}
                />
              </div>
              <div>
                <p className="qv-section-title">カテゴリ</p>
                <TagList
                  kind="category"
                  items={item.categories}
                  onAdd={(name) => void quickView.addCategory(name)}
                  onRemove={(categoryId) => void quickView.removeCategory(categoryId)}
                />
              </div>
              <div>
                <p className="qv-section-title">マイリスト</p>
                <div className="qv-mylist-row">
                  {quickView.mylists.length ? (
                    <div className="qv-pills">
                      {quickView.mylists.map((mylist) => (
                        <span key={mylist.id} className="qv-pill">
                          {mylist.name}
                          <button
                            type="button"
                            className="qv-pill-remove"
                            aria-label="マイリストから削除"
                            onClick={() => void quickView.removeMylist(mylist.id)}
                          >
                            ×
                          </button>
                        </span>
                      ))}
                    </div>
                  ) : (
                    <span className="qv-fact-value">所属しているマイリストはありません</span>
                  )}
                  <button
                    type="button"
                    className="qv-pill-add"
                    onClick={() => toast.info("マイリスト追加UIは今後の対応予定です。")}
                  >
                    ＋ マイリストに追加
                  </button>
                </div>
              </div>
              <div>
                <p className="qv-section-title">概要</p>
                <p className="qv-desc">{item.description || "概要は未登録です。編集ページから追記できます。"}</p>
              </div>
            </div>
            <div className="qv-footer">
              <Link className="btn-secondary" to={`/media/${item.id}`} onClick={onClose}>
                編集する
              </Link>
              <button type="button" className="btn-danger" onClick={() => void handleDelete()}>
                削除する
              </button>
            </div>
          </>
        ) : null}
      </aside>
    </>
  );
}
