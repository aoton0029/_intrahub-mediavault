import type { ReactNode } from 'react'
import { Sidebar } from './Sidebar'
import { Titlebar } from './Titlebar'
import { Content } from './Content'

export interface AppShellProps {
  title: string
  breadcrumb?: ReactNode
  action?: ReactNode
  children: ReactNode
}

export function AppShell({ title, breadcrumb, action, children }: AppShellProps) {
  return (
    <div className="collapse:grid-cols-1 grid h-screen grid-cols-[var(--width-sidebar)_1fr]">
      <Sidebar />
      <main className="min-w-0 overflow-y-auto">
        <Titlebar title={title} breadcrumb={breadcrumb} action={action} />
        <Content>{children}</Content>
      </main>
    </div>
  )
}
