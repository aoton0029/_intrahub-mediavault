import { createBrowserRouter } from 'react-router-dom'
import { AppShell } from '@/components/layout/AppShell'
import { PlaceholderPage } from '@/pages/PlaceholderPage'
import { HomePage } from '@/pages/HomePage'
import { GeneralMediaPage } from '@/pages/GeneralMediaPage'
import { SearchAddPage } from '@/pages/SearchAddPage'
import { SettingsPage } from '@/pages/SettingsPage'
import { ItemDetailPage } from '@/pages/ItemDetailPage'

export const router = createBrowserRouter([
  {
    element: <AppShell />,
    children: [
      { path: '/', element: <HomePage /> },
      { path: '/general-media', element: <GeneralMediaPage /> },
      { path: '/items/:id', element: <ItemDetailPage /> },
      { path: '/academic-books', element: <PlaceholderPage title="学術書" /> },
      { path: '/papers', element: <PlaceholderPage title="論文" /> },
      { path: '/mylists', element: <PlaceholderPage title="マイリスト" /> },
      { path: '/tags', element: <PlaceholderPage title="タグ" /> },
      { path: '/categories', element: <PlaceholderPage title="カテゴリ" /> },
      { path: '/staff', element: <PlaceholderPage title="スタッフ" /> },
      { path: '/search-add', element: <SearchAddPage /> },
      { path: '/search-add/manual', element: <PlaceholderPage title="手動で入力する" /> },
      { path: '/settings', element: <SettingsPage /> },
    ],
  },
])
