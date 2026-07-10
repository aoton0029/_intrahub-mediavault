import { Outlet } from 'react-router-dom';
import { Sidebar } from './Sidebar';
import { Titlebar } from './Titlebar';

export function AppShell() {
  return (
    <div className="app-shell">
      <Sidebar />
      <main className="main">
        <Titlebar />
        <div className="content">
          <Outlet />
        </div>
      </main>
    </div>
  );
}
