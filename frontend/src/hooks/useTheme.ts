import { useEffect, useState } from 'react';
import type { Theme } from '@/types/ui';

const STORAGE_KEY = 'mediavault-theme';

function getInitialTheme(): Theme {
  if (typeof window === 'undefined') {
    return 'dark';
  }

  const savedTheme = window.localStorage.getItem(STORAGE_KEY);
  return savedTheme === 'light' ? 'light' : 'dark';
}

export function useTheme() {
  const [theme, setTheme] = useState<Theme>(getInitialTheme);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    window.localStorage.setItem(STORAGE_KEY, theme);
  }, [theme]);

  return {
    theme,
    setTheme,
    toggle: () => setTheme((current) => (current === 'dark' ? 'light' : 'dark')),
  };
}
