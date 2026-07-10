import { renderHook, act } from '@testing-library/react';
import { useTheme } from './useTheme';

describe('useTheme', () => {
  beforeEach(() => {
    window.localStorage.clear();
    document.documentElement.removeAttribute('data-theme');
  });

  it('uses dark by default and syncs html/localStorage on toggle', () => {
    const { result } = renderHook(() => useTheme());

    expect(result.current.theme).toBe('dark');

    act(() => result.current.toggle());

    expect(result.current.theme).toBe('light');
    expect(document.documentElement.dataset.theme).toBe('light');
    expect(window.localStorage.getItem('mediavault-theme')).toBe('light');
  });
});
