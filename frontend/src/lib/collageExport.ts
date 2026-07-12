export type CollageCell = {
  imageUrl: string | null;
  title: string | null;
};

export type CollageExportOptions = {
  rows: number;
  cols: number;
  outputWidth: number;
  outputHeight: number;
  showTitles: boolean;
  cells: CollageCell[];
};

const TITLE_COLUMN_WIDTH = 240;
const BACKGROUND_COLOR = "#ffffff";
const TEXT_COLOR = "#1a1a1a";

function loadImage(url: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.crossOrigin = "anonymous";
    img.onload = () => resolve(img);
    img.onerror = () => reject(new Error(`画像の読み込みに失敗しました: ${url}`));
    img.src = url;
  });
}

function truncateText(ctx: CanvasRenderingContext2D, text: string, maxWidth: number) {
  if (ctx.measureText(text).width <= maxWidth) {
    return text;
  }

  let truncated = text;
  while (truncated.length > 0 && ctx.measureText(`${truncated}…`).width > maxWidth) {
    truncated = truncated.slice(0, -1);
  }

  return `${truncated}…`;
}

export async function renderCollageToBlob(options: CollageExportOptions): Promise<Blob> {
  const { rows, cols, outputWidth, outputHeight, showTitles, cells } = options;
  const titleColumnWidth = showTitles ? TITLE_COLUMN_WIDTH : 0;
  const gridWidth = outputWidth - titleColumnWidth;
  const cellWidth = gridWidth / cols;
  const cellHeight = outputHeight / rows;

  const canvas = document.createElement("canvas");
  canvas.width = outputWidth;
  canvas.height = outputHeight;
  const ctx = canvas.getContext("2d");

  if (!ctx) {
    throw new Error("Canvasの初期化に失敗しました");
  }

  ctx.fillStyle = BACKGROUND_COLOR;
  ctx.fillRect(0, 0, outputWidth, outputHeight);

  const images = await Promise.all(
    cells.map((cell) => (cell.imageUrl ? loadImage(cell.imageUrl).catch(() => null) : Promise.resolve(null))),
  );

  cells.forEach((_cell, index) => {
    const row = Math.floor(index / cols);
    const col = index % cols;
    const cellX = col * cellWidth;
    const cellY = row * cellHeight;

    const img = images[index];
    if (img) {
      const scale = Math.min(cellWidth / img.width, cellHeight / img.height);
      const drawWidth = img.width * scale;
      const drawHeight = img.height * scale;
      const drawX = cellX + (cellWidth - drawWidth) / 2;
      const drawY = cellY + (cellHeight - drawHeight) / 2;
      ctx.drawImage(img, drawX, drawY, drawWidth, drawHeight);
    }

    ctx.strokeStyle = "#e0e0e0";
    ctx.strokeRect(cellX, cellY, cellWidth, cellHeight);
  });

  if (showTitles) {
    const titleX = gridWidth + 16;
    const rowHeight = outputHeight / cells.length;

    ctx.fillStyle = TEXT_COLOR;
    ctx.font = "14px sans-serif";
    ctx.textBaseline = "middle";

    cells.forEach((cell, index) => {
      if (!cell.title) {
        return;
      }

      const y = rowHeight * index + rowHeight / 2;
      const text = truncateText(ctx, cell.title, titleColumnWidth - 32);
      ctx.fillText(text, titleX, y);
    });
  }

  return new Promise((resolve, reject) => {
    canvas.toBlob((blob) => {
      if (blob) {
        resolve(blob);
      } else {
        reject(new Error("画像の生成に失敗しました"));
      }
    }, "image/png");
  });
}

export function downloadBlob(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  link.click();
  URL.revokeObjectURL(url);
}
