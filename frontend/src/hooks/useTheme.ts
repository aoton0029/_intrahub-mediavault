import { useEffect, useState } from "react";

export const THEME_STORAGE_KEY = "mediavault-theme";
export type ThemeMode = "dark" | "light";

function getInitialTheme(): ThemeMode {
  if (typeof window === "undefined") {
    return "dark";
  }

  const saved = window.localStorage.getItem(THEME_STORAGE_KEY);
  return saved === "light" ? "light" : "dark";
}

export function useTheme() {
  const [theme, setTheme] = useState<ThemeMode>(getInitialTheme);

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
    window.localStorage.setItem(THEME_STORAGE_KEY, theme);
  }, [theme]);

  return {
    theme,
    setTheme,
    toggle() {
      setTheme((current) => (current === "light" ? "dark" : "light"));
    },
  };
}
