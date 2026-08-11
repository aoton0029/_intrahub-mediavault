import { FiCheckCircle, FiCircle, FiEye, FiFolderPlus, FiHeart, FiPlayCircle, FiTrash2 } from "react-icons/fi";
import { useOutsideClick } from "@/hooks/useOutsideClick";
import { cn } from "@/lib/cn";

export type ContextMenuStatus = "not_started" | "in_progress" | "completed";

export type MediaContextMenuTarget = {
  id: string;
  title: string;
  status: ContextMenuStatus;
  favorite: boolean;
  /** Computed once at open-time from the triggering button's bounding rect. */
  position: { top: number; right: number };
};

const STATUS_OPTIONS: { value: ContextMenuStatus; label: string; icon: typeof FiCircle }[] = [
  { value: "not_started", label: "未着手", icon: FiCircle },
  { value: "in_progress", label: "視聴中", icon: FiPlayCircle },
  { value: "completed", label: "視聴済", icon: FiCheckCircle },
];

export function MediaContextMenu({
  target,
  onClose,
  onQuickView,
  onToggleFavorite,
  onSetStatus,
  onAddToMylist,
  onDelete,
}: {
  target: MediaContextMenuTarget | null;
  onClose: () => void;
  onQuickView: (id: string) => void;
  onToggleFavorite: (id: string, next: boolean) => void;
  onSetStatus: (id: string, status: ContextMenuStatus) => void;
  onAddToMylist: (id: string) => void;
  onDelete: (id: string) => void;
}) {
  const rootRef = useOutsideClick<HTMLDivElement>(onClose, Boolean(target));

  if (!target) {
    return null;
  }

  return (
    <div className="card-menu open" style={{ top: target.position.top, right: target.position.right }} ref={rootRef}>
      <button type="button" className="dropdown-item" onClick={() => { onQuickView(target.id); onClose(); }}>
        <FiEye className="icon" />
        詳細を見る
      </button>
      <button
        type="button"
        className="dropdown-item"
        onClick={() => { onToggleFavorite(target.id, !target.favorite); onClose(); }}
      >
        <FiHeart className="icon" />
        {target.favorite ? "お気に入りを解除" : "お気に入りに追加"}
      </button>
      <div className="dropdown-sep" />
      <p className="dropdown-group-label">ステータスを変更</p>
      {STATUS_OPTIONS.map(({ value, label, icon: Icon }) => (
        <button
          key={value}
          type="button"
          className={cn("dropdown-item", target.status === value && "is-selected")}
          onClick={() => { onSetStatus(target.id, value); onClose(); }}
        >
          <Icon className="icon" />
          {label}
        </button>
      ))}
      <div className="dropdown-sep" />
      <button type="button" className="dropdown-item" onClick={() => { onAddToMylist(target.id); onClose(); }}>
        <FiFolderPlus className="icon" />
        マイリストに追加
      </button>
      <div className="dropdown-sep" />
      <button type="button" className="dropdown-item danger" onClick={() => { onDelete(target.id); onClose(); }}>
        <FiTrash2 className="icon" />
        削除する
      </button>
    </div>
  );
}
