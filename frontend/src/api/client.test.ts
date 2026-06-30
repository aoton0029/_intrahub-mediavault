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
