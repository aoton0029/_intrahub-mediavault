import { useState } from "react";
import { FiCheck, FiChevronDown, FiGrid } from "react-icons/fi";
import { useOutsideClick } from "@/hooks/useOutsideClick";
import { cn } from "@/lib/cn";
import { mediaTypeConfig, mediaTypes, type MediaType } from "../../config/mediaTypes";

export type MediaTypeDropdownProps = {
  value: MediaType | "all";
  onChange: (value: MediaType | "all") => void;
  includeAll?: boolean;
  options?: MediaType[];
  ariaLabel?: string;
};

export function MediaTypeDropdown({ value, onChange, includeAll, options = mediaTypes, ariaLabel = "メディア種別" }: MediaTypeDropdownProps) {
  const [open, setOpen] = useState(false);
  const rootRef = useOutsideClick<HTMLDivElement>(() => setOpen(false), open);

  const TriggerIcon = value === "all" ? FiGrid : mediaTypeConfig[value].icon;
  const triggerLabel = value === "all" ? "すべて" : mediaTypeConfig[value].label;

  const handleSelect = (next: MediaType | "all") => {
    onChange(next);
    setOpen(false);
  };

  const items: Array<{ value: MediaType | "all"; label: string; Icon: typeof FiGrid }> = [
    ...(includeAll ? [{ value: "all" as const, label: "すべて", Icon: FiGrid }] : []),
    ...options.map((type) => ({ value: type, label: mediaTypeConfig[type].label, Icon: mediaTypeConfig[type].icon })),
  ];

  return (
    <div className="media-type-dropdown" ref={rootRef}>
      <button
        type="button"
        className="chip media-type-dropdown-trigger"
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-label={ariaLabel}
        onClick={() => setOpen((current) => !current)}
      >
        <TriggerIcon className="icon" />
        {triggerLabel}
        <FiChevronDown className="icon" />
      </button>
      {open ? (
        <div className="media-type-dropdown-popover" role="listbox" aria-label={ariaLabel}>
          {items.map(({ value: itemValue, label, Icon }) => (
            <button
              key={itemValue}
              type="button"
              role="option"
              aria-selected={value === itemValue}
              className={cn("media-type-option", value === itemValue && "active")}
              data-media-type={itemValue === "all" ? undefined : itemValue}
              onClick={() => handleSelect(itemValue)}
            >
              <Icon className="icon" />
              <span className="label">{label}</span>
              {value === itemValue ? <FiCheck className="icon check" /> : null}
            </button>
          ))}
        </div>
      ) : null}
    </div>
  );
}
