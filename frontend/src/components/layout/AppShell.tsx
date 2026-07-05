import { Outlet } from 'react-router-dom'
import { Sidebar } from './Sidebar'
import { Titlebar } from './Titlebar'
import { Content } from './Content'
import { TitlebarProvider } from './TitlebarContext'
import { useTitlebarState } from './useTitlebar'

function AppShellLayout() {
  const { title, breadcrumb, action } = useTitlebarState()
  return (
    <div className="max-collapse:grid-cols-1 grid min-h-screen grid-cols-[var(--width-sidebar)_1fr]">
      <Sidebar />
      <main className="min-w-0">
        <Titlebar title={title} breadcrumb={breadcrumb} action={action} />
        <Content>
          <Outlet />
        </Content>
      </main>
    </div>
  )
}

export function AppShell() {
  return (
    <TitlebarProvider>
      <AppShellLayout />
    </TitlebarProvider>
  )
}
