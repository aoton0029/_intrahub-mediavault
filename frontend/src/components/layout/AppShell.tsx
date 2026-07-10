import type { ReactNode } from "react";
import { Outlet, useMatches } from "react-router-dom";
import { Sidebar } from "./Sidebar";
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

  return (
    <div className="app-shell">
      <Sidebar />
      <main className="main">
        <Titlebar
          title={props.title ?? matchedHandle?.title}
          breadcrumbs={props.breadcrumbs ?? matchedHandle?.breadcrumbs}
          actions={props.actions ?? matchedHandle?.actions}
        />
        <div className="content">{props.children ?? <Outlet />}</div>
      </main>
    </div>
  );
}
