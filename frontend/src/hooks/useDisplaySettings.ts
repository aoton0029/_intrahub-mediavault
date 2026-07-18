import { useCallback, useState } from "react";

const STORAGE_KEY = "mediavault:display-settings";

export type DisplaySettings = {
  thumbnailOrientation: "horizontal" | "vertical";
  /** undefined = auto (auto-fill grid) */
  columns?: number;
};

const DEFAULT_SETTINGS: DisplaySettings = {
  thumbnailOrientation: "horizontal",
  columns: undefined,
};

function loadSettings(): DisplaySettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return DEFAULT_SETTINGS;

    const parsed = JSON.parse(raw) as Partial<DisplaySettings>;
    const columns = Number(parsed.columns);

    return {
      thumbnailOrientation: parsed.thumbnailOrientation === "vertical" ? "vertical" : "horizontal",
      columns: Number.isInteger(columns) && columns >= 2 && columns <= 16 ? columns : undefined,
    };
  } catch {
    return DEFAULT_SETTINGS;
  }
}

/** カード表示設定（サムネイル向き・列数）を localStorage に永続化する。 */
export function useDisplaySettings() {
  const [settings, setSettings] = useState<DisplaySettings>(loadSettings);

  const update = useCallback((updates: Partial<DisplaySettings>) => {
    setSettings((current) => {
      const next = { ...current, ...updates };
      try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
      } catch {
        // ストレージが使えない環境でも表示自体は継続する
      }
      return next;
    });
  }, []);

  const setThumbnailOrientation = useCallback(
    (value: "horizontal" | "vertical") => update({ thumbnailOrientation: value }),
    [update],
  );
  const setColumns = useCallback((value: number | undefined) => update({ columns: value }), [update]);

  return {
    thumbnailOrientation: settings.thumbnailOrientation,
    columns: settings.columns,
    setThumbnailOrientation,
    setColumns,
  };
}
