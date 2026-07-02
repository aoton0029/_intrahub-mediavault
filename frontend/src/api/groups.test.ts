/**
 * TASK-0018: groups/episodes APIフック テストファイル (Redフェーズ)
 * 🔵 docs/tasks/frontend-collection-ui/TASK-0018.md REQ-015/016 より
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { createElement } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ApiClientError } from '@/types';
import type { ItemGroup, ItemEpisode, CreateGroupRequest, CreateEpisodeRequest } from '@/types';
import {
  useItemGroupsQuery,
  useCreateGroupMutation,
  useGroupEpisodesQuery,
  useCreateEpisodeMutation,
} from './groups';

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

function mockFetchSuccess(data: unknown) {
  // 【テストデータ準備】: 成功レスポンスのfetchモックを設定
  vi.stubGlobal(
    'fetch',
    vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, data }),
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

// ===== テストデータ =====

const mockGroup: ItemGroup = {
  id: 'group-001',
  itemId: 'item-001',
  groupType: 'season',
  groupName: 'Season 1',
  displayOrder: 1,
  createdAt: '2026-07-01T00:00:00Z',
  updatedAt: '2026-07-01T00:00:00Z',
};

const mockEpisode: ItemEpisode = {
  id: 'ep-001',
  groupId: 'group-001',
  episodeNumber: 1,
  title: 'Episode 1',
  createdAt: '2026-07-01T00:00:00Z',
  updatedAt: '2026-07-01T00:00:00Z',
};

// ===== useItemGroupsQuery テスト =====

describe('useItemGroupsQuery', () => {
  let queryClient: QueryClient;
  let wrapper: ReturnType<typeof createWrapper>;

  beforeEach(() => {
    // 【テスト前準備】: 各テストで独立したQueryClientを生成
    queryClient = createQueryClient();
    wrapper = createWrapper(queryClient);
  });

  afterEach(() => {
    // 【テスト後処理】: グローバルモックとQueryClientキャッシュをリセット
    vi.unstubAllGlobals();
    queryClient.clear();
  });

  it('TC-GRP-N-01: GET /items/:id/groups でグループ一覧を取得する', async () => {
    // 【テスト目的】: useItemGroupsQueryがGET /items/:id/groupsを正しく呼び出すことを確認
    // 【テスト内容】: itemId='item-001'でフックを実行し、正しいURLとデータが返ることを検証
    // 【期待される動作】: fetchが /items/item-001/groups を含むURLで呼ばれ、グループ一覧が返る
    // 🔵 信頼性レベル: REQ-015 GET /items/:id/groups より

    // 【テストデータ準備】: グループ1件の成功レスポンスをモック
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, data: [mockGroup] }),
    } as Response);
    vi.stubGlobal('fetch', fetchMock);

    // 【実際の処理実行】: useItemGroupsQueryフックを実行
    const { result } = renderHook(() => useItemGroupsQuery('item-001'), { wrapper });

    // 【結果検証】: ローディング完了を待機してURLとデータを確認
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    const calledUrl = fetchMock.mock.calls[0][0] as string;
    expect(calledUrl).toContain('/items/item-001/groups'); // 【確認内容】: 正しいエンドポイントURLが呼ばれる 🔵
    expect(result.current.data?.data).toEqual([mockGroup]); // 【確認内容】: グループ一覧データが正しく返る 🔵
    expect(result.current.isError).toBe(false); // 【確認内容】: エラー状態でないことを確認 🔵
  });

  it('TC-GRP-N-05: queryKey が ["items", "groups", itemId] 形式である', async () => {
    // 【テスト目的】: queryKeyが正しい形式でキャッシュされることを確認
    // 【期待される動作】: ['items', 'groups', 'item-001'] でキャッシュが作成される
    // 🔵 信頼性レベル: architecture.md「リソース別フック」方針より
    mockFetchSuccess([mockGroup]);

    const { result } = renderHook(() => useItemGroupsQuery('item-001'), { wrapper });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    // 【結果検証】: 正しいqueryKeyでキャッシュが存在することを確認
    const queries = queryClient.getQueriesData({ queryKey: ['items', 'groups', 'item-001'] });
    expect(queries.length).toBeGreaterThan(0); // 【確認内容】: 指定queryKeyでキャッシュが作成されている 🔵
  });

  it('TC-GRP-B-01: itemId が空文字のとき fetch しない', async () => {
    // 【テスト目的】: enabled: !!itemId によって空IDの場合にfetchが発生しないことを確認
    // 【期待される動作】: fetchが一度も呼ばれない
    // 🔵 信頼性レベル: note.md 注意事項「enabled: !!itemId」より
    const fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);

    renderHook(() => useItemGroupsQuery(''), { wrapper });

    // 【結果検証】: 一定時間経過後もfetchが呼ばれていないことを確認
    await new Promise((r) => setTimeout(r, 100));
    expect(fetchMock).not.toHaveBeenCalled(); // 【確認内容】: 空文字IDではfetchが実行されない 🔵
  });

  it('TC-GRP-B-03: グループが0件のとき空配列を返す', async () => {
    // 【テスト目的】: グループ0件の場合も正常にハンドリングされることを確認
    // 【期待される動作】: data.dataが空配列でisLoadingがfalseになる
    // 🟡 信頼性レベル: 一般的なAPIパターンから推測
    mockFetchSuccess([]);

    const { result } = renderHook(() => useItemGroupsQuery('item-001'), { wrapper });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(result.current.data?.data).toEqual([]); // 【確認内容】: 空配列が正しく返る 🟡
    expect(result.current.isError).toBe(false); // 【確認内容】: エラー状態にならない 🟡
  });

  it('TC-GRP-E-01: APIエラー時に isError=true を返す', async () => {
    // 【テスト目的】: APIエラー時にisErrorがtrueになりApiClientErrorが返ることを確認
    // 【期待される動作】: isError=true でerrorがApiClientErrorのインスタンス
    // 🔵 信頼性レベル: items.test.ts エラーパターンより
    mockFetchError('SERVER_ERROR', 'サーバーエラー');

    const { result } = renderHook(() => useItemGroupsQuery('item-001'), { wrapper });
    await waitFor(() => expect(result.current.isError).toBe(true));

    expect(result.current.error).toBeInstanceOf(ApiClientError); // 【確認内容】: ApiClientErrorのインスタンスが返る 🔵
    expect((result.current.error as ApiClientError).code).toBe('SERVER_ERROR'); // 【確認内容】: エラーコードが正しく設定されている 🔵
  });
});

// ===== useCreateGroupMutation テスト =====

describe('useCreateGroupMutation', () => {
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

  it('TC-GRP-N-02: POST /items/:id/groups でグループを作成しキャッシュ無効化する', async () => {
    // 【テスト目的】: グループ作成後にキャッシュが無効化されることを確認
    // 【テスト内容】: mutate実行後にinvalidateQueriesが正しいqueryKeyで呼ばれる
    // 【期待される動作】: POST成功後に['items', 'groups', 'item-001']キャッシュが無効化される
    // 🔵 信頼性レベル: REQ-015 POST /items/:id/groups より
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, data: mockGroup }),
    } as Response));

    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useCreateGroupMutation('item-001'), { wrapper });

    // 【実際の処理実行】: グループ作成mutationを実行
    const body: CreateGroupRequest = { groupType: 'season', groupName: 'Season 1', displayOrder: 1 };
    result.current.mutate(body);

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    // 【結果検証】: invalidateQueriesが正しいqueryKeyで呼ばれたことを確認
    expect(invalidateSpy).toHaveBeenCalledWith(
      expect.objectContaining({ queryKey: ['items', 'groups', 'item-001'] })
    ); // 【確認内容】: グループ一覧キャッシュが無効化される 🔵
  });

  it('TC-GRP-E-02: グループ作成API失敗時に isError=true でキャッシュ無効化しない', async () => {
    // 【テスト目的】: ミューテーション失敗時のキャッシュ汚染がないことを確認
    // 【期待される動作】: isError=trueでinvalidateQueriesが呼ばれない
    // 🔵 信頼性レベル: items.test.ts エラーパターンより
    mockFetchError('VALIDATION_ERROR', 'バリデーションエラー');

    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useCreateGroupMutation('item-001'), { wrapper });

    result.current.mutate({ groupType: 'season', groupName: 'Season 1' });
    await waitFor(() => expect(result.current.isError).toBe(true));

    expect(invalidateSpy).not.toHaveBeenCalled(); // 【確認内容】: エラー時にキャッシュ無効化されない 🔵
  });
});

// ===== useGroupEpisodesQuery テスト =====

describe('useGroupEpisodesQuery', () => {
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

  it('TC-GRP-N-03: GET /groups/:group_id/episodes で話数一覧を取得する', async () => {
    // 【テスト目的】: useGroupEpisodesQueryがGET /groups/:group_id/episodesを正しく呼び出す
    // 【期待される動作】: fetchが /groups/group-001/episodes を含むURLで呼ばれ、話数一覧が返る
    // 🔵 信頼性レベル: REQ-016 GET /groups/:group_id/episodes より
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, data: [mockEpisode] }),
    } as Response);
    vi.stubGlobal('fetch', fetchMock);

    const { result } = renderHook(() => useGroupEpisodesQuery('group-001'), { wrapper });
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    const calledUrl = fetchMock.mock.calls[0][0] as string;
    expect(calledUrl).toContain('/groups/group-001/episodes'); // 【確認内容】: 正しいエンドポイントURLが呼ばれる 🔵
    expect(result.current.data?.data).toEqual([mockEpisode]); // 【確認内容】: 話数一覧データが正しく返る 🔵
  });

  it('TC-GRP-B-02: groupId が空文字のとき fetch しない', async () => {
    // 【テスト目的】: enabled: !!groupId によって空IDの場合にfetchが発生しないことを確認
    // 🔵 信頼性レベル: note.md 注意事項「enabled: !!groupId」より
    const fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);

    renderHook(() => useGroupEpisodesQuery(''), { wrapper });
    await new Promise((r) => setTimeout(r, 100));

    expect(fetchMock).not.toHaveBeenCalled(); // 【確認内容】: 空文字groupIdではfetchが実行されない 🔵
  });

  it('TC-GRP-E-03: ネットワークエラー時に isError=true を返す', async () => {
    // 【テスト目的】: ネットワーク障害時のエラーハンドリングを確認
    // 【期待される動作】: isError=true になる
    // 🔵 信頼性レベル: items.test.ts ネットワークエラーパターンより
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('connection refused')));

    const { result } = renderHook(() => useGroupEpisodesQuery('group-001'), { wrapper });
    await waitFor(() => expect(result.current.isError).toBe(true));

    expect(result.current.isError).toBe(true); // 【確認内容】: ネットワークエラーがisError=trueで表現される 🔵
  });
});

// ===== useCreateEpisodeMutation テスト =====

describe('useCreateEpisodeMutation', () => {
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

  it('TC-GRP-N-04: POST /groups/:group_id/episodes で話数を作成しキャッシュ無効化する', async () => {
    // 【テスト目的】: 話数作成後にエピソードキャッシュが無効化されることを確認
    // 【期待される動作】: POST成功後に['groups', 'episodes', 'group-001']キャッシュが無効化される
    // 🔵 信頼性レベル: REQ-016 POST /groups/:group_id/episodes より
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, data: mockEpisode }),
    } as Response));

    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useCreateEpisodeMutation('group-001'), { wrapper });

    const body: CreateEpisodeRequest = { episodeNumber: 1, title: 'Episode 1' };
    result.current.mutate(body);

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(invalidateSpy).toHaveBeenCalledWith(
      expect.objectContaining({ queryKey: ['groups', 'episodes', 'group-001'] })
    ); // 【確認内容】: 話数一覧キャッシュが無効化される 🔵
  });
});
