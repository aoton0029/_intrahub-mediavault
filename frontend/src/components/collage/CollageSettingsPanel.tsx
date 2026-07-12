export function CollageSettingsPanel({
  rows,
  cols,
  outputWidth,
  outputHeight,
  showTitles,
  onRowsChange,
  onColsChange,
  onOutputWidthChange,
  onOutputHeightChange,
  onShowTitlesChange,
}: {
  rows: number;
  cols: number;
  outputWidth: number;
  outputHeight: number;
  showTitles: boolean;
  onRowsChange: (value: number) => void;
  onColsChange: (value: number) => void;
  onOutputWidthChange: (value: number) => void;
  onOutputHeightChange: (value: number) => void;
  onShowTitlesChange: (value: boolean) => void;
}) {
  return (
    <div className="collage-settings">
      <label>
        縦（行数）
        <input
          type="number"
          min={1}
          max={10}
          value={rows}
          onChange={(event) => onRowsChange(Number(event.target.value))}
        />
      </label>
      <label>
        横（列数）
        <input
          type="number"
          min={1}
          max={10}
          value={cols}
          onChange={(event) => onColsChange(Number(event.target.value))}
        />
      </label>
      <label>
        保存画像の幅（px）
        <input
          type="number"
          min={100}
          step={100}
          value={outputWidth}
          onChange={(event) => onOutputWidthChange(Number(event.target.value))}
        />
      </label>
      <label>
        保存画像の高さ（px）
        <input
          type="number"
          min={100}
          step={100}
          value={outputHeight}
          onChange={(event) => onOutputHeightChange(Number(event.target.value))}
        />
      </label>
      <label className="collage-settings-checkbox">
        <input
          type="checkbox"
          checked={showTitles}
          onChange={(event) => onShowTitlesChange(event.target.checked)}
        />
        作品名を表示する
      </label>
    </div>
  );
}
