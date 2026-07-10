import { FiArrowDown, FiPlus, FiSearch } from "react-icons/fi";

export type FilterChip = { id: string; label: string; active?: boolean };

export function FilterToolbar({
  chips = [],
  filterOptions = [],
  sortOptions = [],
  searchValue = "",
}: {
  chips?: FilterChip[];
  filterOptions?: string[];
  sortOptions?: string[];
  searchValue?: string;
}) {
  return (
    <div className="filter-toolbar">
      <div className="filter-bar">
        {chips.map((chip) => (
          <button key={chip.id} type="button" className={`chip${chip.active ? " active" : ""}`}>
            {chip.label}
          </button>
        ))}
        <button type="button" className="chip chip-add">
          <FiPlus className="icon" />
          フィルター追加
        </button>
        {filterOptions.length ? (
          <label className="filter-select">
            種別
            <select defaultValue={filterOptions[0]}>
              {filterOptions.map((option) => (
                <option key={option}>{option}</option>
              ))}
            </select>
          </label>
        ) : null}
      </div>
      <div className="sort-search-group">
        <label className="sort-select">
          <FiArrowDown className="icon" />
          <select defaultValue={sortOptions[0]}>
            {sortOptions.map((option) => (
              <option key={option}>{option}</option>
            ))}
          </select>
        </label>
        <label className="search-box">
          <FiSearch className="icon" />
          <input defaultValue={searchValue} placeholder="検索..." />
        </label>
      </div>
    </div>
  );
}
