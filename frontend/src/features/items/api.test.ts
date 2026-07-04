import { describe, it, expect, vi, beforeEach } from 'vitest';
import { apiRequest } from '../../lib/apiClient';
import { listItems, getItemDetail, updateItemStatus, deleteItem } from './api';
import type { Item, ItemDetail, PaginatedOk, ApiOk } from './types';

vi.mock('../../lib/apiClient', () => ({
  apiRequest: vi.fn(),
}));

const mockedApiRequest = vi.mocked(apiRequest);

describe('items api', () => {
  beforeEach(() => {
    mockedApiRequest.mockReset();
  });

  describe('listItems', () => {
    it('builds query params from all provided fields', async () => {
      const expected = {
        data: [] as Item[],
        pagination: { page: 1, limit: 20, total: 0 },
      } as unknown as PaginatedOk<Item[]>;
      mockedApiRequest.mockResolvedValueOnce(expected);

      await listItems({
        media_type: 'anime',
        tag_id: 'tag-1',
        category_id: 'cat-1',
        is_favorite: true,
        status: 'in_progress',
        title: 'foo',
        page: 2,
        limit: 50,
      });

      expect(mockedApiRequest).toHaveBeenCalledTimes(1);
      const [path] = mockedApiRequest.mock.calls[0];
      const url = new URL(`http://localhost${path}`);
      expect(url.pathname).toBe('/items');
      expect(url.searchParams.get('media_type')).toBe('anime');
      expect(url.searchParams.get('tag_id')).toBe('tag-1');
      expect(url.searchParams.get('category_id')).toBe('cat-1');
      expect(url.searchParams.get('is_favorite')).toBe('true');
      expect(url.searchParams.get('status')).toBe('in_progress');
      expect(url.searchParams.get('title')).toBe('foo');
      expect(url.searchParams.get('page')).toBe('2');
      expect(url.searchParams.get('limit')).toBe('50');
    });

    it('converts is_favorite=false to string "false"', async () => {
      mockedApiRequest.mockResolvedValueOnce({
        data: [],
        pagination: { page: 1, limit: 20, total: 0 },
      } as unknown as PaginatedOk<Item[]>);

      await listItems({ is_favorite: false });

      const [path] = mockedApiRequest.mock.calls[0];
      const url = new URL(`http://localhost${path}`);
      expect(url.searchParams.get('is_favorite')).toBe('false');
    });

    it('omits unset fields from the query string', async () => {
      mockedApiRequest.mockResolvedValueOnce({
        data: [],
        pagination: { page: 1, limit: 20, total: 0 },
      } as unknown as PaginatedOk<Item[]>);

      await listItems({});

      const [path] = mockedApiRequest.mock.calls[0];
      const url = new URL(`http://localhost${path}`);
      expect(url.pathname).toBe('/items');
      expect([...url.searchParams.keys()]).toHaveLength(0);
    });

    it('omits empty string title from the query string', async () => {
      mockedApiRequest.mockResolvedValueOnce({
        data: [],
        pagination: { page: 1, limit: 20, total: 0 },
      } as unknown as PaginatedOk<Item[]>);

      await listItems({ title: '' });

      const [path] = mockedApiRequest.mock.calls[0];
      const url = new URL(`http://localhost${path}`);
      expect(url.searchParams.has('title')).toBe(false);
    });

    it('passes GET method and no body', async () => {
      mockedApiRequest.mockResolvedValueOnce({
        data: [],
        pagination: { page: 1, limit: 20, total: 0 },
      } as unknown as PaginatedOk<Item[]>);

      await listItems({});

      const [, init] = mockedApiRequest.mock.calls[0];
      expect(init?.method ?? 'GET').toBe('GET');
    });
  });

  describe('getItemDetail', () => {
    it('calls GET /items/{id}', async () => {
      const expected = { data: {} as ItemDetail } as unknown as ApiOk<ItemDetail>;
      mockedApiRequest.mockResolvedValueOnce(expected);

      await getItemDetail('item-123');

      expect(mockedApiRequest).toHaveBeenCalledWith('/items/item-123');
    });
  });

  describe('updateItemStatus', () => {
    it('calls PATCH /items/{id}/status with body', async () => {
      const expected = { data: {} as Item } as unknown as ApiOk<Item>;
      mockedApiRequest.mockResolvedValueOnce(expected);

      await updateItemStatus('item-123', {
        status: 'completed',
        consumed_date: '2026-07-01',
      });

      expect(mockedApiRequest).toHaveBeenCalledWith(
        '/items/item-123/status',
        expect.objectContaining({
          method: 'PATCH',
          headers: expect.objectContaining({ 'Content-Type': 'application/json' }),
          body: JSON.stringify({ status: 'completed', consumed_date: '2026-07-01' }),
        }),
      );
    });
  });

  describe('deleteItem', () => {
    it('calls DELETE /items/{id}', async () => {
      mockedApiRequest.mockResolvedValueOnce(undefined as unknown as void);

      await deleteItem('item-123');

      expect(mockedApiRequest).toHaveBeenCalledWith(
        '/items/item-123',
        expect.objectContaining({ method: 'DELETE' }),
      );
    });
  });
});
