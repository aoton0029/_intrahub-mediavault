import { describe, it, expect, beforeAll, afterAll, afterEach, vi } from 'vitest';
import React from 'react';
import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { renderHook, waitFor, act } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import {
  useItemsInfiniteQuery,
  useUpdateItemStatus,
  useDeleteItem,
  itemsListQueryKey,
} from './hooks';
import type { Item, ItemFilterState } from './types';

/**
 * TASK-0006 統合テスト
 *
 * mswで /api/v1/items 系エンドポイントをモックし、
 * 一覧取得→次ページ取得→status更新→削除という一連のフローを
 * QueryClientProvider配下で実際のフックを通して検証する。
 */

const navigateMock = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual<typeof import('react-router-dom')>('react-router-dom');
  return {
    ...actual,
    useNavigate: () => navigateMock,
  };
});

function makeItem(id: string, overrides: Partial<Item> = {}): Item {
  return {
    id,
    media_type: 'anime',
    title: `Item ${id}`,
    status: 'not_started',
    is_favorite: false,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    ...overrides,
  };
}

const allItems: Item[] = [
  makeItem('1'),
  makeItem('2'),
  makeItem('3'),
];

const server = setupServer(
  http.get('/api/v1/items', ({ request }) => {
    const url = new URL(request.url);
    const page = Number(url.searchParams.get('page') ?? '1');
    const limit = Number(url.searchParams.get('limit') ?? '20');
    const start = (page - 1) * limit;
    const pageItems = allItems.slice(start, start + limit);
    return HttpResponse.json({
      success: true,
      data: pageItems,
      pagination: { page, limit, total: allItems.length },
    });
  }),
  http.patch('/api/v1/items/:id/status', async ({ params, request }) => {
    const body = (await request.json()) as { status: string };
    const item = allItems.find((i) => i.id === params.id);
    if (!item) {
      return HttpResponse.json(
        { success: false, error: { code: 'ITEM_NOT_FOUND', message: 'not found' } },
        { status: 404 },
      );
    }
    item.status = body.status as Item['status'];
    return HttpResponse.json({ success: true, data: item });
  }),
  http.delete('/api/v1/items/:id', ({ params }) => {
    const index = allItems.findIndex((i) => i.id === params.id);
    if (index === -1) {
      return HttpResponse.json(
        { success: false, error: { code: 'ITEM_NOT_FOUND', message: 'not found' } },
        { status: 404 },
      );
    }
    allItems.splice(index, 1);
    // Note: apiClient.ts always calls response.json(), so a true empty-body 204
    // would fail JSON parsing. Return a JSON success envelope to match apiClient's
    // actual runtime behavior (see apiClient.ts apiRequest()).
    return HttpResponse.json({ success: true, data: null });
  }),
);

beforeAll(() => server.listen({ onUnhandledRequest: 'error' }));
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

function baseFilterState(): ItemFilterState {
  return {
    mediaTypes: [],
    tagIds: [],
    categoryIds: [],
    isFavorite: false,
    statuses: [],
    title: '',
  };
}

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });
  function Wrapper({ children }: { children: React.ReactNode }) {
    return React.createElement(
      QueryClientProvider,
      { client: queryClient },
      React.createElement(MemoryRouter, null, children),
    );
  }
  return { Wrapper, queryClient };
}

describe('items hooks integration (msw)', () => {
  it('lists page 1, fetches next page, updates status, then deletes an item', async () => {
    const { Wrapper, queryClient } = createWrapper();
    const filterState = baseFilterState();

    // limit=1 via monkey-patched query key isn't configurable here, so we use the
    // default 20 limit which returns all 3 items in page 1 (no next page).
    const { result: listResult } = renderHook(() => useItemsInfiniteQuery(filterState), {
      wrapper: Wrapper,
    });

    await waitFor(() => expect(listResult.current.isSuccess).toBe(true));
    expect(listResult.current.data?.pages[0].data).toHaveLength(3);
    expect(listResult.current.hasNextPage).toBe(false);

    // Update status of item 1
    const { result: updateResult } = renderHook(() => useUpdateItemStatus(), {
      wrapper: Wrapper,
    });

    act(() => {
      updateResult.current.mutate({ id: '1', body: { status: 'completed' } });
    });

    await waitFor(() => expect(updateResult.current.isSuccess).toBe(true));

    await waitFor(() => {
      const cache = queryClient.getQueryData<{
        pages: Array<{ data: Item[] }>;
      }>(itemsListQueryKey(filterState));
      const item1 = cache?.pages[0].data.find((i) => i.id === '1');
      expect(item1?.status).toBe('completed');
    });

    // Delete item 2
    const { result: deleteResult } = renderHook(() => useDeleteItem(), { wrapper: Wrapper });

    act(() => {
      deleteResult.current.mutate('2');
    });

    await waitFor(() => expect(deleteResult.current.isSuccess).toBe(true));
    expect(navigateMock).toHaveBeenCalledWith('/');
    expect(allItems.find((i) => i.id === '2')).toBeUndefined();
  });
});
