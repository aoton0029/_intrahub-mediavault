import { useQuery } from "@tanstack/react-query";
import { apiFetch } from "@/lib/apiClient";

type ApiOk<T> = { success: boolean; data: T };

export type ThemeSongSearchLink = {
  id: string;
  theme_song_id: string;
  link_type: string;
  url: string;
  label: string | null;
  sort_order: number;
  created_at: string;
};

export type ThemeSongSearchResult = {
  id: string;
  title: string;
  artist: string | null;
  composer: string | null;
  lyricist: string | null;
  arranger: string | null;
  note: string | null;
  links: ThemeSongSearchLink[];
  created_at: string;
  updated_at: string;
};

async function parseJson<T>(response: Response) {
  if (!response.ok) {
    throw new Error(`Request failed: ${response.status}`);
  }
  return (await response.json()) as T;
}

async function searchThemeSongsByTitle(q: string): Promise<ThemeSongSearchResult[]> {
  const params = new URLSearchParams({ q });
  const json = await parseJson<ApiOk<ThemeSongSearchResult[]>>(await apiFetch(`/theme-songs?${params.toString()}`));
  return json.data;
}

export async function createThemeSong(input: {
  title: string;
  artist?: string;
  composer?: string;
  lyricist?: string;
  arranger?: string;
  note?: string;
}): Promise<ThemeSongSearchResult> {
  const json = await parseJson<ApiOk<ThemeSongSearchResult>>(
    await apiFetch("/theme-songs", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        title: input.title,
        artist: input.artist || undefined,
        composer: input.composer || undefined,
        lyricist: input.lyricist || undefined,
        arranger: input.arranger || undefined,
        note: input.note || undefined,
      }),
    }),
  );
  return json.data;
}

export function useThemeSongSearch(q: string) {
  const query = useQuery({
    queryKey: ["theme-song-search", q],
    queryFn: () => searchThemeSongsByTitle(q),
    enabled: q.trim().length > 0,
  });

  return {
    results: query.data ?? [],
    isLoading: query.isLoading,
    isError: query.isError,
  };
}
