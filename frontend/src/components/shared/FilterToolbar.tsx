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
  onClick?: () => void;
  onRemove?: () => void;
};

export type FilterOption = {
  label: string;
  value: string;
};

type FilterToolbarProps = {
  chips?: FilterChip[];
  filterOptions?: FilterOption[];
  selectedFilter?: string;
  onFilterChange?: (value: string) => void;
  sortOptions?: FilterOption[];
  selectedSort?: string;
  onSortChange?: (value: string) => void;
  searchValue?: string;
  searchPlaceholder?: string;
  onSearchChange?: (value: string) => void;
};

export function FilterToolbar({
  chips = [],
  filterOptions = [],
  selectedFilter = "",
  onFilterChange,
  sortOptions = [],
  selectedSort = "",
  onSortChange,
  searchValue = "",
  searchPlaceholder = "タイトルで検索...",
  onSearchChange,
}: FilterToolbarProps) {
  return (
    <div className="filter-toolbar">
      <div className="filter-bar">
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
            </button>
          );
        })}

        {filterOptions.length ? (
          <label className="filter-select">
            種別
            <select aria-label="種別" value={selectedFilter} onChange={(event) => onFilterChange?.(event.target.value)}>
              {filterOptions.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </label>
        ) : null}
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
      </div>
    </div>
  );
}
