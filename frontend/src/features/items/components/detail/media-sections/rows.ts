/**
 * 種別固有情報セクションの行ビルダー群。
 * detail（正規化済みMediaDetails）のメディア別拡張フィールドを日本語ラベル付きの表示行へ変換する。
 * コンポーネント本体（*.tsx）とは react-refresh 制約のためファイルを分離している。
 */
import type {
  AnimeDetails,
  DramaDetails,
  GameDetails,
  MangaDetails,
  MovieDetails,
  NovelDetails,
} from '../../../types';

/** 種別固有情報の1行。href がある場合は値を外部リンクとして描画する */
export interface DetailRow {
  label: string;
  value: string;
  href?: string;
}

/**
 * 値を表示用文字列へ整形する。null・undefined・空文字・空配列は null を返し（行ごとスキップ）、
 * 配列は「 / 」区切りで結合する。
 */
export function formatValue(value: unknown): string | null {
  if (value === null || value === undefined) return null;
  if (Array.isArray(value)) {
    return value.length > 0 ? value.map(String).join(' / ') : null;
  }
  const text = String(value);
  return text.trim() === '' ? null : text;
}

/** formatValue が有効値を返す場合のみ DetailRow を組み立てる行ビルダー */
function buildRow(label: string, value: unknown, href?: string): DetailRow | null {
  const formatted = formatValue(value);
  if (formatted === null) return null;
  return href ? { label, value: formatted, href } : { label, value: formatted };
}

/** null行を除去して行リストを確定させる */
function compactRows(rows: (DetailRow | null)[]): DetailRow[] {
  return rows.filter((row): row is DetailRow => row !== null);
}

/** アニメ固有情報（シーズンは「season year」形式で結合、PVは外部リンク） */
export function buildAnimeRows(detail: AnimeDetails): DetailRow[] {
  const season = [detail.season, detail.year].filter((v) => v !== null && v !== undefined);
  return compactRows([
    buildRow('話数', detail.episodes),
    buildRow('放送状況', detail.status),
    buildRow('シーズン', season.length > 0 ? season.join(' ') : null),
    buildRow('制作会社', detail.studios),
    buildRow('原作', detail.source),
    buildRow('1話の長さ', detail.duration),
    detail.trailer_url ? buildRow('PV', detail.trailer_url, detail.trailer_url) : null,
  ]);
}

export function buildMangaRows(detail: MangaDetails): DetailRow[] {
  return compactRows([
    buildRow('話数', detail.chapters),
    buildRow('巻数', detail.volumes),
    buildRow('連載状況', detail.status),
    buildRow('著者', detail.authors),
    buildRow('掲載誌', detail.serializations),
  ]);
}

export function buildMovieRows(detail: MovieDetails): DetailRow[] {
  return compactRows([
    buildRow('上映時間(分)', detail.runtime_minutes),
    buildRow('原語', detail.original_language),
    buildRow('評価数', detail.vote_count),
    buildRow('シリーズ', detail.collection),
    buildRow('制作会社', detail.production_companies),
  ]);
}

export function buildDramaRows(detail: DramaDetails): DetailRow[] {
  return compactRows([
    buildRow('シーズン数', detail.number_of_seasons),
    buildRow('話数', detail.number_of_episodes),
    buildRow('放送局', detail.networks),
    buildRow('放送状況', detail.status),
    buildRow('原語', detail.original_language),
    buildRow('初回放送日', detail.first_air_date),
    buildRow('最終放送日', detail.last_air_date),
  ]);
}

export function buildGameRows(detail: GameDetails): DetailRow[] {
  return compactRows([
    buildRow('プラットフォーム', detail.platforms),
    buildRow('開発', detail.developers),
    buildRow('パブリッシャー', detail.publishers),
    buildRow('Metacritic', detail.metacritic),
    buildRow('Steam AppID', detail.steam_appid),
    buildRow('ストーリー', detail.storyline),
  ]);
}

/** 小説・学術書・論文が共有する書誌情報 */
export function buildBookRows(detail: NovelDetails): DetailRow[] {
  return compactRows([
    buildRow('著者', detail.authors),
    buildRow('出版社', detail.publisher),
    buildRow('ISBN', detail.isbn),
    buildRow('ページ数', detail.page_count),
    buildRow('判型', detail.physical_format),
  ]);
}
