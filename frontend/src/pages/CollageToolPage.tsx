import { useState } from "react";
import { CollageSettingsPanel } from "@/components/collage/CollageSettingsPanel";
import { CollageGrid } from "@/components/collage/CollageGrid";
import { ItemPickerPanel } from "@/components/collage/ItemPickerPanel";
import { useCollageBuilder } from "@/hooks/useCollageBuilder";
import type { MediaType } from "@/hooks/useMediaListData";

const MEDIA_TYPE_OPTIONS: { label: string; value: MediaType }[] = [
  { label: "映画", value: "movie" },
  { label: "ドラマ", value: "drama" },
  { label: "アニメ", value: "anime" },
  { label: "漫画", value: "manga" },
  { label: "小説", value: "novel" },
  { label: "ゲーム", value: "game" },
];

export function CollageToolPage() {
  const [mediaType, setMediaType] = useState<MediaType>("movie");
  const builder = useCollageBuilder();

  return (
    <div className="collage-page">
      <div className="collage-toolbar">
        <div className="filter-select">
          メディアを選択
          <select value={mediaType} onChange={(event) => setMediaType(event.target.value as MediaType)}>
            {MEDIA_TYPE_OPTIONS.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        </div>

        <button type="button" className="btn btn-accent" disabled={builder.isExporting} onClick={() => void builder.exportAsImage()}>
          {builder.isExporting ? "保存中…" : "保存する"}
        </button>
      </div>

      <CollageSettingsPanel
        rows={builder.rows}
        cols={builder.cols}
        outputWidth={builder.outputWidth}
        outputHeight={builder.outputHeight}
        showTitles={builder.showTitles}
        onRowsChange={builder.setRows}
        onColsChange={builder.setCols}
        onOutputWidthChange={builder.setOutputWidth}
        onOutputHeightChange={builder.setOutputHeight}
        onShowTitlesChange={builder.setShowTitles}
      />

      <div className="collage-layout">
        <CollageGrid
          rows={builder.rows}
          cols={builder.cols}
          cells={builder.cells}
          activeCellIndex={builder.activeCellIndex}
          showTitles={builder.showTitles}
          onSelectCell={builder.selectCell}
        />

        <ItemPickerPanel mediaType={mediaType} onSelectItem={builder.assignActiveCell} />
      </div>
    </div>
  );
}
