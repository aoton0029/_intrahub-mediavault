/**
 * TASK-0023 E2Eテスト用フィクスチャ
 *
 * frontend/src/features/items/types.ts の Item 型に準拠したモックデータと、
 * page.route() 用のヘルパー関数群を提供する。
 *
 * 参照: docs/implements/collection-browsing/TASK-0023/{requirements,testcases}.md
 */
import type { Page, Route } from '@playwright/test';

export type MediaType =
  | 'anime'
  | 'movie'
  | 'drama'
  | 'manga'
  | 'novel'
  | 'game'
  | 'academic_book'
  | 'paper';

export type ItemStatus = 'not_started' | 'in_progress' | 'completed';

export interface MockItem {
  id: string;
  media_type: MediaType;
  title: string;
  status: ItemStatus;
  is_favorite: boolean;
  created_at: string;
  updated_at: string;
}

function makeItem(index: number, overrides: Partial<MockItem> = {}): MockItem {
  return {
    id: `00000000-0000-0000-0000-${String(index).padStart(12, '0')}`,
    media_type: 'anime',
    title: `テストアイテム${index}`,
    status: 'in_progress',
    is_favorite: false,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    ...overrides,
  };
}

/**
 * 一覧表示の基本検証用: 20件、うち3件が is_favorite:true
 * 🔵 TC-001-01/TC-001-02 用
 */
export const mockItems: MockItem[] = Array.from({ length: 20 }, (_, i) =>
  makeItem(i + 1, { is_favorite: i < 3 }),
);

/**
 * 無限スクロール検証用: 40件（20件 x 2ページ）
 * 🔵 TC-103-01/TC-103-B01 用
 */
export const mockItemsForScroll: MockItem[] = Array.from({ length: 40 }, (_, i) =>
  makeItem(100 + i + 1, { title: `スクロールアイテム${i + 1}` }),
);

/**
 * お気に入り + タグ(#SF) AND絞り込み結果用: 2件
 * 🔵 TC-101-01 用
 */
export const mockFilteredItems: MockItem[] = [
  makeItem(201, { title: 'SFお気に入り作品1', is_favorite: true }),
  makeItem(202, { title: 'SFお気に入り作品2', is_favorite: true }),
];

/**
 * タイトル検索「星屑」部分一致結果用: 2件
 * 🔵 TC-101-02 用
 */
export const mockSearchItems: MockItem[] = [
  makeItem(301, { title: '星屑のシンフォニア' }),
  makeItem(302, { title: '星屑と旅する物語' }),
];

interface PaginationOverride {
  page?: number;
  limit?: number;
  total?: number;
}

function buildSuccessBody(items: MockItem[], pagination: PaginationOverride) {
  return {
    success: true,
    data: items,
    pagination: {
      page: pagination.page ?? 1,
      limit: pagination.limit ?? 20,
      total: pagination.total ?? items.length,
    },
  };
}

function buildErrorBody(message = 'Failed to fetch items') {
  return {
    success: false,
    error: {
      code: 'INTERNAL_ERROR',
      message,
    },
  };
}

/**
 * 【共通ルート名】: 全モック関数が対象とする GET /api/v1/items のURLパターン
 * 【重複除去】: 各 mockItemsRoute* 関数で個別に文字列リテラルを書くと、
 *   将来エンドポイントが変わった際に修正漏れが発生しやすいため定数化する
 * 🔵 信頼性レベル: 既存実装（page.route呼び出し箇所5箇所）から抽出、値は変更なし
 */
const ITEMS_ROUTE_PATTERN = '**/api/v1/items*';

/**
 * 【ヘルパー関数】: page.route() 登録処理を1箇所に集約するための共通ラッパー
 * 【再利用性】: mockItemsRoute/mockItemsRouteByPage/mockItemsRouteByQuery/
 *   mockItemsRouteErrorThenSuccess/mockItemsRouteAlwaysError の全てが利用する
 * 【単一責任】: 「/api/v1/items へのルート登録」という責務のみを担い、
 *   レスポンス内容の組み立ては呼び出し側のhandlerに委譲する
 * 【改善内容】: 5つのモック関数それぞれで重複していた `page.route('**\/api/v1/items*', ...)`
 *   の登録処理を共通化し、変更点を1箇所に限定した（Greenフェーズ申し送り事項の解消）
 * 🔵 信頼性レベル: green-phase.md §7「route登録ヘルパーの重複除去」に基づくテストコードのみの整理
 */
async function registerItemsRoute(
  page: Page,
  handler: (route: Route, url: URL) => Promise<void>,
): Promise<void> {
  await page.route(ITEMS_ROUTE_PATTERN, async (route: Route) => {
    const url = new URL(route.request().url());
    await handler(route, url);
  });
}

/**
 * 単純な成功レスポンスを常に返すモック登録（クエリパラメータは無視）。
 * 🔵 TC-001-01/02, TC-001-B01（空配列時）等の単純ケース用
 */
export async function mockItemsRoute(
  page: Page,
  options: { items: MockItem[]; total?: number },
): Promise<void> {
  await registerItemsRoute(page, async (route, url) => {
    const requestedPage = Number(url.searchParams.get('page') ?? '1');
    const limit = Number(url.searchParams.get('limit') ?? '20');

    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(
        buildSuccessBody(options.items, {
          page: requestedPage,
          limit,
          total: options.total ?? options.items.length,
        }),
      ),
    });
  });
}

/**
 * ページ番号に応じて異なるレスポンスを返すモック登録（無限スクロール検証用）。
 * 🔵 TC-103-01/TC-103-B01 用
 */
export async function mockItemsRouteByPage(
  page: Page,
  options: { page1Items: MockItem[]; page2Items: MockItem[]; total: number },
): Promise<{ getCallCount: () => number }> {
  let callCount = 0;

  await registerItemsRoute(page, async (route, url) => {
    callCount += 1;
    const requestedPage = Number(url.searchParams.get('page') ?? '1');
    const items = requestedPage === 1 ? options.page1Items : options.page2Items;

    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(buildSuccessBody(items, { page: requestedPage, total: options.total })),
    });
  });

  return { getCallCount: () => callCount };
}

/**
 * フィルタ・検索条件によって応答を出し分けるモック登録。
 * matcher が true を返すクエリの場合のみ items を返し、それ以外は0件を返す。
 * 🔵 TC-101-01/TC-101-02/TC-101-E01/TC-101-B02 用
 */
export async function mockItemsRouteByQuery(
  page: Page,
  matcher: (query: URLSearchParams) => MockItem[] | null,
): Promise<{ getLastQuery: () => URLSearchParams | undefined }> {
  let lastQuery: URLSearchParams | undefined;

  await registerItemsRoute(page, async (route, url) => {
    lastQuery = url.searchParams;
    const matched = matcher(url.searchParams);
    const items = matched ?? [];

    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(buildSuccessBody(items, { total: items.length })),
    });
  });

  return { getLastQuery: () => lastQuery };
}

/**
 * 呼び出し回数に応じてエラー/成功を切り替えるモック登録（再試行リカバリ検証用）。
 * 🟡 TC-001-E01/TC-001-E02 用
 */
export async function mockItemsRouteErrorThenSuccess(
  page: Page,
  options: { failCount: number; successItems: MockItem[] },
): Promise<{ getCallCount: () => number }> {
  let callCount = 0;

  await registerItemsRoute(page, async (route) => {
    callCount += 1;
    if (callCount <= options.failCount) {
      await route.fulfill({
        status: 500,
        contentType: 'application/json',
        body: JSON.stringify(buildErrorBody()),
      });
      return;
    }

    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(
        buildSuccessBody(options.successItems, { total: options.successItems.length }),
      ),
    });
  });

  return { getCallCount: () => callCount };
}

/**
 * 常にエラーを返すモック登録（継続障害検証用）。
 * 🟡 TC-001-E02 用
 */
export async function mockItemsRouteAlwaysError(page: Page): Promise<void> {
  await registerItemsRoute(page, async (route) => {
    await route.fulfill({
      status: 500,
      contentType: 'application/json',
      body: JSON.stringify(buildErrorBody()),
    });
  });
}
