/**
 * TASK-0019: relations APIフック テストファイル (Redフェーズ)
 * 🔵 docs/tasks/frontend-collection-ui/TASK-0019.md REQ-010 より
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { createElement } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ApiClientError } from '@/types';
import type { ItemRelation } from '@/types';
import {
  useItemRelationsQuery,
  useCreateItemRelationMutation,
  useDeleteItemRelationMutation,
} from './relations';

// ===== テストユーティリティ =====

function createQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });
}

function createWrapper(queryClient: QueryClient) {
  return ({ children }: { children: React.ReactNode }) =>
    createElement(QueryClientProvider, { client: queryClient }, children);
}

function mockFetchSuccess(data: unknown) {
  vi.stubGlobal(
    'fetch',
    vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, data }),
    } as Response)
  );
}

function mockFetchError(code: string, message: string) {
  vi.stubGlobal(
    'fetch',
    vi.fn().mockResolvedValue({
      ok: false,
      json: async () => ({ success: false, error: { code, message } }),
    } as Response)
  );
}

// ===== テストデータ =====

const mockRelation: ItemRelation = {
  id: 'rel-001',
  itemId: 'item-001',
  relatedItemId: 'item-002',
  relationType: 'reference',
  createdAt: '2026-07-01T00:00:00Z',
};

const mockDlcRelation: ItemRelation = {
  id: 'rel-002',
  itemId: 'item-game',
  relatedItemId: 'item-dlc',
  relationType: 'dlc',
  createdAt: '2026-07-01T00:00:00Z',
};

// ===== useItemRelationsQuery テスト =====

describe('useItemRelationsQuery', () => {
  let queryClient: QueryClient;
  let wrapper: ReturnType<typeof createWrapper>;

  beforeEach(() => {
    queryClient = createQueryClient();
    wrapper = createWrapper(queryClient);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    queryClient.clear();
  });

  it('TC-REL-N-01: GET /items/:id/relations で関連付け一覧を取得する', async () => {
    // 【テスト目的】: useItemRelationsQueryがGET /items/:id/relationsを正しく呼び出すことを確認
    // 【期待される動作】: fetchが /items/item-001/relations を含むURLで呼ばれ、一覧が返る
    // 🔵 信頼性レベル: REQ-010より
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, data: [mockRelation] }),
    } as Response);
    vi.stubGlobal('fetch', fetchMock);

    const { result } = renderHook(() => useItemRelationsQuery('item-001'), { wrapper });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    const calledUrl = fetchMock.mock.calls[0][0] as string;
    expect(calledUrl).toContain('/items/item-001/relations'); // 【確認内容】: 正しいエンドポイントURLが呼ばれる 🔵
    expect(result.current.data?.data).toEqual([mockRelation]); // 【確認内容】: 一覧データが正しく返る 🔵
    expect(result.current.isError).toBe(false); // 【確認内容】: エラー状態でない 🔵
  });

  it('TC-REL-N-05: queryKey が ["items", "relations", itemId] 形式である', async () => {
    // 【テスト目的】: queryKeyが正しい形式でキャッシュされることを確認
    // 🔵 信頼性レベル: architecture.md「リソース別フック」方針より
    mockFetchSuccess([mockRelation]);

    const { result } = renderHook(() => useItemRelationsQuery('item-001'), { wrapper });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    const queries = queryClient.getQueriesData({ queryKey: ['items', 'relations', 'item-001'] });
    expect(queries.length).toBeGreaterThan(0); // 【確認内容】: 指定queryKeyでキャッシュが作成されている 🔵
  });

  it('TC-REL-B-01: itemId が空文字のとき fetch しない', async () => {
    // 【テスト目的】: enabled: !!itemId によって空IDの場合にfetchが発生しないことを確認
    // 🔵 信頼性レベル: groups.ts パターンより
    const fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);

    renderHook(() => useItemRelationsQuery(''), { wrapper });
    await new Promise((r) => setTimeout(r, 100));

    expect(fetchMock).not.toHaveBeenCalled(); // 【確認内容】: 空文字IDではfetchが実行されない 🔵
  });

  it('TC-REL-E-01: APIエラー時に isError=true を返す', async () => {
    // 【テスト目的】: APIエラー時にisErrorがtrueになりApiClientErrorが返ることを確認
    // 🔵 信頼性レベル: groups.test.ts エラーパターンより
    mockFetchError('SERVER_ERROR', 'サーバーエラー');

    const { result } = renderHook(() => useItemRelationsQuery('item-001'), { wrapper });
    await waitFor(() => expect(result.current.isError).toBe(true));

    expect(result.current.error).toBeInstanceOf(ApiClientError); // 【確認内容】: ApiClientErrorのインスタンスが返る 🔵
  });
});

// ===== useCreateItemRelationMutation テスト =====

describe('useCreateItemRelationMutation', () => {
  let queryClient: QueryClient;
  let wrapper: ReturnType<typeof createWrapper>;

  beforeEach(() => {
    queryClient = createQueryClient();
    wrapper = createWrapper(queryClient);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    queryClient.clear();
  });

  it('TC-REL-N-02: POST /item-relations で関連付けを作成しキャッシュ無効化する', async () => {
    // 【テスト目的】: 関連付け作成後に当該アイテムの詳細クエリが無効化されることを確認
    // 【期待される動作】: POST成功後に['items', itemId] および ['items','relations',itemId]キャッシュが無効化される
    // 🔵 信頼性レベル: REQ-010・TASK-0019.md「当該アイテムの詳細クエリを無効化」より
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, data: mockRelation }),
    } as Response));

    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(
      () => useCreateItemRelationMutation('item-001'),
      { wrapper }
    );

    result.current.mutate({ relatedItemId: 'item-002', relationType: 'reference' });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    // 【結果検証】: 詳細クエリとリレーション一覧クエリの両方が無効化される
    expect(invalidateSpy).toHaveBeenCalledWith(
      expect.objectContaining({ queryKey: ['items', 'item-001'] })
    ); // 【確認内容】: アイテム詳細キャッシュが無効化される 🔵
    expect(invalidateSpy).toHaveBeenCalledWith(
      expect.objectContaining({ queryKey: ['items', 'relations', 'item-001'] })
    ); // 【確認内容】: 関連付け一覧キャッシュが無効化される 🔵
  });

  it('TC-REL-N-06: dlc タイプで関連付けを作成できる', async () => {
    // 【テスト目的】: relationType=dlcでPOSTが正しく呼ばれることを確認
    // 🔵 信頼性レベル: REQ-018/103 DLC関連付けより
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, data: mockDlcRelation }),
    } as Response);
    vi.stubGlobal('fetch', fetchMock);

    const { result } = renderHook(
      () => useCreateItemRelationMutation('item-game'),
      { wrapper }
    );

    result.current.mutate({ relatedItemId: 'item-dlc', relationType: 'dlc' });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    const body = JSON.parse(fetchMock.mock.calls[0][1]?.body as string);
    expect(body.relation_type).toBe('dlc'); // 【確認内容】: dlcタイプがリクエストボディに含まれる 🔵
  });

  it('TC-REL-E-02: 関連付け作成失敗時に isError=true でキャッシュ無効化しない', async () => {
    // 【テスト目的】: ミューテーション失敗時のキャッシュ汚染がないことを確認
    // 🔵 信頼性レベル: groups.test.ts エラーパターンより
    mockFetchError('VALIDATION_ERROR', 'バリデーションエラー');

    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(
      () => useCreateItemRelationMutation('item-001'),
      { wrapper }
    );

    result.current.mutate({ relatedItemId: 'item-002', relationType: 'reference' });
    await waitFor(() => expect(result.current.isError).toBe(true));

    expect(invalidateSpy).not.toHaveBeenCalled(); // 【確認内容】: エラー時にキャッシュ無効化されない 🔵
  });
});

// ===== useDeleteItemRelationMutation テスト =====

describe('useDeleteItemRelationMutation', () => {
  let queryClient: QueryClient;
  let wrapper: ReturnType<typeof createWrapper>;

  beforeEach(() => {
    queryClient = createQueryClient();
    wrapper = createWrapper(queryClient);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    queryClient.clear();
  });

  it('TC-REL-N-03: DELETE /item-relations/:id で関連付けを削除しキャッシュ無効化する', async () => {
    // 【テスト目的】: 関連付け削除後にアイテム詳細クエリが無効化されることを確認
    // 【期待される動作】: DELETE成功後に['items', itemId] および ['items','relations',itemId]キャッシュが無効化される
    // 🔵 信頼性レベル: REQ-010より
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, data: null }),
    } as Response));

    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(
      () => useDeleteItemRelationMutation('item-001'),
      { wrapper }
    );

    result.current.mutate('rel-001');
    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    const fetchMock = vi.mocked(global.fetch);
    const calledUrl = fetchMock.mock.calls[0][0] as string;
    expect(calledUrl).toContain('/item-relations/rel-001'); // 【確認内容】: DELETE /item-relations/:id が呼ばれる 🔵
    expect(invalidateSpy).toHaveBeenCalledWith(
      expect.objectContaining({ queryKey: ['items', 'item-001'] })
    ); // 【確認内容】: アイテム詳細キャッシュが無効化される 🔵
    expect(invalidateSpy).toHaveBeenCalledWith(
      expect.objectContaining({ queryKey: ['items', 'relations', 'item-001'] })
    ); // 【確認内容】: 関連付け一覧キャッシュが無効化される 🔵
  });

  it('TC-REL-E-03: 関連付け削除失敗時に isError=true でキャッシュ無効化しない', async () => {
    // 【テスト目的】: 削除ミューテーション失敗時のキャッシュ汚染がないことを確認
    // 🔵 信頼性レベル: groups.test.ts エラーパターンより
    mockFetchError('NOT_FOUND', 'Not found');

    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(
      () => useDeleteItemRelationMutation('item-001'),
      { wrapper }
    );

    result.current.mutate('rel-999');
    await waitFor(() => expect(result.current.isError).toBe(true));

    expect(invalidateSpy).not.toHaveBeenCalled(); // 【確認内容】: エラー時にキャッシュ無効化されない 🔵
  });
});
