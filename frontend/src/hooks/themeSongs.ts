//! item detail の theme_songs をタブ表示用のビューモデルへ整形する共通処理。
//! useMovieDetailData / useAnimeDetailData / useItemDetailData(drama) から使う。

import type { ThemeSongGroup } from "@/components/detail";
import { themeSongTypeLabels } from "@/config/themeSongLabels";

export type ThemeSongTypeName = "op" | "ed" | "insert" | "image" | "character" | "theme" | "other";
export type ThemeSongLinkTypeName =
  | "youtube"
  | "spotify"
  | "apple_music"
  | "amazon_music"
  | "niconico"
  | "official"
  | "other";

export type ThemeSongLinkRecord = {
  id: string;
  theme_song_id: string;
  link_type: ThemeSongLinkTypeName;
  url: string;
  label: string | null;
  sort_order: number;
  created_at: string;
};

export type ItemThemeSong = {
  id: string;
  item_id: string;
  theme_type: ThemeSongTypeName;
  display_order: number;
  created_at: string;
  theme_song: {
    id: string;
    title: string;
    artist: string | null;
    composer: string | null;
    lyricist: string | null;
    arranger: string | null;
    note: string | null;
    links: ThemeSongLinkRecord[];
    created_at: string;
    updated_at: string;
  };
};

/** アーティスト・作曲・作詞・編曲を「・」区切りの1行にまとめる */
function buildSub(song: ItemThemeSong["theme_song"]): string {
  return [
    song.artist,
    song.composer ? `作曲: ${song.composer}` : null,
    song.lyricist ? `作詞: ${song.lyricist}` : null,
    song.arranger ? `編曲: ${song.arranger}` : null,
  ]
    .filter(Boolean)
    .join(" ・ ");
}

/**
 * theme_type ごとにグループ化する。
 * バックエンドが theme_type(enum順) → display_order → created_at でソート済みのため、
 * 並び順を保ったまま連続する同一 theme_type をまとめるだけでよい。
 */
export function mapThemeSongGroups(songs: ItemThemeSong[] | undefined): ThemeSongGroup[] {
  const groups: ThemeSongGroup[] = [];

  for (const entry of songs ?? []) {
    const song = entry.theme_song;
    const mapped = {
      id: entry.id,
      title: song.title,
      sub: buildSub(song),
      note: song.note,
      links: song.links.map((link) => ({
        id: link.id,
        type: link.link_type,
        url: link.url,
        label: link.label,
      })),
    };

    const last = groups[groups.length - 1];
    if (last && last.type === entry.theme_type) {
      last.songs.push(mapped);
    } else {
      groups.push({
        type: entry.theme_type,
        label: themeSongTypeLabels[entry.theme_type] ?? entry.theme_type,
        songs: [mapped],
      });
    }
  }

  return groups;
}
