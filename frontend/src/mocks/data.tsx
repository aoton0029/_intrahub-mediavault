import { FiCalendar, FiFileText, FiFilm, FiLink2, FiPaperclip, FiPlayCircle, FiTag, FiTv, FiUsers } from 'react-icons/fi';
import type {
  DetailMylistItem,
  FilterChip,
  GroupItem,
  LiteratureItem,
  MediaCardItem,
  PropertyItem,
  RailMetaItem,
  RelatedWorkItem,
  ResourceTabItem,
  StaffMember,
  StreamingLinkItem,
  TagLikeItem,
} from '@/types/ui';
import { ResourceList } from '@/components/shared';

export const mediaCards: MediaCardItem[] = [
  { id: 'anime-001', title: '攻殻機動隊 SAC_2045', meta: 'アニメ • 24話', badge: 'ANIME', isFavorite: true, rating: 4.5, to: '/media/anime/anime-001' },
  { id: 'movie-001', title: 'Dune: Part Two', meta: '映画 • 166 min', badge: 'MOVIE', rating: 4.0, to: '/media/movies/movie-001' },
  { id: 'manga-001', title: 'チ。-地球の運動について-', meta: '漫画 • 8巻', badge: 'MANGA', rating: 5.0, to: '/media/manga/manga-001' },
  { id: 'game-001', title: 'Baldur’s Gate 3', meta: 'ゲーム • CRPG', badge: 'GAME', isFavorite: true, rating: 5.0, to: '/media/games/game-001' },
];

export const searchResults: MediaCardItem[] = [
  { id: 'search-001', title: 'Interstellar', meta: '映画 • 2014', badge: 'MOVIE', actionLabel: '取り込む' },
  { id: 'search-002', title: 'The Leftovers', meta: 'ドラマ • 2014', badge: 'DRAMA', actionLabel: '取り込み済み', actionDisabled: true },
  { id: 'search-003', title: 'Ergo Proxy', meta: 'アニメ • 2006', badge: 'ANIME', actionLabel: '取り込む' },
];

export const literatureItems: LiteratureItem[] = [
  {
    id: 'book-001',
    title: 'Designing Data-Intensive Applications',
    byline: 'Martin Kleppmann',
    meta: 'O’Reilly • 2017',
    tags: [{ id: 'tag-ddia', label: 'distributed' }],
    favorite: true,
    rating: 5,
    to: '/academic-books/book-001',
  },
  {
    id: 'paper-001',
    title: 'Attention Is All You Need',
    byline: 'Vaswani et al.',
    meta: 'NIPS 2017',
    doi: '10.48550/arXiv.1706.03762',
    tags: [{ id: 'tag-transformer', label: 'transformer' }],
    rating: 5,
    to: '/papers/paper-001',
  },
];

export const filterChips: FilterChip[] = [
  { id: 'all', label: 'すべて', active: true },
  { id: 'favorite', label: 'お気に入り' },
  { id: 'type', label: '種別を追加', addTrigger: true },
  { id: 'tag', label: 'タグを追加', addTrigger: true },
];

export const railMetaItems: RailMetaItem[] = [
  { id: 'created', label: '登録日', value: '2026-07-01', icon: FiCalendar },
  { id: 'source', label: '登録方法', value: 'TMDB import', icon: FiLink2, muted: true },
];

export const detailTags: TagLikeItem[] = [
  { id: 't1', label: 'sf' },
  { id: 't2', label: 'rewatch' },
];

export const detailCategories: TagLikeItem[] = [
  { id: 'c1', label: 'favorites' },
  { id: 'c2', label: '2026' },
];

export const detailMylists: DetailMylistItem[] = [
  { id: 'm1', label: '週末に見る' },
  { id: 'm2', label: '研究モチーフ' },
];

export const animeGroups: GroupItem[] = [
  {
    id: 'season-1',
    title: 'Season 1',
    subtitle: '12 episodes',
    episodes: [
      { id: 'ep1', number: '01', title: 'NO NOISE NO LIFE', meta: '24m' },
      { id: 'ep2', number: '02', title: 'AT YOUR OWN RISK', meta: '24m' },
    ],
  },
  {
    id: 'season-2',
    title: 'Season 2',
    subtitle: '12 episodes',
    episodes: [{ id: 'ep13', number: '13', title: 'NEW WORLD', meta: '24m' }],
  },
];

export const paperProperties: PropertyItem[] = [
  { key: 'authors', label: '著者', value: 'Ashish Vaswani et al.' },
  { key: 'venue', label: '掲載', value: 'NIPS 2017' },
  { key: 'doi', label: 'DOI', value: '10.48550/arXiv.1706.03762' },
  { key: 'pages', label: 'ページ', value: '30-40', muted: true },
  { key: 'language', label: '言語', value: 'English' },
];

export const movieProperties: PropertyItem[] = [
  { key: 'director', label: '監督', value: 'Denis Villeneuve' },
  { key: 'runtime', label: '上映時間', value: '166 min' },
  { key: 'studio', label: '制作', value: 'Legendary Pictures' },
  { key: 'release', label: '公開', value: '2024-03-15' },
  { key: 'country', label: '国', value: 'US / CA' },
  { key: 'lang', label: '言語', value: 'English' },
];

export const staffMembers: StaffMember[] = [
  { id: 's1', label: '監督: 神山健治', subLabel: 'Director' },
  { id: 's2', label: '脚本: 檜垣亮', subLabel: 'Writer' },
];

export const streamingLinks: StreamingLinkItem[] = [
  { id: 'st1', label: 'Netflix', subLabel: 'Subscription' },
  { id: 'st2', label: 'Prime Video', subLabel: 'Rental' },
];

export const relatedWorks: RelatedWorkItem[] = [
  { id: 'r1', title: '攻殻機動隊 STAND ALONE COMPLEX', meta: 'シリーズ前作', to: '/media/anime/anime-001' },
  { id: 'r2', title: 'Ghost in the Shell', meta: '原作関連', to: '/media/manga/manga-001' },
];

export const animeResourceTabs: ResourceTabItem[] = [
  {
    key: 'links',
    label: 'リンク',
    icon: FiLink2,
    content: <ResourceList items={[{ id: 'l1', label: '公式サイト', href: 'https://example.com' }, { id: 'l2', label: 'AniList', href: 'https://example.com/anilist' }]} />,
  },
  {
    key: 'files',
    label: 'ファイル',
    icon: FiPaperclip,
    content: <ResourceList items={[{ id: 'f1', label: '視聴メモ.md', sub: 'Markdown' }, { id: 'f2', label: '設定資料.pdf', sub: 'PDF' }]} />,
  },
  {
    key: 'trailers',
    label: 'トレーラー',
    icon: FiFilm,
    content: <ResourceList items={[{ id: 't1', label: 'Trailer 01', href: 'https://example.com/trailer' }]} />,
  },
];

export const paperResourceTabs: ResourceTabItem[] = [
  {
    key: 'links',
    label: '出版社ページ',
    icon: FiLink2,
    content: <ResourceList items={[{ id: 'pl1', label: 'DOI resolver', href: 'https://doi.org/10.48550/arXiv.1706.03762' }]} />,
  },
  {
    key: 'files',
    label: 'ファイル',
    icon: FiPaperclip,
    content: <ResourceList items={[{ id: 'pf1', label: 'paper.pdf', sub: 'PDF' }, { id: 'pf2', label: 'notes.md', sub: 'Markdown' }]} />,
  },
];

export const sectionIcons = {
  summary: FiFileText,
  properties: FiTag,
  groups: FiPlayCircle,
  staff: FiUsers,
  related: FiLink2,
  streaming: FiTv,
};
