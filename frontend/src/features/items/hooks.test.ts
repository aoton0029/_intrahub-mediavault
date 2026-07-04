import { describe, it, expect, vi, beforeEach } from 'vitest';
import React from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { renderHook, waitFor, act } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { toast } from 'sonner';
import {
  useItemsInfiniteQuery,
  useItemDetailQuery,
  useUpdateItemStatus,
  useDeleteItem,
} from './hooks';
import { listItems, getItemDetail, updateItemStatus, deleteItem } from './api';
import { ApiClientError } from '../../lib/apiClient';
import type { Item, ItemFilterState } from './types';

vi.mock('./api', () => ({
  listItems: vi.fn(),
  getItemDetail: vi.fn(),
  updateItemStatus: vi.fn(),
  deleteItem: vi.fn(),
}));

const navigateMock = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual<typeof import('react-router-dom')>('react-router-dom');
  return {
    ...actual,
    useNavigate: () => navigateMock,
  };
});

vi.mock('sonner', () => ({
  toast: {
    error: vi.fn(),
    success: vi.fn(),
  },
}));

const mockedListItems = vi.mocked(listItems);
const mockedGetItemDetail = vi.mocked(getItemDetail);
const mockedUpdateItemStatus = vi.mocked(updateItemStatus);
const mockedDeleteItem = vi.mocked(deleteItem);

function baseFilterState(overrides: Partial<ItemFilterState> = {}): ItemFilterState {
  return {
    mediaTypes: [],
    tagIds: [],
    categoryIds: [],
    isFavorite: false,
    statuses: [],
    title: '',
    ...overrides,
  };
}

function makeItem(overrides: Partial<Item> = {}): Item {
  return {
    id: 'item-1',
    media_type: 'anime',
    title: 'Sample',
    status: 'not_started',
    is_favorite: false,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    ...overrides,
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

describe('useItemsInfiniteQuery', () => {
  beforeEach(() => {
    mockedListItems.mockReset();
  });

  it('EDGE-102: hasNextPage is false once cumulative fetched count reaches pagination.total', async () => {
    mockedListItems.mockResolvedValueOnce({
      data: [makeItem({ id: '1' }), makeItem({ id: '2' })],
      pagination: { page: 1, limit: 2, total: 2 },
    } as never);

    const { Wrapper } = createWrapper();
    const { result } = renderHook(() => useItemsInfiniteQuery(baseFilterState()), {
      wrapper: Wrapper,
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.hasNextPage).toBe(false);
  });

  it('hasNextPage is true when cumulative fetched count is below pagination.total', async () => {
    mockedListItems.mockResolvedValueOnce({
      data: [makeItem({ id: '1' })],
      pagination: { page: 1, limit: 1, total: 5 },
    } as never);

    const { Wrapper } = createWrapper();
    const { result } = renderHook(() => useItemsInfiniteQuery(baseFilterState()), {
      wrapper: Wrapper,
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(result.current.hasNextPage).toBe(true);
  });

  it('refetches with a new queryKey when filterState changes', async () => {
    mockedListItems.mockResolvedValue({
      data: [makeItem({ id: '1' })],
      pagination: { page: 1, limit: 20, total: 1 },
    } as never);

    const { Wrapper } = createWrapper();
    const { result, rerender } = renderHook(
      ({ filterState }) => useItemsInfiniteQuery(filterState),
      { wrapper: Wrapper, initialProps: { filterState: baseFilterState() } },
    );

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockedListItems).toHaveBeenCalledTimes(1);

    rerender({ filterState: baseFilterState({ title: 'changed' }) });

    await waitFor(() => expect(mockedListItems).toHaveBeenCalledTimes(2));
    expect(mockedListItems.mock.calls[1][0]).toEqual(
      expect.objectContaining({ title: 'changed' }),
    );
  });
});

describe('useItemDetailQuery', () => {
  beforeEach(() => {
    mockedGetItemDetail.mockReset();
  });

  it('fetches item detail by id', async () => {
    mockedGetItemDetail.mockResolvedValueOnce({
      data: { ...makeItem(), tags: [], categories: [], calibre_links: [] },
    } as never);

    const { Wrapper } = createWrapper();
    const { result } = renderHook(() => useItemDetailQuery('item-1'), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(mockedGetItemDetail).toHaveBeenCalledWith('item-1');
  });
});

describe('useUpdateItemStatus', () => {
  beforeEach(() => {
    mockedUpdateItemStatus.mockReset();
    vi.mocked(toast.error).mockReset();
  });

  it('optimistically updates list cache before the server responds', async () => {
    const { Wrapper, queryClient } = createWrapper();

    queryClient.setQueryData(['items', baseFilterState()], {
      pages: [
        {
          data: [makeItem({ id: 'item-1', status: 'not_started' })],
          pagination: { page: 1, limit: 20, total: 1 },
        },
      ],
      pageParams: [1],
    });

    let resolveUpdate!: (value: unknown) => void;
    mockedUpdateItemStatus.mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          resolveUpdate = resolve;
        }) as never,
    );

    const { result } = renderHook(() => useUpdateItemStatus(), { wrapper: Wrapper });

    act(() => {
      result.current.mutate({ id: 'item-1', body: { status: 'completed' } });
    });

    await waitFor(() => {
      const cache = queryClient.getQueryData<{
        pages: Array<{ data: Item[] }>;
      }>(['items', baseFilterState()]);
      expect(cache?.pages[0].data[0].status).toBe('completed');
    });

    resolveUpdate({ data: makeItem({ id: 'item-1', status: 'completed' }) });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
  });

  it('EDGE-003: rolls back cache and shows a toast on error', async () => {
    const { Wrapper, queryClient } = createWrapper();

    queryClient.setQueryData(['items', baseFilterState()], {
      pages: [
        {
          data: [makeItem({ id: 'item-1', status: 'not_started' })],
          pagination: { page: 1, limit: 20, total: 1 },
        },
      ],
      pageParams: [1],
    });

    mockedUpdateItemStatus.mockRejectedValueOnce(
      new ApiClientError('INTERNAL_ERROR', 'failed'),
    );

    const { result } = renderHook(() => useUpdateItemStatus(), { wrapper: Wrapper });

    act(() => {
      result.current.mutate({ id: 'item-1', body: { status: 'completed' } });
    });

    await waitFor(() => expect(result.current.isError).toBe(true));

    const cache = queryClient.getQueryData<{
      pages: Array<{ data: Item[] }>;
    }>(['items', baseFilterState()]);
    expect(cache?.pages[0].data[0].status).toBe('not_started');
    expect(toast.error).toHaveBeenCalled();
  });
});

describe('useDeleteItem', () => {
  beforeEach(() => {
    mockedDeleteItem.mockReset();
    navigateMock.mockReset();
    vi.mocked(toast.error).mockReset();
  });

  it('navigates to "/" on success', async () => {
    mockedDeleteItem.mockResolvedValueOnce(undefined);

    const { Wrapper } = createWrapper();
    const { result } = renderHook(() => useDeleteItem(), { wrapper: Wrapper });

    act(() => {
      result.current.mutate('item-1');
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(navigateMock).toHaveBeenCalledWith('/');
  });

  it('EDGE-002: shows a toast and invalidates items on 404', async () => {
    mockedDeleteItem.mockRejectedValueOnce(
      new ApiClientError('ITEM_NOT_FOUND', 'not found'),
    );

    const { Wrapper, queryClient } = createWrapper();
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');

    const { result } = renderHook(() => useDeleteItem(), { wrapper: Wrapper });

    act(() => {
      result.current.mutate('missing-item');
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(toast.error).toHaveBeenCalled();
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['items'] });
    expect(navigateMock).not.toHaveBeenCalled();
  });
});
