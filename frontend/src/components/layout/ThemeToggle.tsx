import { FiMoon, FiSun } from 'react-icons/fi';
import { useTheme } from '@/hooks/useTheme';

export function ThemeToggle() {
  const { theme, toggle } = useTheme();
  const isLight = theme === 'light';

  return (
    <button type="button" className="theme-toggle" onClick={toggle} data-theme-toggle>
      {isLight ? <FiMoon className="icon" /> : <FiSun className="icon" />}
      <span>{isLight ? 'ダークモードに切替' : 'ライトモードに切替'}</span>
    </button>
  );
}
