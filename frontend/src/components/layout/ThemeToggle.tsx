import { FiMoon, FiSun } from "react-icons/fi";
import { useTheme } from "@/hooks/useTheme";

export function ThemeToggle() {
  const { theme, toggle } = useTheme();

  return (
    <button type="button" className="theme-toggle" onClick={toggle} data-theme-toggle>
      {theme === "light" ? <FiMoon className="icon" /> : <FiSun className="icon" />}
      <span>{theme === "light" ? "ダークモードに切替" : "ライトモードに切替"}</span>
    </button>
  );
}
