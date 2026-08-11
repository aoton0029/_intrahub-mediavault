import { useState } from "react";
import { MediaTypeDropdown } from "@/components/shared";
import { CollageSettingsPanel } from "@/components/collage/CollageSettingsPanel";
import { CollageGrid } from "@/components/collage/CollageGrid";
import { ItemPickerPanel } from "@/components/collage/ItemPickerPanel";
import { useCollageBuilder } from "@/hooks/useCollageBuilder";
import type { MediaType } from "@/config/mediaTypes";

export function CollageToolPage() {
  const [mediaType, setMediaType] = useState<MediaType | "all">("all");
  const builder = useCollageBuilder();

  return (
    <div className="collage-page">
      <div className="collage-toolbar">
        <MediaTypeDropdown includeAll value={mediaType} onChange={setMediaType} />
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
        <section className="collage-preview-panel">
          <div className="collage-preview-heading">
            <div className="collage-preview-heading-text">
              <h2>プレビュー</h2>
              <span className="hint">マスをクリックして選択 → 右の作品で「このマスに反映」</span>
            </div>
            <button type="button" className="btn btn-accent" disabled={builder.isExporting} onClick={() => void builder.exportAsImage()}>
              {builder.isExporting ? "保存中…" : "保存する"}
            </button>
          </div>

          <div className="collage-preview-body">
            <CollageGrid
              rows={builder.rows}
              cols={builder.cols}
              cells={builder.cells}
              activeCellIndex={builder.activeCellIndex}
              showTitles={builder.showTitles}
              onSelectCell={builder.selectCell}
            />
          </div>
        </section>

        <ItemPickerPanel mediaType={mediaType} onSelectItem={builder.assignActiveCell} />
      </div>
    </div>
  );
}
