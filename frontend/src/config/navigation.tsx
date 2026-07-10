import {
  FiBookOpen,
  FiBookmark,
  FiFileText,
  FiFilm,
  FiGrid,
  FiHome,
  FiMonitor,
  FiSettings,
  FiTv,
} from 'react-icons/fi';
import type { NavSectionConfig } from '@/types/ui';

export const navigationSections: NavSectionConfig[] = [
  {
    label: 'Dashboard',
    items: [{ label: 'ホーム', to: '/', count: 128, icon: FiHome, match: (pathname) => pathname === '/' }],
  },
  {
    label: '一般メディア',
    items: [
      {
        label: 'すべて',
        to: '/media',
        count: 96,
        icon: FiGrid,
        match: (pathname) => pathname === '/media' || pathname.startsWith('/media/'),
      },
      { label: '映画', to: '/media/movies/movie-001', count: 9, indent: true, icon: FiFilm },
      { label: 'ドラマ', to: '/media/dramas/drama-001', count: 0, indent: true, icon: FiTv },
      { label: 'アニメ', to: '/media/anime/anime-001', count: 28, indent: true, icon: FiTv },
      { label: '漫画', to: '/media/manga/manga-001', count: 34, indent: true, icon: FiBookOpen },
      { label: '小説', to: '/media/novels/novel-001', count: 15, indent: true, icon: FiBookOpen },
      { label: 'ゲーム', to: '/media/games/game-001', count: 3, indent: true, icon: FiMonitor },
    ],
  },
  {
    label: 'リサーチ',
    items: [
      { label: '学術書・専門書', to: '/academic-books', count: 21, icon: FiBookOpen },
      { label: '論文・文献', to: '/papers', count: 11, icon: FiFileText },
    ],
  },
  {
    label: 'Collections',
    items: [{ label: 'マイリスト', to: '/mylists', icon: FiBookmark }],
  },
  {
    label: 'System',
    items: [{ label: '設定', to: '/settings', icon: FiSettings }],
  },
];
