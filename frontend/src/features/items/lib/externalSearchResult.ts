import type { ExternalSearchResult } from '../types';

/**
 * ExternalSearchResult.raw_data は各プロバイダの生レスポンスをそのまま格納したものであり、
 * 正規化フィールドを持たない（12_general_media_search.htmlのコメント参照）。
 * そのため、一覧カード表示に必要な原題・カバー画像・公開年はプロバイダごとに
 * best-effort で抽出する。未知の形・欠損時は undefined を返す。
 */

function asRecord(value: unknown): Record<string, unknown> | undefined {
  return typeof value === 'object' && value !== null ? (value as Record<string, unknown>) : undefined;
}

function asString(value: unknown): string | undefined {
  return typeof value === 'string' && value.length > 0 ? value : undefined;
}

export function getOriginalTitle(result: ExternalSearchResult): string | undefined {
  const raw = result.raw_data;
  switch (result.provider) {
    case 'jikan':
      return asString(raw.title_japanese) ?? asString(raw.title);
    case 'tmdb':
      return asString(raw.original_title) ?? asString(raw.original_name);
    case 'open_library':
      return asString(raw.title);
    case 'igdb':
      return asString(raw.name);
    default:
      return undefined;
  }
}

export function getCoverImageUrl(result: ExternalSearchResult): string | undefined {
  return result.thumbnail_url ?? undefined;
}

export function getReleaseYear(result: ExternalSearchResult): string | undefined {
  const raw = result.raw_data;
  switch (result.provider) {
    case 'jikan':
      return asString(raw.year) ?? asString(raw.aired && asRecord(raw.aired)?.from)?.slice(0, 4);
    case 'tmdb': {
      const date = asString(raw.release_date) ?? asString(raw.first_air_date);
      return date?.slice(0, 4);
    }
    case 'open_library': {
      const year = raw.first_publish_year;
      return typeof year === 'number' ? String(year) : undefined;
    }
    case 'igdb': {
      const timestamp = raw.first_release_date;
      return typeof timestamp === 'number'
        ? String(new Date(timestamp * 1000).getUTCFullYear())
        : undefined;
    }
    default:
      return undefined;
  }
}
