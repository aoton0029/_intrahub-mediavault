import type { IconType } from "react-icons";
import { FiBook, FiBookOpen, FiFilm, FiMonitor, FiTv } from "react-icons/fi";

export type MediaType = "movie" | "drama" | "anime" | "manga" | "novel" | "game";

export const mediaTypeConfig = {
  movie: { label: "映画", icon: FiFilm },
  drama: { label: "ドラマ", icon: FiTv },
  anime: { label: "アニメ", icon: FiTv },
  manga: { label: "漫画", icon: FiBookOpen },
  novel: { label: "小説", icon: FiBook },
  game: { label: "ゲーム", icon: FiMonitor },
} satisfies Record<MediaType, { label: string; icon: IconType }>;

export const mediaTypes = Object.keys(mediaTypeConfig) as MediaType[];
