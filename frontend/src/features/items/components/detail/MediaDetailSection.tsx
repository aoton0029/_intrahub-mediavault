import type { MediaDetails } from '../../types';
import { DetailSection } from './media-sections/DetailSection';
import {
  type DetailRow,
  buildAnimeRows,
  buildBookRows,
  buildDramaRows,
  buildGameRows,
  buildMangaRows,
  buildMovieRows,
} from './media-sections/rows';

/** detail の media_type に応じてメディア別の表示行を組み立てる */
function buildRows(detail: MediaDetails): DetailRow[] {
  switch (detail.media_type) {
    case 'anime':
      return buildAnimeRows(detail);
    case 'manga':
      return buildMangaRows(detail);
    case 'movie':
      return buildMovieRows(detail);
    case 'drama':
      return buildDramaRows(detail);
    case 'game':
      return buildGameRows(detail);
    case 'novel':
    case 'academic_book':
    case 'paper':
      return buildBookRows(detail);
  }
}

/**
 * 種別固有情報のディスパッチャ。detail（正規化済みMediaDetails）の media_type に応じて
 * メディア別の項目を表示する。detail が無いアイテム（手動作成・移行前インポート）は
 * 何も描画しない。
 */
export function MediaDetailSection({ detail }: { detail: MediaDetails | null }) {
  if (!detail) return null;
  return <DetailSection rows={buildRows(detail)} />;
}
