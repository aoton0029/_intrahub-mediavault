import type { ReactNode } from 'react';
import { createBrowserRouter, Link } from 'react-router-dom';
import { AppShell } from '@/components/layout/AppShell';
import type { AppRouteHandle, BreadcrumbItem } from '@/types/ui';
import {
  AcademicBookDetailPage,
  AcademicBookSearchPage,
  AcademicBooksPage,
  AnimeDetailPage,
  DramaDetailPage,
  GameDetailPage,
  GeneralMediaPage,
  GeneralMediaSearchPage,
  HomePage,
  MangaDetailPage,
  MovieDetailPage,
  MylistsPage,
  MylistDetailPage,
  NovelDetailPage,
  PaperDetailPage,
  PaperFormPage,
  PapersPage,
  SettingsPage,
} from './pages';

function handle(title: string, breadcrumb: BreadcrumbItem[], actions?: ReactNode): AppRouteHandle {
  return { title, breadcrumb, actions };
}

const editAction = (to: string) => (
  <Link className="btn btn-accent" to={to}>
    編集する
  </Link>
);

export const router = createBrowserRouter([
  {
    path: '/',
    element: <AppShell />,
    children: [
      { index: true, element: <HomePage />, handle: handle('ホーム', [{ label: 'Dashboard' }]) },
      {
        path: 'media',
        element: <GeneralMediaPage />,
        handle: handle('一般メディア一覧', [{ label: '一般メディア' }], <Link className="btn btn-accent" to="/media/search">＋ 作品を追加</Link>),
      },
      {
        path: 'academic-books',
        element: <AcademicBooksPage />,
        handle: handle('学術書・専門書一覧', [{ label: '学術書・専門書' }], <Link className="btn btn-accent" to="/academic-books/search">＋ 作品を追加</Link>),
      },
      {
        path: 'papers',
        element: <PapersPage />,
        handle: handle('論文・文献一覧', [{ label: '論文・文献' }], <Link className="btn btn-accent" to="/papers/new">＋ 作品を追加</Link>),
      },
      { path: 'mylists', element: <MylistsPage />, handle: handle('マイリスト', [{ label: 'マイリスト' }]) },
      {
        path: 'media/search',
        element: <GeneralMediaSearchPage />,
        handle: handle('一般メディア検索', [{ label: '一般メディア', to: '/media' }, { label: '検索' }]),
      },
      {
        path: 'academic-books/search',
        element: <AcademicBookSearchPage />,
        handle: handle('学術書検索', [{ label: '学術書・専門書', to: '/academic-books' }, { label: '検索' }]),
      },
      {
        path: 'papers/new',
        element: <PaperFormPage />,
        handle: handle('論文登録フォーム', [{ label: '論文・文献', to: '/papers' }, { label: '新規登録' }]),
      },
      {
        path: 'mylists/:mylistId',
        element: <MylistDetailPage />,
        handle: handle('マイリスト詳細', [{ label: 'マイリスト', to: '/mylists' }, { label: '詳細' }]),
      },
      {
        path: 'media/anime/:itemId',
        element: <AnimeDetailPage />,
        handle: handle('アニメ詳細', [{ label: '一般メディア', to: '/media' }, { label: 'アニメ' }], editAction('/media/search')),
      },
      {
        path: 'media/movies/:itemId',
        element: <MovieDetailPage />,
        handle: handle('映画詳細', [{ label: '一般メディア', to: '/media' }, { label: '映画' }], editAction('/media/search')),
      },
      {
        path: 'media/dramas/:itemId',
        element: <DramaDetailPage />,
        handle: handle('ドラマ詳細', [{ label: '一般メディア', to: '/media' }, { label: 'ドラマ' }], editAction('/media/search')),
      },
      {
        path: 'media/manga/:itemId',
        element: <MangaDetailPage />,
        handle: handle('漫画詳細', [{ label: '一般メディア', to: '/media' }, { label: '漫画' }], editAction('/media/search')),
      },
      {
        path: 'media/novels/:itemId',
        element: <NovelDetailPage />,
        handle: handle('小説詳細', [{ label: '一般メディア', to: '/media' }, { label: '小説' }], editAction('/media/search')),
      },
      {
        path: 'media/games/:itemId',
        element: <GameDetailPage />,
        handle: handle('ゲーム詳細', [{ label: '一般メディア', to: '/media' }, { label: 'ゲーム' }], editAction('/media/search')),
      },
      {
        path: 'academic-books/:itemId',
        element: <AcademicBookDetailPage />,
        handle: handle('学術書詳細', [{ label: '学術書・専門書', to: '/academic-books' }], editAction('/academic-books/search')),
      },
      {
        path: 'papers/:itemId',
        element: <PaperDetailPage />,
        handle: handle('論文詳細', [{ label: '論文・文献', to: '/papers' }], editAction('/papers/new')),
      },
      { path: 'settings', element: <SettingsPage />, handle: handle('設定', [{ label: '設定' }]) },
    ],
  },
]);
