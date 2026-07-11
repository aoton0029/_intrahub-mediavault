import type { ReactNode } from "react";
import { useMemo, useState } from "react";
import { Outlet, useMatches } from "react-router-dom";
import type { PageChrome } from "./pageChromeContext";
import { Sidebar } from "./Sidebar";
import { PageChromeContext } from "./pageChromeContext";
import { Titlebar, type BreadcrumbItem } from "./Titlebar";

export type AppRouteHandle = {
  title?: ReactNode;
  breadcrumbs?: BreadcrumbItem[];
  actions?: ReactNode;
};

type AppShellProps = {
  title?: ReactNode;
  breadcrumbs?: BreadcrumbItem[];
  actions?: ReactNode;
  children?: ReactNode;
};

export function AppShell(props: AppShellProps) {
  const matches = useMatches();
  const matchedHandle = [...matches].reverse().find((match) => match.handle)?.handle as AppRouteHandle | undefined;
  const [pageChrome, setPageChrome] = useState<PageChrome | null>(null);
  const contextValue = useMemo(() => ({ setPageChrome }), []);

  return (
    <PageChromeContext.Provider value={contextValue}>
      <div className="app-shell">
        <Sidebar />
        <main className="main">
          <Titlebar
            title={pageChrome?.title ?? props.title ?? matchedHandle?.title}
            breadcrumbs={pageChrome?.breadcrumbs ?? props.breadcrumbs ?? matchedHandle?.breadcrumbs}
            actions={pageChrome?.actions ?? props.actions ?? matchedHandle?.actions}
          />
          <div className="content">{props.children ?? <Outlet />}</div>
        </main>
      </div>
    </PageChromeContext.Provider>
  );
}
