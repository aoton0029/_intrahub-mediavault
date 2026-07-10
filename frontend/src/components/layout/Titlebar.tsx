import type { ReactNode } from "react";
import { Link } from "react-router-dom";

export type BreadcrumbItem = {
  label: string;
  to?: string;
};

type TitlebarProps = {
  title?: ReactNode;
  breadcrumbs?: BreadcrumbItem[];
  actions?: ReactNode;
};

export function Titlebar({ title, breadcrumbs, actions }: TitlebarProps) {
  return (
    <div className="titlebar">
      <div>
        {breadcrumbs?.length ? (
          <div className="breadcrumb">
            {breadcrumbs.map((item, index) => (
              <span key={`${item.label}-${index}`}>
                {index > 0 ? " / " : null}
                {item.to ? <Link to={item.to}>{item.label}</Link> : <span>{item.label}</span>}
              </span>
            ))}
          </div>
        ) : null}
        {title ? <h1>{title}</h1> : null}
      </div>
      {actions ? <div>{actions}</div> : null}
    </div>
  );
}
