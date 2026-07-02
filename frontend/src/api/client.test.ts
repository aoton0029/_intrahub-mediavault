import { describe, it, expect, vi, afterEach } from 'vitest';
import { ApiClientError } from '@/types';
import { apiClient } from './client';

function mockFetchOnce(response: Partial<Response> & { json: () => Promise<unknown> }) {
  vi.stubGlobal(
    'fetch',
    vi.fn().mockResolvedValue({
      ok: true,
      ...response,
    } as Response)
  );
}

describe('apiClient', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.unstubAllEnvs();
  });

  it('成功レスポンス時にdataを返す', async () => {
    mockFetchOnce({ json: async () => ({ success: true, data: { id: '1' } }) });

    const result = await apiClient<{ id: string }>('/items');

    expect(result.data).toEqual({ id: '1' });
    expect(result.pagination).toBeUndefined();
  });

  it('paginationを含むレスポンスの場合はpaginationも返す', async () => {
    const pagination = { page: 1, limit: 20, total: 1 };
    mockFetchOnce({ json: async () => ({ success: true, data: [{ id: '1' }], pagination }) });

    const result = await apiClient<{ id: string }[]>('/items');

    expect(result.pagination).toEqual(pagination);
  });

  it('エラーレスポンス時にApiClientErrorをthrowする', async () => {
    mockFetchOnce({
      json: async () => ({
        success: false,
        error: { code: 'VALIDATION_ERROR', message: '不正な入力です' },
      }),
    });

    await expect(apiClient('/items')).rejects.toMatchObject({
      code: 'VALIDATION_ERROR',
      message: '不正な入力です',
    });
    await expect(apiClient('/items')).rejects.toBeInstanceOf(ApiClientError);
  });

  it('fetch自体が失敗した場合NETWORK_ERRORでApiClientErrorをthrowする', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('connection refused')));

    await expect(apiClient('/items')).rejects.toMatchObject({
      code: 'NETWORK_ERROR',
    });
    await expect(apiClient('/items')).rejects.toBeInstanceOf(ApiClientError);
  });

  it('レスポンスのJSON parseに失敗した場合PARSE_ERRORでApiClientErrorをthrowする', async () => {
    mockFetchOnce({
      json: async () => {
        throw new Error('invalid json');
      },
    });

    await expect(apiClient('/items')).rejects.toMatchObject({ code: 'PARSE_ERROR' });
  });

  it('VITE_API_BASE_URL未設定時はデフォルトURLにリクエストする', async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, data: {} }),
    } as Response);
    vi.stubGlobal('fetch', fetchMock);

    await apiClient('/items');

    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining('/items'),
      expect.any(Object)
    );
  });

  it('デフォルトで相対パス /api/v1 をベースにリクエストする', async () => {
    // 【テスト目的】: BASE_URLのデフォルト値が絶対URLではなく相対パスであることを確認する
    // 【テスト内容】: VITE_API_BASE_URL未設定時にapiClientが呼び出すfetchのURLを検証する
    // 【期待される動作】: fetchが '/api/v1/items' という相対パスで呼び出される
    // 🔵 信頼性レベル: TASK-0003要件定義・テストケースより
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, data: {} }),
    } as Response);
    vi.stubGlobal('fetch', fetchMock);

    // 【実際の処理実行】: itemsエンドポイントへのGETリクエストを実行
    await apiClient('/items');

    // 【結果検証】: リクエスト先が同一オリジンの相対パスであることを確認
    // 【検証項目】: fetchの第一引数が絶対URLでなく相対パス '/api/v1/items' であること 🔵
    expect(fetchMock).toHaveBeenCalledWith('/api/v1/items', expect.anything());
  });

  it('VITE_API_BASE_URL設定時はその値が優先される', async () => {
    // 【テスト目的】: 環境変数によるBASE_URL上書きの既存挙動が維持されていることを確認する
    // 【テスト内容】: VITE_API_BASE_URLを設定した状態でモジュールを再読込しapiClientを呼び出す
    // 【期待される動作】: fetchが環境変数の値をベースにしたURLで呼び出される
    // 🟡 信頼性レベル: 既存実装の `??` フォールバックからの妥当な推測
    // 【テストデータ準備】: BASE_URLはモジュールトップレベルで評価されるため、
    // 環境変数を反映するにはモジュールキャッシュをリセットして再importする必要がある
    vi.stubEnv('VITE_API_BASE_URL', 'https://example.com/api/v1');
    vi.resetModules();
    const { apiClient: apiClientWithEnv } = await import('./client');

    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, data: {} }),
    } as Response);
    vi.stubGlobal('fetch', fetchMock);

    // 【実際の処理実行】: itemsエンドポイントへのGETリクエストを実行
    await apiClientWithEnv('/items');

    // 【結果検証】: 環境変数の値がベースURLとして使用されることを確認
    // 【検証項目】: fetchの第一引数が環境変数由来のURLであること 🟡
    expect(fetchMock).toHaveBeenCalledWith(
      'https://example.com/api/v1/items',
      expect.anything()
    );
  });

  it('POSTリクエスト時にbodyをJSON化しContent-Typeを設定する', async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, data: {} }),
    } as Response);
    vi.stubGlobal('fetch', fetchMock);

    await apiClient('/items', { method: 'POST', body: { title: 'foo' } });

    expect(fetchMock).toHaveBeenCalledWith(
      expect.any(String),
      expect.objectContaining({
        method: 'POST',
        body: JSON.stringify({ title: 'foo' }),
        headers: expect.objectContaining({ 'Content-Type': 'application/json' }),
      })
    );
  });
});
