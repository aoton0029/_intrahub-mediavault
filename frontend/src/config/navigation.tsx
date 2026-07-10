import type { IconType } from "react-icons";
import { FiBook, FiBookOpen, FiFileText, FiFilm, FiFolder, FiHome, FiMonitor, FiSettings, FiTv } from "react-icons/fi";

export type NavigationItem = {
  label: string;
  to: string;
  icon: IconType;
  count?: number;
  indent?: boolean;
};

export type NavigationSection = {
  label?: string;
  items: NavigationItem[];
  grow?: boolean;
};

export const navigationSections: NavigationSection[] = [
  { items: [{ label: "ホーム", to: "/", icon: FiHome, count: 128 }] },
  {
    label: "メディア",
    items: [
      { label: "すべて", to: "/media", icon: FiFilm, count: 96 },
      { label: "映画", to: "/media?media_type=movie", icon: FiFilm, count: 9 },
      { label: "ドラマ", to: "/media?media_type=drama", icon: FiTv },
      { label: "アニメ", to: "/media?media_type=anime", icon: FiTv, count: 28 },
      { label: "漫画", to: "/media?media_type=manga", icon: FiBookOpen, count: 34 },
      { label: "小説", to: "/media?media_type=novel", icon: FiBook, count: 15 },
      { label: "ゲーム", to: "/media?media_type=game", icon: FiMonitor, count: 3 },
    ],
  },
  {
    label: "リサーチ",
    items: [
      { label: "学術書・専門書", to: "/academic-books", icon: FiBookOpen, count: 21 },
      { label: "論文・文献", to: "/research/papers", icon: FiFileText, count: 11 },
    ],
  },
  { label: "コレクション", items: [{ label: "マイリスト", to: "/mylists", icon: FiFolder }] },
  { label: "Misc", grow: true, items: [{ label: "設定", to: "/settings", icon: FiSettings }] },
];
