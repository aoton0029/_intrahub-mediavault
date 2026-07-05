import { useSearchParams } from 'react-router-dom';
import { useMemo } from 'react';
import type { ItemFilters } from '../api';
import type { MediaType } from '../types';

export function useItemFilters() {
  const [searchParams, setSearchParams] = useSearchParams();

  const filters: ItemFilters = useMemo(
    () => ({
      media_type: (searchParams.get('media_type') as MediaType | null) ?? undefined,
      is_favorite: searchParams.get('is_favorite') === 'true' ? true : undefined,
      tag_id: searchParams.get('tag_id') ?? undefined,
      category_id: searchParams.get('category_id') ?? undefined,
      title: searchParams.get('title') ?? undefined,
    }),
    [searchParams],
  );

  function update(mutate: (params: URLSearchParams) => void) {
    setSearchParams(
      (prev) => {
        const next = new URLSearchParams(prev);
        mutate(next);
        return next;
      },
      { replace: true },
    );
  }

  function toggleFavorite() {
    update((params) => {
      if (params.get('is_favorite') === 'true') {
        params.delete('is_favorite');
      } else {
        params.set('is_favorite', 'true');
      }
    });
  }

  function setMediaType(value: MediaType | '') {
    update((params) => {
      if (value) {
        params.set('media_type', value);
      } else {
        params.delete('media_type');
      }
    });
  }

  function toggleTag(tagId: string) {
    update((params) => {
      if (params.get('tag_id') === tagId) {
        params.delete('tag_id');
      } else {
        params.set('tag_id', tagId);
      }
    });
  }

  function toggleCategory(categoryId: string) {
    update((params) => {
      if (params.get('category_id') === categoryId) {
        params.delete('category_id');
      } else {
        params.set('category_id', categoryId);
      }
    });
  }

  function setTitle(value: string) {
    update((params) => {
      if (value) {
        params.set('title', value);
      } else {
        params.delete('title');
      }
    });
  }

  return { filters, toggleFavorite, setMediaType, toggleTag, toggleCategory, setTitle };
}
