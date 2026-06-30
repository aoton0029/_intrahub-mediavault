import { Outlet } from 'react-router-dom';
import { Sidebar } from '@/components/common/Sidebar';

export default function RootLayout() {
  return (
    <div className="flex min-h-screen">
      <Sidebar />
      <main className="flex-1">
        <Outlet />
      </main>
    </div>
  );
}
