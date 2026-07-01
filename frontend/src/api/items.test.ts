/**
 * TASK-0009: items APIフック テストファイル (Redフェーズ)
 * 🔵 docs/tasks/frontend-collection-ui/TASK-0009.md 完了条件より
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { createElement } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ApiClientError } from '@/types';
import type { ItemListFilters, Item, UpdateItemStatusRequest } from '@/types';
import {
  useItemsQuery,
  useItemQuery,
  useDeleteItemMutation,
  useUpdateItemStatusMutation,
} from './items';

// ===== テストユーティリティ =====

function createQueryClient() {
  // 【テスト前準備】: retryを無効化してテストを確定的に実行できるQueryClientを生成
  return new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });
}

function createWrapper(queryClient: QueryClient) {
  // 【環境初期化】: TanStack QueryフックをテストするためのProviderラッパー
  return ({ children }: { children: React.ReactNode }) =>
    createElement(QueryClientProvider, { client: queryClient }, children);
}

function mockFetchSuccess(data: unknown, extra?: { pagination?: unknown }) {
  // 【テストデータ準備】: 成功レスポンスのfetchモックを設定
  vi.stubGlobal(
    'fetch',
    vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, data, ...extra }),
    } as Response)
  );
}

function mockFetchError(code: string, message: string) {
  // 【テストデータ準備】: APIエラーレスポンスのfetchモックを設定
  vi.stubGlobal(
    'fetch',
    vi.fn().mockResolvedValue({
      ok: false,
      json: async () => ({ success: false, error: { code, message } }),
    } as Response)
  );
}

const mockItem: Item = {
  id: 'item-001',
  title: 'テストアニメ',
  mediaType: 'anime',
  status: 'not_started',
  isFavorite: false,
  source: 'manual',
  createdAt: '2026-07-01T00:00:00Z',
  updatedAt: '2026-07-01T00:00:00Z',
  details: { genreList: [] },
};

// ===== テストスイート =====

describe('useItemsQuery', () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    // 【テスト前準備】: 各テスト前にQueryClientとfetchモックをリセット
    // 【環境初期化】: キャッシュ汚染を防ぐため毎回新しいQueryClientを生成
    queryClient = createQueryClient();
  });

  afterEach(() => {
    // 【テスト後処理】: fetchのグローバルモックをリセット
    // 【状態復元】: 次のテストにモック状態が持ち越されないようにする
    vi.unstubAllGlobals();
    queryClient.clear();
  });

  it('TC-IQ-N-01: フィルタなしでGET /itemsを呼び出す', async () => {
    // 【テスト目的】: フィルタ未指定時に余分なクエリパラメータが付かないことを確認
    // 【テスト内容】: filters={}でuseItemsQueryを実行し、fetchのURL引数を検証
    // 【期待される動作】: GET /items（クエリパラメータなし）が発行され、data配列が返る
    // 🔵 TASK-0009.md 完了条件より

    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, data: [mockItem] }),
    } as Response);
    vi.stubGlobal('fetch', fetchMock);

    // 【初期条件設定】: 空フィルタでクエリを実行
    const filters: ItemListFilters = {};
    const wrapper = createWrapper(queryClient);
    const { result } = renderHook(() => useItemsQuery(filters), { wrapper });

    // 【実際の処理実行】: データ取得完了を待機
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    // 【結果検証】: fetchが呼ばれ、URLにクエリパラメータが付いていないことを確認
    expect(fetchMock).toHaveBeenCalled(); // 【確認内容】: fetchが1回呼ばれたことを確認
    const calledUrl: string = fetchMock.mock.calls[0][0] as string;
    expect(calledUrl).toContain('/items'); // 【確認内容】: /itemsエンドポイントへのリクエストであることを確認
    expect(calledUrl).not.toContain('media_type='); // 【確認内容】: 空フィルタ時にmedia_typeが含まれないことを確認
  });

  it('TC-IQ-N-02: mediaType/page/limitフィルタ付きでGET /itemsを呼び出す', async () => {
    // 【テスト目的】: フィルタオブジェクトのプロパティがスネークケースに変換されることを確認
    // 【テスト内容】: filters={mediaType:'anime',page:1,limit:20}でuseItemsQueryを実行
    // 【期待される動作】: GET /items?media_type=anime&page=1&limit=20 が発行される
    // 🔵 TASK-0009.md テストケース1より

    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, data: [mockItem] }),
    } as Response);
    vi.stubGlobal('fetch', fetchMock);

    const filters: ItemListFilters = { mediaType: 'anime', page: 1, limit: 20 };
    const wrapper = createWrapper(queryClient);
    const { result } = renderHook(() => useItemsQuery(filters), { wrapper });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    // 【結果検証】: URLにスネークケースのクエリパラメータが含まれることを確認
    const calledUrl: string = fetchMock.mock.calls[0][0] as string;
    expect(calledUrl).toContain('media_type=anime'); // 【確認内容】: mediaType→media_type変換を確認
    expect(calledUrl).toContain('page=1'); // 【確認内容】: pageパラメータの変換を確認
    expect(calledUrl).toContain('limit=20'); // 【確認内容】: limitパラメータの変換を確認
  });

  it('TC-IQ-N-03: isFavorite/status/tagId/categoryIdフィルタ付きで呼び出す', async () => {
    // 【テスト目的】: 全フィルタフィールドのキャメルケース→スネークケース変換を確認
    // 【テスト内容】: 全フィールドを指定してuseItemsQueryを実行
    // 【期待される動作】: is_favorite/status/tag_id/category_idがURLに含まれる
    // 🔵 requirements.md「URLクエリパラメータ変換ルール」より

    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, data: [] }),
    } as Response);
    vi.stubGlobal('fetch', fetchMock);

    const filters: ItemListFilters = {
      isFavorite: true,
      status: 'completed',
      tagId: 't1',
      categoryId: 'c1',
    };
    const wrapper = createWrapper(queryClient);
    const { result } = renderHook(() => useItemsQuery(filters), { wrapper });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    const calledUrl: string = fetchMock.mock.calls[0][0] as string;
    expect(calledUrl).toContain('is_favorite=true'); // 【確認内容】: isFavorite→is_favorite変換を確認
    expect(calledUrl).toContain('status=completed'); // 【確認内容】: statusパラメータを確認
    expect(calledUrl).toContain('tag_id=t1'); // 【確認内容】: tagId→tag_id変換を確認
    expect(calledUrl).toContain('category_id=c1'); // 【確認内容】: categoryId→category_id変換を確認
  });

  it('TC-IQ-N-06: queryKeyが["items", filters]形式である', async () => {
    // 【テスト目的】: TanStack Queryのキャッシュキー形式が正しいことを確認
    // 【テスト内容】: useItemsQueryのqueryKeyをQueryClientキャッシュから検証
    // 【期待される動作】: queryKey=['items', filters] でキャッシュされる
    // 🔵 TASK-0009.md 完了条件より

    mockFetchSuccess([mockItem]);

    const filters: ItemListFilters = { mediaType: 'manga' };
    const wrapper = createWrapper(queryClient);
    const { result } = renderHook(() => useItemsQuery(filters), { wrapper });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    // 【結果検証】: QueryClientのキャッシュにqueryKeyが正しい形式で存在することを確認
    const queries = queryClient.getQueriesData({ queryKey: ['items', filters] });
    expect(queries.length).toBeGreaterThan(0); // 【確認内容】: ['items', filters]キーでキャッシュされていることを確認
  });

  it('TC-IQ-N-11: undefinedフィールドはクエリパラメータに含まれない', async () => {
    // 【テスト目的】: undefinedのフィルタフィールドがURLに含まれないことを確認
    // 【テスト内容】: tagId=undefinedを含むfiltersでuseItemsQueryを実行
    // 【期待される動作】: tag_idパラメータがURLに含まれない
    // 🔵 TASK-0009.md 注意事項より

    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, data: [] }),
    } as Response);
    vi.stubGlobal('fetch', fetchMock);

    const filters: ItemListFilters = { mediaType: 'anime', tagId: undefined, page: 1 };
    const wrapper = createWrapper(queryClient);
    const { result } = renderHook(() => useItemsQuery(filters), { wrapper });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    const calledUrl: string = fetchMock.mock.calls[0][0] as string;
    expect(calledUrl).toContain('media_type=anime'); // 【確認内容】: 有効なパラメータは含まれることを確認
    expect(calledUrl).not.toContain('tag_id'); // 【確認内容】: undefinedフィールドはURLに含まれないことを確認
  });

  it('TC-IQ-E-01: APIエラー時にApiClientErrorを返す', async () => {
    // 【テスト目的】: 一覧取得時のエラーがApiClientErrorとして伝播することを確認
    // 【テスト内容】: APIエラーレスポンスを返すモックでuseItemsQueryを実行
    // 【期待される動作】: query.errorがApiClientErrorインスタンスで、code===SERVER_ERRORである
    // 🔵 TASK-0009.md「APIエラー時、各フックのerrorにApiClientErrorがそのまま伝播する」より

    mockFetchError('SERVER_ERROR', 'サーバーエラー');

    const filters: ItemListFilters = {};
    const wrapper = createWrapper(queryClient);
    const { result } = renderHook(() => useItemsQuery(filters), { wrapper });

    await waitFor(() => expect(result.current.isError).toBe(true));

    // 【結果検証】: errorがApiClientErrorインスタンスであることを確認
    expect(result.current.error).toBeInstanceOf(ApiClientError); // 【確認内容】: エラーがApiClientErrorであることを確認
    expect((result.current.error as ApiClientError).code).toBe('SERVER_ERROR'); // 【確認内容】: エラーコードが正しく伝播することを確認
  });
});

describe('useItemQuery', () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    queryClient = createQueryClient();
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    queryClient.clear();
  });

  it('TC-IQ-N-05: 存在するidでGET /items/:idを呼び出しItemを返す', async () => {
    // 【テスト目的】: 詳細取得APIの正常動作を確認
    // 【テスト内容】: id='item-001'でuseItemQueryを実行し、Itemが返ることを検証
    // 【期待される動作】: GET /items/item-001が呼び出され、data.dataにItemが返る
    // 🔵 TASK-0009.md テストケース3・完了条件より

    mockFetchSuccess(mockItem);

    const wrapper = createWrapper(queryClient);
    const { result } = renderHook(() => useItemQuery('item-001'), { wrapper });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    // 【結果検証】: 正常なItemデータが返ることを確認
    expect(result.current.data?.data).toBeDefined(); // 【確認内容】: dataが存在することを確認
    expect((result.current.data?.data as Item).id).toBe('item-001'); // 【確認内容】: 取得したItemのidが一致することを確認
  });

  it('TC-IQ-N-07: queryKeyが["items", "detail", id]形式である', async () => {
    // 【テスト目的】: 詳細クエリのキャッシュキー形式を確認
    // 【テスト内容】: useItemQueryのqueryKeyをQueryClientキャッシュから検証
    // 【期待される動作】: queryKey=['items', 'detail', 'item-001']でキャッシュされる
    // 🔵 TASK-0009.md 完了条件より

    mockFetchSuccess(mockItem);

    const wrapper = createWrapper(queryClient);
    const { result } = renderHook(() => useItemQuery('item-001'), { wrapper });

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    const queries = queryClient.getQueriesData({ queryKey: ['items', 'detail', 'item-001'] });
    expect(queries.length).toBeGreaterThan(0); // 【確認内容】: ['items','detail','item-001']キーでキャッシュされることを確認
  });

  it('TC-IQ-B-01: idが空文字のときfetchしない（enabled=false）', async () => {
    // 【テスト目的】: id未指定時のクエリスキップ動作を確認
    // 【テスト内容】: id=''でuseItemQueryを実行し、fetchが呼ばれないことを検証
    // 【期待される動作】: fetchが呼ばれず、data===undefined
    // 🔵 TASK-0009.md 実装詳細「enabled: !!id」より

    const fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);

    const wrapper = createWrapper(queryClient);
    const { result } = renderHook(() => useItemQuery(''), { wrapper });

    // 【実際の処理実行】: 少し待機してfetchが呼ばれないことを確認
    await new Promise((r) => setTimeout(r, 100));

    expect(fetchMock).not.toHaveBeenCalled(); // 【確認内容】: id=''のときfetchが呼ばれないことを確認
    expect(result.current.data).toBeUndefined(); // 【確認内容】: dataがundefinedであることを確認
  });

  it('TC-IQ-E-02: APIエラー時にApiClientErrorを返す', async () => {
    // 【テスト目的】: 詳細取得時のエラーがApiClientErrorとして伝播することを確認
    // 【テスト内容】: NOT_FOUNDエラーレスポンスを返すモックでuseItemQueryを実行
    // 【期待される動作】: query.errorがApiClientErrorで、code===NOT_FOUND
    // 🔵 TASK-0009.md 完了条件より

    mockFetchError('NOT_FOUND', 'アイテムが見つかりません');

    const wrapper = createWrapper(queryClient);
    const { result } = renderHook(() => useItemQuery('not-found'), { wrapper });

    await waitFor(() => expect(result.current.isError).toBe(true));

    expect(result.current.error).toBeInstanceOf(ApiClientError); // 【確認内容】: ApiClientErrorが伝播することを確認
    expect((result.current.error as ApiClientError).code).toBe('NOT_FOUND'); // 【確認内容】: エラーコードがNOT_FOUNDであることを確認
  });

  it('TC-IQ-E-05: ネットワークエラー時にNETWORK_ERRORコードのApiClientErrorが伝播する', async () => {
    // 【テスト目的】: ネットワーク障害時のエラーコードを確認
    // 【テスト内容】: fetchが例外をthrowする状況でuseItemQueryを実行
    // 【期待される動作】: query.error.code===NETWORK_ERROR
    // 🔵 client.test.ts パターンより

    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('connection refused')));

    const wrapper = createWrapper(queryClient);
    const { result } = renderHook(() => useItemQuery('item-001'), { wrapper });

    await waitFor(() => expect(result.current.isError).toBe(true));

    expect((result.current.error as ApiClientError).code).toBe('NETWORK_ERROR'); // 【確認内容】: ネットワークエラー時にNETWORK_ERRORコードが設定されることを確認
  });
});

describe('useDeleteItemMutation', () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    queryClient = createQueryClient();
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    queryClient.clear();
  });

  it('TC-IQ-N-08: 削除成功時にinvalidateQueriesが["items"]で呼ばれる', async () => {
    // 【テスト目的】: 削除後の一覧キャッシュ無効化を確認
    // 【テスト内容】: DELETE /items/item-001成功後にinvalidateQueriesが呼ばれることを検証
    // 【期待される動作】: queryClient.invalidateQueries({queryKey:['items']})が呼ばれる
    // 🔵 TASK-0009.md テストケース4・完了条件より

    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: async () => ({ success: true, data: null }),
      } as Response)
    );
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');

    const wrapper = createWrapper(queryClient);
    const { result } = renderHook(() => useDeleteItemMutation(), { wrapper });

    // 【実際の処理実行】: 削除ミューテーションを実行
    result.current.mutate('item-001');

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    // 【結果検証】: invalidateQueriesが['items']キーで呼ばれたことを確認
    expect(invalidateSpy).toHaveBeenCalledWith(
      expect.objectContaining({ queryKey: ['items'] })
    ); // 【確認内容】: ['items']キーでキャッシュが無効化されることを確認
  });

  it('TC-IQ-B-04: 削除成功後にinvalidateQueriesが1回のみ呼ばれる', async () => {
    // 【テスト目的】: onSuccess内の副作用が二重実行されないことを確認
    // 【テスト内容】: mutateを1回実行し、invalidateQueriesの呼び出し回数を確認
    // 【期待される動作】: invalidateQueriesが['items']で厳密に1回呼ばれる
    // 🔵 TASK-0009.md 完了条件より

    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: async () => ({ success: true, data: null }),
      } as Response)
    );
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');

    const wrapper = createWrapper(queryClient);
    const { result } = renderHook(() => useDeleteItemMutation(), { wrapper });

    result.current.mutate('item-001');

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    const itemsCalls = invalidateSpy.mock.calls.filter(
      (call) =>
        JSON.stringify(call[0]) === JSON.stringify({ queryKey: ['items'] })
    );
    expect(itemsCalls.length).toBe(1); // 【確認内容】: ['items']キーのinvalidateQueriesが厳密に1回のみ呼ばれることを確認
  });

  it('TC-IQ-E-04: APIエラー時にApiClientErrorを返す', async () => {
    // 【テスト目的】: 削除ミューテーションのエラー伝播を確認
    // 【テスト内容】: NOT_FOUNDエラーでDELETE失敗時のerrorを検証
    // 【期待される動作】: mutation.errorがApiClientErrorインスタンス
    // 🔵 requirements.md 制約条件「エラー伝播」より

    mockFetchError('NOT_FOUND', '削除対象が見つかりません');

    const wrapper = createWrapper(queryClient);
    const { result } = renderHook(() => useDeleteItemMutation(), { wrapper });

    result.current.mutate('not-exist');

    await waitFor(() => expect(result.current.isError).toBe(true));

    expect(result.current.error).toBeInstanceOf(ApiClientError); // 【確認内容】: ApiClientErrorが伝播することを確認
  });
});

describe('useUpdateItemStatusMutation', () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    queryClient = createQueryClient();
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    queryClient.clear();
  });

  it('TC-IQ-N-09: 成功時に一覧と詳細の両方のキャッシュが無効化される', async () => {
    // 【テスト目的】: ステータス更新後に一覧・詳細両方のキャッシュが無効化されることを確認
    // 【テスト内容】: mutate({id:'item-001', body:{status:'completed'}})実行後のinvalidateQueriesを検証
    // 【期待される動作】: ['items']と['items','detail','item-001']の両方がinvalidateされる
    // 🔵 TASK-0009.md 完了条件より

    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: async () => ({ success: true, data: { ...mockItem, status: 'completed' } }),
      } as Response)
    );
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');

    const wrapper = createWrapper(queryClient);
    const { result } = renderHook(() => useUpdateItemStatusMutation(), { wrapper });

    const body: UpdateItemStatusRequest = { status: 'completed' };
    result.current.mutate({ id: 'item-001', body });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    // 【結果検証】: 一覧キャッシュの無効化を確認
    expect(invalidateSpy).toHaveBeenCalledWith(
      expect.objectContaining({ queryKey: ['items'] })
    ); // 【確認内容】: 一覧クエリが無効化されることを確認

    // 【結果検証】: 詳細キャッシュの無効化を確認
    expect(invalidateSpy).toHaveBeenCalledWith(
      expect.objectContaining({ queryKey: ['items', 'detail', 'item-001'] })
    ); // 【確認内容】: 詳細クエリも無効化されることを確認
  });

  it('TC-IQ-N-10: 成功時に更新後のItemを返す', async () => {
    // 【テスト目的】: PATCH /items/:id/statusのレスポンスが正しく取得できることを確認
    // 【テスト内容】: ステータス更新後にmutation.dataに更新後Itemが入ることを検証
    // 【期待される動作】: mutation.data.dataに更新後のItemが返る
    // 🔵 requirements.md 2-4「出力」より

    const updatedItem = { ...mockItem, status: 'in_progress' as const };
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: async () => ({ success: true, data: updatedItem }),
      } as Response)
    );

    const wrapper = createWrapper(queryClient);
    const { result } = renderHook(() => useUpdateItemStatusMutation(), { wrapper });

    result.current.mutate({ id: 'item-001', body: { status: 'in_progress' } });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.data?.data).toBeDefined(); // 【確認内容】: dataが存在することを確認
    expect((result.current.data?.data as Item).status).toBe('in_progress'); // 【確認内容】: 更新後のstatusが正しいことを確認
  });

  it('TC-IQ-E-03: バリデーションエラー時にApiClientErrorを伝播する', async () => {
    // 【テスト目的】: 不正なstatusを送信した際のエラー伝播を確認
    // 【テスト内容】: VALIDATION_ERRORレスポンス時のmutation.errorを検証
    // 【期待される動作】: mutation.errorがApiClientErrorでcode===VALIDATION_ERROR
    // 🟡 TASK-0009.md テストケース5より（バックエンドエラーコードは推測）

    mockFetchError('VALIDATION_ERROR', '不正なステータス値です');

    const wrapper = createWrapper(queryClient);
    const { result } = renderHook(() => useUpdateItemStatusMutation(), { wrapper });

    result.current.mutate({ id: 'item-001', body: { status: 'invalid_status' as UpdateItemStatusRequest['status'] } });

    await waitFor(() => expect(result.current.isError).toBe(true));

    expect(result.current.error).toBeInstanceOf(ApiClientError); // 【確認内容】: ApiClientErrorが伝播することを確認
    expect((result.current.error as ApiClientError).code).toBe('VALIDATION_ERROR'); // 【確認内容】: VALIDATION_ERRORコードが正しく設定されることを確認
  });
});
