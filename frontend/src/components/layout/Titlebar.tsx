import type { ReactNode } from 'react'

export interface TitlebarProps {
  title: string
  breadcrumb?: ReactNode
  action?: ReactNode
}

export function Titlebar({ title, breadcrumb, action }: TitlebarProps) {
  return (
    <div className="sticky top-0 z-5 flex items-center justify-between border-b border-border-soft bg-bg-app px-7 py-3">
      <div>
        {breadcrumb && <div className="mb-0.5 text-xs text-text-faint">{breadcrumb}</div>}
        <h1 className="m-0 font-display text-xl font-semibold">{title}</h1>
      </div>
      {action}
    </div>
  )
}
