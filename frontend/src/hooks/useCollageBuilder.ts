import { useState } from "react";
import { toast } from "sonner";
import { downloadBlob, renderCollageToBlob } from "@/lib/collageExport";

export type CollageSelectedItem = {
  id: string;
  title: string;
  imageUrl: string | null;
};

const DEFAULT_ROWS = 3;
const DEFAULT_COLS = 3;
const DEFAULT_OUTPUT_WIDTH = 1200;
const DEFAULT_OUTPUT_HEIGHT = 1200;

function resizeCells(cells: (CollageSelectedItem | null)[], size: number) {
  const next = [...cells];
  next.length = size;
  return next.map((cell) => cell ?? null);
}

export function useCollageBuilder() {
  const [rows, setRowsState] = useState(DEFAULT_ROWS);
  const [cols, setColsState] = useState(DEFAULT_COLS);
  const [outputWidth, setOutputWidth] = useState(DEFAULT_OUTPUT_WIDTH);
  const [outputHeight, setOutputHeight] = useState(DEFAULT_OUTPUT_HEIGHT);
  const [showTitles, setShowTitles] = useState(false);
  const [cells, setCells] = useState<(CollageSelectedItem | null)[]>(
    resizeCells([], DEFAULT_ROWS * DEFAULT_COLS),
  );
  const [activeCellIndex, setActiveCellIndex] = useState<number | null>(0);
  const [isExporting, setIsExporting] = useState(false);

  function setRows(value: number) {
    const nextRows = Math.max(1, value);
    setRowsState(nextRows);
    setCells((prev) => resizeCells(prev, nextRows * cols));
  }

  function setCols(value: number) {
    const nextCols = Math.max(1, value);
    setColsState(nextCols);
    setCells((prev) => resizeCells(prev, rows * nextCols));
  }

  function selectCell(index: number) {
    setActiveCellIndex(index);
  }

  function assignActiveCell(item: CollageSelectedItem) {
    if (activeCellIndex === null) {
      return;
    }

    setCells((prev) => {
      const next = [...prev];
      next[activeCellIndex] = item;
      return next;
    });

    const nextEmptyIndex = cells.findIndex((cell, index) => index !== activeCellIndex && cell === null);
    setActiveCellIndex(nextEmptyIndex === -1 ? activeCellIndex : nextEmptyIndex);
  }

  function clearCell(index: number) {
    setCells((prev) => {
      const next = [...prev];
      next[index] = null;
      return next;
    });
  }

  async function exportAsImage() {
    setIsExporting(true);
    try {
      const blob = await renderCollageToBlob({
        rows,
        cols,
        outputWidth,
        outputHeight,
        showTitles,
        cells: cells.map((cell) => ({ imageUrl: cell?.imageUrl ?? null, title: cell?.title ?? null })),
      });
      downloadBlob(blob, "collage.png");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "画像の保存に失敗しました。");
    } finally {
      setIsExporting(false);
    }
  }

  return {
    rows,
    cols,
    outputWidth,
    outputHeight,
    showTitles,
    cells,
    activeCellIndex,
    isExporting,
    setRows,
    setCols,
    setOutputWidth,
    setOutputHeight,
    setShowTitles,
    selectCell,
    assignActiveCell,
    clearCell,
    exportAsImage,
  };
}
