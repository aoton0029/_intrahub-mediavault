import { describe, expect, it } from 'vitest';
import type {
  AnimeDetails,
  DramaDetails,
  GameDetails,
  MangaDetails,
  MediaCore,
  MovieDetails,
  NovelDetails,
} from '../../types';
import {
  buildAnimeRows,
  buildBookRows,
  buildDramaRows,
  buildGameRows,
  buildMangaRows,
  buildMovieRows,
  formatValue,
} from './media-sections/rows';

/** MediaCore 部分の共通フィクスチャ（行ビルダーは拡張フィールドのみ参照する） */
function core(media_type: MediaCore['media_type']): MediaCore {
  return {
    media_type,
    provider: null,
    external_id: 'ext-1',
    title: 'テスト作品',
    original_title: null,
    alternative_titles: [],
    description: null,
    release_date: null,
    image_url: null,
    genres: [],
    rating: null,
    url: null,
  };
}

describe('formatValue', () => {
  it('null・undefined・空文字・空配列は null を返す', () => {
    expect(formatValue(null)).toBeNull();
    expect(formatValue(undefined)).toBeNull();
    expect(formatValue('')).toBeNull();
    expect(formatValue('  ')).toBeNull();
    expect(formatValue([])).toBeNull();
  });

  it('配列は「 / 」区切りで整形する', () => {
    expect(formatValue(['PC', 'Switch'])).toBe('PC / Switch');
  });

  it('数値は文字列化する', () => {
    expect(formatValue(12)).toBe('12');
  });
});

describe('buildAnimeRows', () => {
  const detail: AnimeDetails = {
    ...core('anime'),
    media_type: 'anime',
    episodes: 24,
    status: 'Finished Airing',
    season: 'spring',
    year: 2011,
    studios: ['Studio A', 'Studio B'],
    source: 'Manga',
    duration: '24 min per ep',
    trailer_url: 'https://example.com/pv',
  };

  it('全フィールドを日本語ラベル・定義順で返し、PVは外部リンクになる', () => {
    expect(buildAnimeRows(detail)).toEqual([
      { label: '話数', value: '24' },
      { label: '放送状況', value: 'Finished Airing' },
      { label: 'シーズン', value: 'spring 2011' },
      { label: '制作会社', value: 'Studio A / Studio B' },
      { label: '原作', value: 'Manga' },
      { label: '1話の長さ', value: '24 min per ep' },
      { label: 'PV', value: 'https://example.com/pv', href: 'https://example.com/pv' },
    ]);
  });

  it('null・空配列のフィールドはスキップする', () => {
    const rows = buildAnimeRows({
      ...detail,
      status: null,
      season: null,
      year: null,
      studios: [],
      source: null,
      duration: null,
      trailer_url: null,
    });
    expect(rows).toEqual([{ label: '話数', value: '24' }]);
  });

  it('season のみ（year なし）でもシーズン行を表示する', () => {
    const rows = buildAnimeRows({ ...detail, year: null });
    expect(rows).toContainEqual({ label: 'シーズン', value: 'spring' });
  });
});

describe('buildMangaRows', () => {
  it('漫画固有フィールドを日本語ラベルで返す', () => {
    const detail: MangaDetails = {
      ...core('manga'),
      media_type: 'manga',
      chapters: 100,
      volumes: 12,
      status: 'Publishing',
      authors: ['作者A'],
      serializations: ['週刊誌B'],
    };
    expect(buildMangaRows(detail)).toEqual([
      { label: '話数', value: '100' },
      { label: '巻数', value: '12' },
      { label: '連載状況', value: 'Publishing' },
      { label: '著者', value: '作者A' },
      { label: '掲載誌', value: '週刊誌B' },
    ]);
  });
});

describe('buildMovieRows', () => {
  it('映画固有フィールドを日本語ラベルで返す', () => {
    const detail: MovieDetails = {
      ...core('movie'),
      media_type: 'movie',
      runtime_minutes: 120,
      original_language: 'ja',
      vote_count: 4321,
      collection: 'シリーズX',
      production_companies: ['会社A', '会社B'],
    };
    expect(buildMovieRows(detail)).toEqual([
      { label: '上映時間(分)', value: '120' },
      { label: '原語', value: 'ja' },
      { label: '評価数', value: '4321' },
      { label: 'シリーズ', value: 'シリーズX' },
      { label: '制作会社', value: '会社A / 会社B' },
    ]);
  });
});

describe('buildDramaRows', () => {
  it('ドラマ固有フィールドを日本語ラベルで返す', () => {
    const detail: DramaDetails = {
      ...core('drama'),
      media_type: 'drama',
      number_of_seasons: 3,
      number_of_episodes: 30,
      networks: ['局A'],
      status: 'Ended',
      original_language: 'en',
      first_air_date: '2020-01-01',
      last_air_date: '2022-12-31',
    };
    expect(buildDramaRows(detail)).toEqual([
      { label: 'シーズン数', value: '3' },
      { label: '話数', value: '30' },
      { label: '放送局', value: '局A' },
      { label: '放送状況', value: 'Ended' },
      { label: '原語', value: 'en' },
      { label: '初回放送日', value: '2020-01-01' },
      { label: '最終放送日', value: '2022-12-31' },
    ]);
  });
});

describe('buildGameRows', () => {
  it('ゲーム固有フィールドを日本語ラベルで返す', () => {
    const detail: GameDetails = {
      ...core('game'),
      media_type: 'game',
      platforms: ['PC', 'Switch'],
      developers: ['Dev A'],
      publishers: ['Pub A'],
      screenshots: [],
      metacritic: 85,
      steam_appid: 12345,
      storyline: 'あらすじ',
    };
    expect(buildGameRows(detail)).toEqual([
      { label: 'プラットフォーム', value: 'PC / Switch' },
      { label: '開発', value: 'Dev A' },
      { label: 'パブリッシャー', value: 'Pub A' },
      { label: 'Metacritic', value: '85' },
      { label: 'Steam AppID', value: '12345' },
      { label: 'ストーリー', value: 'あらすじ' },
    ]);
  });
});

describe('buildBookRows（novel / academic_book / paper 共用）', () => {
  const bookFields = {
    authors: ['山田 太郎', '佐藤 花子'],
    publisher: '出版社A',
    isbn: '978-4-00-000000-0',
    page_count: 320,
    physical_format: '単行本',
  };

  it.each(['novel', 'academic_book', 'paper'] as const)(
    '%s の書誌情報を日本語ラベルで返す',
    (mediaType) => {
      const detail: NovelDetails = {
        ...core(mediaType),
        media_type: mediaType,
        ...bookFields,
      };
      expect(buildBookRows(detail)).toEqual([
        { label: '著者', value: '山田 太郎 / 佐藤 花子' },
        { label: '出版社', value: '出版社A' },
        { label: 'ISBN', value: '978-4-00-000000-0' },
        { label: 'ページ数', value: '320' },
        { label: '判型', value: '単行本' },
      ]);
    },
  );
});
