import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { apiRequest, ApiClientError } from './apiClient';

/**
 * apiClient.ts のユニットテスト
 * fetchはモック化する。
 */

function mockFetchOnce(response: {
  ok: boolean;
  status?: number;
  json?: () => Promise<unknown>;
  jsonRejects?: boolean;
}) {
  const jsonImpl = response.jsonRejects
    ? () => Promise.reject(new SyntaxError('Unexpected token in JSON'))
    : (response.json ?? (() => Promise.resolve({})));

  const fetchMock = vi.fn().mockResolvedValue({
    ok: response.ok,
    status: response.status ?? (response.ok ? 200 : 400),
    json: jsonImpl,
  });
  vi.stubGlobal('fetch', fetchMock);
  return fetchMock;
}

describe('apiClient', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  describe('正常系', () => {
    it('ApiOk<T> から data を正しく抽出する', async () => {
      const item = { id: '1', title: 'Sample Item' };
      const fetchMock = mockFetchOnce({
        ok: true,
        json: () => Promise.resolve({ success: true, data: item }),
      });

      const result = await apiRequest<typeof item>('/items/1');

      expect(result).toEqual(item);
      expect(fetchMock).toHaveBeenCalledWith(
        '/api/v1/items/1',
        expect.objectContaining({}),
      );
    });

    it('PaginatedOk<T> から data と pagination を正しく抽出する', async () => {
      const items = [{ id: '1' }, { id: '2' }];
      const pagination = { page: 1, limit: 20, total: 2 };
      mockFetchOnce({
        ok: true,
        json: () =>
          Promise.resolve({ success: true, data: items, pagination }),
      });

      const result = await apiRequest<
        { id: string }[],
        { page: number; limit: number; total: number }
      >('/items');

      expect(result).toEqual({ data: items, pagination });
    });

    it('リクエストオプション（method, bodyなど）が fetch にそのまま渡される', async () => {
      const fetchMock = mockFetchOnce({
        ok: true,
        json: () => Promise.resolve({ success: true, data: { ok: true } }),
      });

      await apiRequest('/items/1/status', {
        method: 'PATCH',
        body: JSON.stringify({ status: 'completed' }),
      });

      expect(fetchMock).toHaveBeenCalledWith(
        '/api/v1/items/1/status',
        expect.objectContaining({
          method: 'PATCH',
          body: JSON.stringify({ status: 'completed' }),
        }),
      );
    });
  });

  describe('異常系', () => {
    it('success: false のエラーレスポンス受信時、code と message を保持した ApiClientError をthrowする', async () => {
      mockFetchOnce({
        ok: false,
        status: 404,
        json: () =>
          Promise.resolve({
            success: false,
            error: { code: 'ITEM_NOT_FOUND', message: 'Item not found' },
          }),
      });

      await expect(apiRequest('/items/does-not-exist')).rejects.toMatchObject(
        {
          code: 'ITEM_NOT_FOUND',
          message: 'Item not found',
        },
      );

      await expect(
        apiRequest('/items/does-not-exist'),
      ).rejects.toBeInstanceOf(ApiClientError);
    });

    it('VALIDATION_ERROR などその他のエラーコードでも同様にthrowする', async () => {
      mockFetchOnce({
        ok: false,
        status: 422,
        json: () =>
          Promise.resolve({
            success: false,
            error: {
              code: 'VALIDATION_ERROR',
              message: 'title is required',
            },
          }),
      });

      await expect(apiRequest('/items')).rejects.toMatchObject({
        code: 'VALIDATION_ERROR',
        message: 'title is required',
      });
    });

    it('ネットワークエラー（fetch自体がreject）時にエラーが呼び出し側に伝播する', async () => {
      const fetchMock = vi
        .fn()
        .mockRejectedValue(new TypeError('Failed to fetch'));
      vi.stubGlobal('fetch', fetchMock);

      await expect(apiRequest('/items')).rejects.toThrow();
    });

    it('不正なJSONレスポンス（jsonのparseが失敗）時にエラーとして正規化される', async () => {
      mockFetchOnce({
        ok: true,
        jsonRejects: true,
      });

      await expect(apiRequest('/items')).rejects.toThrow();
    });
  });

  describe('境界値', () => {
    it('pathの先頭にスラッシュがなくてもbase URLと正しく結合される', async () => {
      const fetchMock = mockFetchOnce({
        ok: true,
        json: () => Promise.resolve({ success: true, data: {} }),
      });

      await apiRequest('items');

      expect(fetchMock).toHaveBeenCalledWith(
        '/api/v1/items',
        expect.objectContaining({}),
      );
    });

    it('optionsを省略してもデフォルトで呼び出せる', async () => {
      mockFetchOnce({
        ok: true,
        json: () => Promise.resolve({ success: true, data: 'ok' }),
      });

      const result = await apiRequest<string>('/health');
      expect(result).toBe('ok');
    });
  });
});
