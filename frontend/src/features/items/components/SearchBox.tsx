interface SearchBoxProps {
  value: string;
  onChange: (value: string) => void;
}

export function SearchBox({ value, onChange }: SearchBoxProps) {
  return (
    <div className="flex min-w-[220px] items-center gap-1.5 rounded-app border border-border bg-bg-input px-2.5 py-1.5 text-xs text-text-faint">
      🔍
      <input
        type="text"
        aria-label="タイトルで検索"
        placeholder="タイトルで検索…"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="w-full bg-transparent text-text-primary outline-none"
      />
    </div>
  );
}
