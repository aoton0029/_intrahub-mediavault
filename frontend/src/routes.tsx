import { createBrowserRouter } from 'react-router-dom'
import { PlaceholderPage } from '@/pages/PlaceholderPage'

export const router = createBrowserRouter([
  { path: '/', element: <PlaceholderPage title="ホーム" /> },
  { path: '/general-media', element: <PlaceholderPage title="一般メディア" /> },
  { path: '/academic-books', element: <PlaceholderPage title="学術書" /> },
  { path: '/papers', element: <PlaceholderPage title="論文" /> },
  { path: '/mylists', element: <PlaceholderPage title="マイリスト" /> },
  { path: '/tags', element: <PlaceholderPage title="タグ" /> },
  { path: '/categories', element: <PlaceholderPage title="カテゴリ" /> },
  { path: '/staff', element: <PlaceholderPage title="スタッフ" /> },
])
