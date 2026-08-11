import { useState } from "react";
import { FiChevronDown, FiImage, FiSliders, FiSmartphone } from "react-icons/fi";
import { useOutsideClick } from "@/hooks/useOutsideClick";
import { cn } from "@/lib/cn";

export type DisplaySettingsDropdownProps = {
  thumbnailOrientation: "horizontal" | "vertical";
  onThumbnailOrientationChange: (value: "horizontal" | "vertical") => void;
  /** undefined = auto (auto-fill grid) */
  columns?: number;
  onColumnsChange: (value: number | undefined) => void;
  minColumns?: number;
  maxColumns?: number;
  showTitle: boolean;
  onShowTitleChange: (value: boolean) => void;
  showRating: boolean;
  onShowRatingChange: (value: boolean) => void;
};

export function DisplaySettingsDropdown({
  thumbnailOrientation,
  onThumbnailOrientationChange,
  columns,
  onColumnsChange,
  minColumns = 2,
  maxColumns = 16,
  showTitle,
  onShowTitleChange,
  showRating,
  onShowRatingChange,
}: DisplaySettingsDropdownProps) {
  const [open, setOpen] = useState(false);
  const rootRef = useOutsideClick<HTMLDivElement>(() => setOpen(false), open);

  return (
    <div className="display-settings" ref={rootRef}>
      <button
        type="button"
        className="chip display-settings-trigger"
        aria-haspopup="true"
        aria-expanded={open}
        onClick={() => setOpen((current) => !current)}
      >
        <FiSliders className="icon" />
        表示設定
        <FiChevronDown className="icon" />
      </button>
      {open ? (
        <div className="display-settings-popover" role="dialog" aria-label="表示設定">
          <div className="settings-section">
            <div className="settings-label">サムネイル</div>
            <div className="segmented-control">
              <button
                type="button"
                className={cn("segmented-option", thumbnailOrientation === "horizontal" && "active")}
                aria-pressed={thumbnailOrientation === "horizontal"}
                onClick={() => onThumbnailOrientationChange("horizontal")}
              >
                <FiImage className="icon" />
                横型
              </button>
              <button
                type="button"
                className={cn("segmented-option", thumbnailOrientation === "vertical" && "active")}
                aria-pressed={thumbnailOrientation === "vertical"}
                onClick={() => onThumbnailOrientationChange("vertical")}
              >
                <FiSmartphone className="icon" />
                縦型
              </button>
            </div>
          </div>
          <div className="settings-section">
            <div className="settings-label">
              <label htmlFor="display-settings-cols">表示数</label>
              <span className="settings-value">{columns ?? "自動"}</span>
            </div>
            <input
              id="display-settings-cols"
              className="settings-slider"
              type="range"
              min={minColumns}
              max={maxColumns}
              step={1}
              value={columns ?? 8}
              aria-label="1行あたりの表示数"
              onChange={(event) => onColumnsChange(Number(event.target.value))}
            />
            <button type="button" className="btn-auto" disabled={columns == null} onClick={() => onColumnsChange(undefined)}>
              自動
            </button>
          </div>
          <div className="settings-section">
            <div className="settings-label">表示項目</div>
            <label className="collage-settings-checkbox">
              <input type="checkbox" checked={showTitle} onChange={(event) => onShowTitleChange(event.target.checked)} />
              作品名
            </label>
            <label className="collage-settings-checkbox">
              <input type="checkbox" checked={showRating} onChange={(event) => onShowRatingChange(event.target.checked)} />
              評価
            </label>
          </div>
        </div>
      ) : null}
    </div>
  );
}
