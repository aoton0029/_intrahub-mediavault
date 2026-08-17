/** theme_song_type の表示ラベル。バックエンドの ThemeSongType enum と対応 */
export const themeSongTypeLabels: Record<string, string> = {
  op: "OP",
  ed: "ED",
  insert: "挿入歌",
  image: "イメージソング",
  character: "キャラクターソング",
  theme: "主題歌",
  other: "その他",
};

/** theme_song_link_type の表示ラベル */
export const themeSongLinkLabels: Record<string, string> = {
  youtube: "YouTube",
  spotify: "Spotify",
  apple_music: "Apple Music",
  amazon_music: "Amazon Music",
  niconico: "ニコニコ",
  official: "公式",
  other: "リンク",
};
