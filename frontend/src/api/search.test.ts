/**
 * TASK-0015: 外部API検索フック テストファイル (Redフェーズ)
 * 🔵 docs/tasks/frontend-collection-ui/TASK-0015.md 完了条件より
 */
import { describe, it, expect, vi, afterEach } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { createElement } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ApiClientError } from '@/types';
import type { ExternalSearchQuery, ExternalSearchResultItem, ImportItemRequest, Item } from '@/types';
import { useExternalSearchQuery, useImportItemMutation } from './search';

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
    })
  );
}

function mockFetchError(code: string, message: string, status: number) {
  vi.stubGlobal(
    'fetch',
    vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: false, error: { code, message } }),
      status,
    })
  );
}

afterEach(() => {
  vi.restoreAllMocks();
});

// ===== テストデータ =====

const mockSearchResults: ExternalSearchResultItem[] = [
  {
    externalId: 'ext-001',
    title: '進撃の巨人',
    coverImageUrl: 'https://example.com/cover.jpg',
    releaseDate: '2013-04-07',
    raw: { id: 'ext-001' },
  },
];

const mockItem: Item = {
  id: 'item-001',
  title: '進撃の巨人',
  mediaType: 'anime',
  status: 'not_started',
  source: 'api',
  isFavorite: false,
  details: { genreList: [] },
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: '2026-01-01T00:00:00Z',
};

// ===== テストケース =====

describe('useExternalSearchQuery', () => {
  it('テストケース1: 検索成功で結果一覧が返る', async () => {
    // Given
    mockFetchSuccess(mockSearchResults);
    const queryClient = createQueryClient();
    const wrapper = createWrapper(queryClient);
    const query: ExternalSearchQuery = { mediaType: 'anime', q: '進撃' };

    // When
    const { result } = renderHook(() => useExternalSearchQuery(query), { wrapper });

    // Then
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toEqual(mockSearchResults);

    const fetchMock = vi.mocked(fetch);
    expect(fetchMock).toHaveBeenCalledTimes(1);
    const calledUrl = fetchMock.mock.calls[0][0] as string;
    expect(calledUrl).toContain('/items/search');
    expect(calledUrl).toContain('media_type=anime');
    expect(calledUrl).toContain('q=%E9%80%B2%E6%92%83');
  });

  it('テストケース2: qが空文字の場合クエリが実行されない', async () => {
    // Given
    const fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);
    const queryClient = createQueryClient();
    const wrapper = createWrapper(queryClient);
    const query: ExternalSearchQuery = { mediaType: 'anime', q: '' };

    // When
    const { result } = renderHook(() => useExternalSearchQuery(query), { wrapper });

    // Then: fetchは呼ばれず、クエリはpending状態
    await new Promise((r) => setTimeout(r, 50));
    expect(fetchMock).not.toHaveBeenCalled();
    expect(result.current.fetchStatus).toBe('idle');
  });

  it('テストケース3: API_KEY_NOT_CONFIGUREDエラーが判定可能', async () => {
    // Given
    mockFetchError('API_KEY_NOT_CONFIGURED', 'APIキーが設定されていません', 422);
    const queryClient = createQueryClient();
    const wrapper = createWrapper(queryClient);
    const query: ExternalSearchQuery = { mediaType: 'anime', q: '進撃' };

    // When
    const { result } = renderHook(() => useExternalSearchQuery(query), { wrapper });

    // Then
    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error).toBeInstanceOf(ApiClientError);
    expect((result.current.error as ApiClientError).code).toBe('API_KEY_NOT_CONFIGURED');
  });

  it('テストケース4: EXTERNAL_API_TIMEOUTエラーが判定可能', async () => {
    // Given
    mockFetchError('EXTERNAL_API_TIMEOUT', '外部APIがタイムアウトしました', 502);
    const queryClient = createQueryClient();
    const wrapper = createWrapper(queryClient);
    const query: ExternalSearchQuery = { mediaType: 'anime', q: '進撃' };

    // When
    const { result } = renderHook(() => useExternalSearchQuery(query), { wrapper });

    // Then
    await waitFor(() => expect(result.current.isError).toBe(true));
    expect((result.current.error as ApiClientError).code).toBe('EXTERNAL_API_TIMEOUT');
  });
});

describe('useImportItemMutation', () => {
  it('テストケース5: インポート成功時に一覧キャッシュが無効化される', async () => {
    // Given
    mockFetchSuccess(mockItem);
    const queryClient = createQueryClient();
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const wrapper = createWrapper(queryClient);
    const request: ImportItemRequest = {
      mediaType: 'anime',
      externalId: 'ext-001',
      raw: { id: 'ext-001' },
    };

    // When
    const { result } = renderHook(() => useImportItemMutation(), { wrapper });
    act(() => result.current.mutate(request));

    // Then
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toEqual(mockItem);
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['items'] });
  });
});
