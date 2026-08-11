import { FiGrid, FiList } from "react-icons/fi";
import { cn } from "@/lib/cn";

export type ViewMode = "grid" | "table";

export function ViewModeToggle({ value, onChange }: { value: ViewMode; onChange: (value: ViewMode) => void }) {
  return (
    <div className="view-mode-toggle" role="group" aria-label="表示形式">
      <button
        type="button"
        className={cn(value === "grid" && "active")}
        title="グリッド表示"
        aria-pressed={value === "grid"}
        onClick={() => onChange("grid")}
      >
        <FiGrid className="icon" />
      </button>
      <button
        type="button"
        className={cn(value === "table" && "active")}
        title="テーブル表示"
        aria-pressed={value === "table"}
        onClick={() => onChange("table")}
      >
        <FiList className="icon" />
      </button>
    </div>
  );
}
