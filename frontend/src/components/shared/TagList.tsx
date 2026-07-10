import { useEffect, useRef, useState } from "react";
import { FiFolder, FiPlus, FiTag, FiTrash2 } from "react-icons/fi";

export type TagListItem = { id: string; name: string };

export function TagList({
  kind,
  items,
  onAdd,
  onRemove,
}: {
  kind: "tag" | "category";
  items: TagListItem[];
  onAdd?: (name: string) => void;
  onRemove?: (id: string) => void;
}) {
  const [isAdding, setIsAdding] = useState(false);
  const [value, setValue] = useState("");
  const blurTimer = useRef<number | null>(null);
  const Icon = kind === "tag" ? FiTag : FiFolder;

  useEffect(() => () => {
    if (blurTimer.current) {
      window.clearTimeout(blurTimer.current);
    }
  }, []);

  function closeInput() {
    setIsAdding(false);
    setValue("");
  }

  function submit() {
    const trimmed = value.trim();
    if (!trimmed) {
      closeInput();
      return;
    }
    onAdd?.(trimmed);
    closeInput();
  }

  return (
    <div className="prop-taglist">
      {items.map((item) => (
        <span key={item.id} className="tag-pill">
          {item.name}
          <button type="button" className="tag-remove" onClick={() => onRemove?.(item.id)} aria-label="削除">
            <FiTrash2 className="icon" />
          </button>
        </span>
      ))}
      {isAdding ? (
        <input
          className="tag-add-input"
          autoFocus
          value={value}
          onChange={(event) => setValue(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") submit();
            if (event.key === "Escape") closeInput();
          }}
          onBlur={() => {
            blurTimer.current = window.setTimeout(closeInput, 100);
          }}
          placeholder={kind === "tag" ? "タグ名を入力してEnter" : "カテゴリ名を入力してEnter"}
        />
      ) : (
        <button type="button" className="tag-add-trigger" onClick={() => setIsAdding(true)}>
          <Icon className="icon" />
          <FiPlus className="icon" />
          {kind === "tag" ? "タグを追加" : "カテゴリを追加"}
        </button>
      )}
    </div>
  );
}
