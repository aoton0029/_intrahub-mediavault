import { createContext, type ReactNode } from "react";
import type { BreadcrumbItem } from "./Titlebar";

export type PageChrome = {
  title?: ReactNode;
  breadcrumbs?: BreadcrumbItem[];
  actions?: ReactNode;
};

export type PageChromeContextValue = {
  setPageChrome: (value: PageChrome | null) => void;
};

export const PageChromeContext = createContext<PageChromeContextValue | null>(null);
