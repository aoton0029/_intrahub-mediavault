import { useMutation } from "@tanstack/react-query";
import { apiFetch } from "@/lib/apiClient";

export type SearchMediaType = "anime" | "movie" | "drama" | "manga" | "novel" | "game" | "academic_book";

export type SearchResultItem = {
  id: string;
  media_type: SearchMediaType;
  provider: string | null;
  title: string;
  thumbnail_url: string | null;
  year: number | null;
};

type SearchResponse = {
  success: boolean;
  data: SearchResultItem[];
};

type ApiErrorResponse = {
  code?: string;
  error?: {
    code?: string;
    message?: string;
  };
  message?: string;
};

export class MediaSearchError extends Error {
  status: number;
  code?: string;

  constructor(message: string, status: number, code?: string) {
    super(message);
    this.name = "MediaSearchError";
    this.status = status;
    this.code = code;
  }
}

function buildQueryString(params: Record<string, string | undefined>) {
  const searchParams = new URLSearchParams();

  for (const [key, value] of Object.entries(params)) {
    if (value) {
      searchParams.set(key, value);
    }
  }

  const queryString = searchParams.toString();
  return queryString ? `?${queryString}` : "";
}

async function fetchMediaSearch({ mediaType, query }: { mediaType: SearchMediaType; query: string }) {
  const response = await apiFetch(
    `/items/search${buildQueryString({
      media_type: mediaType,
      q: query.trim(),
    })}`,
  );

  if (!response.ok) {
    let code: string | undefined;
    let message = `Failed to search items: ${response.status}`;

    try {
      const json = (await response.json()) as ApiErrorResponse;
      code = json.code ?? json.error?.code;
      message = json.error?.message ?? json.message ?? message;
    } catch {
      // Ignore JSON parse errors and fall back to the default message.
    }

    throw new MediaSearchError(message, response.status, code);
  }

  const json = (await response.json()) as SearchResponse;
  return json.data;
}

export function useMediaSearch() {
  return useMutation<SearchResultItem[], MediaSearchError, { mediaType: SearchMediaType; query: string }>({
    mutationFn: fetchMediaSearch,
  });
}
