import { useEffect, useRef, useState } from "react";
import { FiCalendar } from "react-icons/fi";

export function ConsumedDateEditor({
  value,
  onChange,
  label = "視聴日",
  emptyLabel = "視聴日未登録",
}: {
  value: string | null;
  onChange: (value: string | null) => void;
  label?: string;
  emptyLabel?: string;
}) {
  const [open, setOpen] = useState(false);
  const [draft, setDraft] = useState(value ?? "");
  const rootRef = useRef<HTMLDivElement>(null);

  useEffect(() => setDraft(value ?? ""), [value]);

  useEffect(() => {
    function handleClick(event: MouseEvent) {
      if (!rootRef.current?.contains(event.target as Node)) {
        setOpen(false);
      }
    }

    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, []);

  return (
    <div className="consumed-date-editor" ref={rootRef}>
      <button type="button" className="meta-item" data-consumed-date-trigger onClick={() => setOpen((current) => !current)}>
        <FiCalendar className="icon" />
        <span className="label">{value ? `${label}: ${value}` : emptyLabel}</span>
      </button>
      {open ? (
        <div className="consumed-date-popover">
          <input type="date" value={draft} onChange={(event) => setDraft(event.target.value)} />
          <div className="consumed-date-actions">
            <button
              type="button"
              className="btn btn-ghost btn-sm"
              onClick={() => {
                onChange(null);
                setOpen(false);
              }}
            >
              クリア
            </button>
            <button
              type="button"
              className="btn btn-accent btn-sm"
              onClick={() => {
                onChange(draft || null);
                setOpen(false);
              }}
            >
              保存
            </button>
          </div>
        </div>
      ) : null}
    </div>
  );
}
