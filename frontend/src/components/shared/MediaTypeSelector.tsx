import { FiGrid } from "react-icons/fi";
import { mediaTypeConfig, mediaTypes, type MediaType } from "../../config/mediaTypes";

export function MediaTypeSelector({
  value,
  onChange,
  includeAll,
  compact,
}: {
  value: MediaType | "all";
  onChange: (value: MediaType | "all") => void;
  includeAll?: boolean;
  compact?: boolean;
}) {
  return (
    <div className="media-type-selector" role="tablist" aria-label="メディア種別" data-compact={compact}>
      {includeAll ? (
        <button
          type="button"
          role="tab"
          aria-selected={value === "all"}
          className="media-type-chip"
          data-active={value === "all"}
          title="すべて"
          onClick={() => onChange("all")}
        >
          <FiGrid className="icon" />
          <span className="label">すべて</span>
        </button>
      ) : null}
      {mediaTypes.map((type) => {
        const Icon = mediaTypeConfig[type].icon;
        return (
          <button
            key={type}
            type="button"
            role="tab"
            aria-selected={value === type}
            className="media-type-chip"
            data-active={value === type}
            data-media-type={type}
            title={mediaTypeConfig[type].label}
            onClick={() => onChange(type)}
          >
            <Icon className="icon" />
            <span className="label">{mediaTypeConfig[type].label}</span>
          </button>
        );
      })}
    </div>
  );
}
