import { describe, it, expect } from 'vitest';
import React from 'react';
import { renderHook, act } from '@testing-library/react';
import { MemoryRouter, useSearchParams } from 'react-router-dom';
import { useItemFilters } from './useItemFilters';

function wrapperFor(initialEntry: string) {
  return function Wrapper({ children }: { children: React.ReactNode }) {
    return React.createElement(MemoryRouter, { initialEntries: [initialEntry] }, children);
  };
}

function renderWithLocation(initialEntry = '/') {
  return renderHook(
    () => {
      const filters = useItemFilters();
      const [searchParams] = useSearchParams();
      return { ...filters, searchParams };
    },
    { wrapper: wrapperFor(initialEntry) },
  );
}

describe('useItemFilters', () => {
  it('フィルタ変更時に即座にURLへ反映される（NFR-201）', () => {
    const { result } = renderWithLocation('/');

    act(() => {
      result.current.setMediaTypes(['anime']);
    });

    expect(result.current.searchParams.getAll('media_type')).toEqual(['anime']);
    expect(result.current.filters.mediaTypes).toEqual(['anime']);

    act(() => {
      result.current.toggleTag('tag-1');
    });

    expect(result.current.searchParams.getAll('tag_id')).toEqual(['tag-1']);
    expect(result.current.filters.tagIds).toEqual(['tag-1']);
  });

  it('URLクエリから初期状態を復元する（REQ-301: リロード相当）', () => {
    const { result } = renderWithLocation('/?media_type=anime&is_favorite=true');

    expect(result.current.filters.mediaTypes).toEqual(['anime']);
    expect(result.current.filters.isFavorite).toBe(true);
    expect(result.current.filters.tagIds).toEqual([]);
    expect(result.current.filters.categoryIds).toEqual([]);
    expect(result.current.filters.statuses).toEqual([]);
    expect(result.current.filters.title).toBe('');
  });

  it('複数種類のフィルタを同時に設定した場合、それぞれ独立してURL・状態に反映される（REQ-101）', () => {
    const { result } = renderWithLocation('/');

    act(() => {
      result.current.setMediaTypes(['anime', 'manga']);
    });
    act(() => {
      result.current.toggleTag('tag-1');
    });
    act(() => {
      result.current.toggleCategory('cat-1');
    });
    act(() => {
      result.current.setIsFavorite(true);
    });
    act(() => {
      result.current.toggleStatus('in_progress');
    });

    expect(result.current.filters).toEqual({
      mediaTypes: ['anime', 'manga'],
      tagIds: ['tag-1'],
      categoryIds: ['cat-1'],
      isFavorite: true,
      statuses: ['in_progress'],
      title: '',
    });

    expect(result.current.searchParams.getAll('media_type')).toEqual(['anime', 'manga']);
    expect(result.current.searchParams.getAll('tag_id')).toEqual(['tag-1']);
    expect(result.current.searchParams.getAll('category_id')).toEqual(['cat-1']);
    expect(result.current.searchParams.get('is_favorite')).toBe('true');
    expect(result.current.searchParams.getAll('status')).toEqual(['in_progress']);
  });

  it('タイトル検索文字列の変更が即時にURLへ反映される（REQ-102/NFR-201）', () => {
    const { result } = renderWithLocation('/');

    act(() => {
      result.current.setTitle('星屑');
    });

    expect(result.current.searchParams.get('title')).toBe('星屑');
    expect(result.current.filters.title).toBe('星屑');

    act(() => {
      result.current.setTitle('');
    });

    expect(result.current.searchParams.get('title')).toBeNull();
    expect(result.current.filters.title).toBe('');
  });
});
