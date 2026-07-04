/**
 * タイトル検索ボックス（.search-box）
 *
 * TASK-0009: 01_home.htmlの `.search-box` 構造に準拠。
 * 入力ごとに即座に useItemFilters の title へ反映する（デバウンスなし、NFR-201）。
 */
interface SearchBoxProps {
  value: string;
  onChange: (value: string) => void;
}

export function SearchBox({ value, onChange }: SearchBoxProps) {
  return (
    <div className="search-box">
      🔍{' '}
      <input
        type="text"
        placeholder="タイトルで検索…"
        aria-label="タイトルで検索"
        value={value}
        onChange={(e) => onChange(e.target.value)}
      />
    </div>
  );
}
