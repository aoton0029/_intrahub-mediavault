import { useEffect, useRef, useState } from "react";
import { FiCheckCircle, FiChevronDown, FiCircle, FiPlayCircle } from "react-icons/fi";

export type StatusValue = "not_started" | "in_progress" | "completed";

const statusConfig = {
  not_started: { label: "未着手", icon: FiCircle },
  in_progress: { label: "視聴中", icon: FiPlayCircle },
  completed: { label: "視聴済", icon: FiCheckCircle },
} satisfies Record<StatusValue, { label: string; icon: typeof FiCircle }>;

export function StatusSwitcher({ value, onChange }: { value: StatusValue; onChange: (value: StatusValue) => void }) {
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);
  const CurrentIcon = statusConfig[value].icon;

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
    <div className="status-switcher" ref={rootRef}>
      <button type="button" className="meta-item status-trigger" data-status-trigger data-current={value} onClick={() => setOpen((current) => !current)}>
        <CurrentIcon className="icon status-icon" />
        <span className="label">{statusConfig[value].label}</span>
        <FiChevronDown className="icon chevron" />
      </button>
      {open ? (
        <div className="status-popover">
          {(Object.keys(statusConfig) as StatusValue[]).map((status) => {
            const Icon = statusConfig[status].icon;
            return (
              <button
                key={status}
                type="button"
                className="status-option"
                data-status={status}
                onClick={() => {
                  onChange(status);
                  setOpen(false);
                }}
              >
                <Icon className="icon" />
                <span className="label">{statusConfig[status].label}</span>
              </button>
            );
          })}
        </div>
      ) : null}
    </div>
  );
}
