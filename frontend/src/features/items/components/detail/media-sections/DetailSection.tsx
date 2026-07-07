import { FiFileText } from 'react-icons/fi';
import type { DetailRow } from './rows';

/** 種別固有情報セクションの共通シェル。行が無い場合は何も描画しない */
export function DetailSection({ rows }: { rows: DetailRow[] }) {
  if (rows.length === 0) return null;

  return (
    <div className="mb-7 max-w-[680px]">
      <h3 className="mb-2.5 flex items-center gap-1.5 border-b border-border-soft pb-1.5 text-xs uppercase tracking-[0.05em] text-text-faint">
        <FiFileText className="h-4 w-4 text-text-faint" />
        種別固有情報
      </h3>
      {rows.map((row) => (
        <div
          key={row.label}
          className="flex items-start justify-between gap-4 border-b border-border-soft py-1.5 text-[12.5px] last:border-b-0"
        >
          <span className="flex-shrink-0 text-text-faint">{row.label}</span>
          {row.href ? (
            <a
              href={row.href}
              target="_blank"
              rel="noreferrer"
              className="break-all text-right text-accent hover:underline"
            >
              {row.value}
            </a>
          ) : (
            <span className="text-right text-text-primary">{row.value}</span>
          )}
        </div>
      ))}
    </div>
  );
}
