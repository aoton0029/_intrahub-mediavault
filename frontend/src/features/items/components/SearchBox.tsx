interface SearchBoxProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  ariaLabel?: string;
}

export function SearchBox({
  value,
  onChange,
  placeholder = 'タイトルで検索…',
  ariaLabel = 'タイトルで検索',
}: SearchBoxProps) {
  return (
    <div className="flex min-w-[220px] items-center gap-1.5 rounded-app border border-border bg-bg-input px-2.5 py-1.5 text-xs text-text-faint">
      🔍
      <input
        type="text"
        aria-label={ariaLabel}
        placeholder={placeholder}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="w-full bg-transparent text-text-primary outline-none"
      />
    </div>
  );
}
