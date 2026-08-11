import { cn } from "@/lib/cn";
import type { CollageSelectedItem } from "@/hooks/useCollageBuilder";

export function CollageGrid({
  rows,
  cols,
  cells,
  activeCellIndex,
  showTitles,
  onSelectCell,
}: {
  rows: number;
  cols: number;
  cells: (CollageSelectedItem | null)[];
  activeCellIndex: number | null;
  showTitles: boolean;
  onSelectCell: (index: number) => void;
}) {
  return (
    <div className="collage-preview">
      <div
        className="collage-grid"
        style={{
          gridTemplateColumns: `repeat(${cols}, 1fr)`,
          gridTemplateRows: `repeat(${rows}, 1fr)`,
          aspectRatio: `${cols} / ${rows}`,
        }}
      >
        {cells.map((cell, index) => (
          <button
            key={index}
            type="button"
            className={cn("collage-cell", activeCellIndex === index && "active")}
            onClick={() => onSelectCell(index)}
          >
            {cell?.imageUrl ? <img src={cell.imageUrl} alt={cell.title} /> : <span className="collage-cell-empty">{index + 1}</span>}
          </button>
        ))}
      </div>

      {showTitles ? (
        <ol className="collage-title-list">
          {cells.map((cell, index) => (
            <li key={index}>
              <span className="idx">{index + 1}.</span>
              {cell ? cell.title : <span className="empty">未設定</span>}
            </li>
          ))}
        </ol>
      ) : null}
    </div>
  );
}
