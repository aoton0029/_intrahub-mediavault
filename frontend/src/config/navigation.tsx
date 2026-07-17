import type { IconType } from "react-icons";
import { FiBook, FiBookOpen, FiFileText, FiFilm, FiFolder, FiGrid, FiHome, FiMonitor, FiSettings, FiTv } from "react-icons/fi";

export type NavigationItem = {
  label: string;
  to: string;
  icon: IconType;
  indent?: boolean;
};

export type NavigationSection = {
  label?: string;
  items: NavigationItem[];
  grow?: boolean;
};

export const navigationSections: NavigationSection[] = [
  { items: [{ label: "ホーム", to: "/", icon: FiHome }] },
  {
    label: "メディア",
    items: [
      { label: "すべて", to: "/media", icon: FiFilm },
      { label: "映画", to: "/media?media_type=movie", icon: FiFilm },
      { label: "ドラマ", to: "/media?media_type=drama", icon: FiTv },
      { label: "アニメ", to: "/media?media_type=anime", icon: FiTv },
      { label: "漫画", to: "/media?media_type=manga", icon: FiBookOpen },
      { label: "小説", to: "/media?media_type=novel", icon: FiBook },
      { label: "ゲーム", to: "/media?media_type=game", icon: FiMonitor },
    ],
  },
  {
    label: "リサーチ",
    items: [
      { label: "学術書・専門書", to: "/academic-books", icon: FiBookOpen },
      { label: "論文・文献", to: "/research/papers", icon: FiFileText },
    ],
  },
  { label: "コレクション", items: [{ label: "マイリスト", to: "/mylists", icon: FiFolder }] },
  { label: "ツール", items: [{ label: "並べてシェア", to: "/tools/collage", icon: FiGrid }] },
  { label: "Misc", grow: true, items: [{ label: "設定", to: "/settings", icon: FiSettings }] },
];
