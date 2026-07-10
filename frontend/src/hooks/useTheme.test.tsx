import { act, renderHook } from "@testing-library/react";
import { THEME_STORAGE_KEY, useTheme } from "./useTheme";

describe("useTheme", () => {
  beforeEach(() => {
    window.localStorage.clear();
    document.documentElement.removeAttribute("data-theme");
  });

  it("defaults to dark when localStorage is empty", () => {
    const { result } = renderHook(() => useTheme());

    expect(result.current.theme).toBe("dark");
    expect(document.documentElement.getAttribute("data-theme")).toBe("dark");
  });

  it("toggle updates localStorage and html attribute", () => {
    const { result } = renderHook(() => useTheme());

    act(() => {
      result.current.toggle();
    });

    expect(result.current.theme).toBe("light");
    expect(window.localStorage.getItem(THEME_STORAGE_KEY)).toBe("light");
    expect(document.documentElement.getAttribute("data-theme")).toBe("light");
  });
});
