import type { ReactNode } from "react";
import { FiPlus, FiSearch, FiX } from "react-icons/fi";
import { LuArrowUpDown } from "react-icons/lu";

export type FilterChip = {
  id: string;
  label: string;
  active?: boolean;
  removable?: boolean;
  add?: boolean;
  icon?: ReactNode;
  /** Set to render an icon-only chip (label is still used as the accessible name). */
  iconOnly?: boolean;
  /** Shows a small numeric badge next to the label, e.g. active filter count. */
  count?: number;
  onClick?: () => void;
  onRemove?: () => void;
};

export type FilterOption = {
  label: string;
  value: string;
};

/** 一覧・年別ページ共通のソート選択肢（valueはGET /itemsのsortパラメータに対応） */
export const MEDIA_SORT_OPTIONS: FilterOption[] = [
  { label: "Rating", value: "rating" },
  { label: "Recently added", value: "created_at" },
  { label: "Recently updated", value: "updated_at" },
  { label: "Title", value: "title" },
  { label: "Release date", value: "release_date" },
];

/** ソート未指定時のデフォルト（rating降順） */
export const DEFAULT_MEDIA_SORT = "rating";

type FilterToolbarProps = {
  chips?: FilterChip[];
  /** Rendered before the chips, e.g. a media type dropdown. */
  filterSlot?: ReactNode;
  sortOptions?: FilterOption[];
  selectedSort?: string;
  onSortChange?: (value: string) => void;
  searchValue?: string;
  searchPlaceholder?: string;
  onSearchChange?: (value: string) => void;
  trailing?: ReactNode;
};

export function FilterToolbar({
  chips = [],
  filterSlot,
  sortOptions = [],
  selectedSort = "",
  onSortChange,
  searchValue = "",
  searchPlaceholder = "タイトルで検索...",
  onSearchChange,
  trailing,
}: FilterToolbarProps) {
  return (
    <div className="filter-toolbar">
      <div className="filter-bar">
        {filterSlot}
        {chips.map((chip) => {
          if (chip.removable) {
            return (
              <span key={chip.id} className={`chip${chip.active ? " active" : ""}`}>
                <button type="button" onClick={chip.onClick}>
                  {chip.icon}
                  {chip.label}
                </button>
                <button type="button" aria-label={`${chip.label}を解除`} className="chip-remove" onClick={chip.onRemove}>
                  <FiX className="icon" />
                </button>
              </span>
            );
          }

          if (chip.iconOnly && chip.icon) {
            return (
              <button key={chip.id} type="button" className={`chip${chip.active ? " active" : ""}`} title={chip.label} aria-label={chip.label} onClick={chip.onClick}>
                {chip.icon}
              </button>
            );
          }

          return (
            <button key={chip.id} type="button" className={`chip${chip.active ? " active" : ""}${chip.add ? " chip-add" : ""}`} onClick={chip.onClick}>
              {chip.icon}
              {chip.add ? <FiPlus className="icon" /> : null}
              {chip.label}
              {chip.count ? <span className="filter-count">{chip.count}</span> : null}
            </button>
          );
        })}
      </div>
      <div className="sort-search-group">
        {sortOptions.length ? (
          <label className="sort-select">
            <LuArrowUpDown className="icon" />
            <select aria-label="並び順" value={selectedSort} onChange={(event) => onSortChange?.(event.target.value)}>
              {sortOptions.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </label>
        ) : null}
        <label className="search-box">
          <FiSearch className="icon" />
          <input aria-label="タイトル検索" value={searchValue} placeholder={searchPlaceholder} onChange={(event) => onSearchChange?.(event.target.value)} />
        </label>
        {trailing}
      </div>
    </div>
  );
}
