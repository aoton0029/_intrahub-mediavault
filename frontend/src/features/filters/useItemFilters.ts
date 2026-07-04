/**
 * フィルタ状態管理フック（useItemFilters）
 *
 * TASK-0007: media_type・tag_id・category_id・is_favorite・status・title の複合フィルタ状態を
 * useSearchParams（react-router-dom v7）と同期させるフック。
 * フィルタ変更は即時にURLへ反映され（NFR-201）、リロード時はURLから状態を復元する（REQ-301）。
 */
import { useCallback, useMemo } from 'react';
import { useSearchParams } from 'react-router-dom';
import type { ItemFilterState, MediaType, ItemStatus } from '../items/types';

const PARAM_MEDIA_TYPE = 'media_type';
const PARAM_TAG_ID = 'tag_id';
const PARAM_CATEGORY_ID = 'category_id';
const PARAM_IS_FAVORITE = 'is_favorite';
const PARAM_STATUS = 'status';
const PARAM_TITLE = 'title';

export interface UseItemFiltersResult {
  filters: ItemFilterState;
  setMediaTypes: (mediaTypes: MediaType[]) => void;
  toggleTag: (tagId: string) => void;
  toggleCategory: (categoryId: string) => void;
  setIsFavorite: (isFavorite: boolean) => void;
  toggleStatus: (status: ItemStatus) => void;
  setTitle: (title: string) => void;
}

function toggleValue<T>(values: T[], value: T): T[] {
  return values.includes(value) ? values.filter((v) => v !== value) : [...values, value];
}

/**
 * URLSearchParamsから ItemFilterState を導出する。
 */
function parseFilterState(searchParams: URLSearchParams): ItemFilterState {
  return {
    mediaTypes: searchParams.getAll(PARAM_MEDIA_TYPE) as MediaType[],
    tagIds: searchParams.getAll(PARAM_TAG_ID),
    categoryIds: searchParams.getAll(PARAM_CATEGORY_ID),
    isFavorite: searchParams.get(PARAM_IS_FAVORITE) === 'true',
    statuses: searchParams.getAll(PARAM_STATUS) as ItemStatus[],
    title: searchParams.get(PARAM_TITLE) ?? '',
  };
}

/**
 * ItemFilterStateをURLSearchParamsへ書き戻す。
 * フィルタ以外の既存クエリパラメータ（将来的な拡張用）は保持する。
 */
function writeFilterState(prev: URLSearchParams, next: ItemFilterState): URLSearchParams {
  const params = new URLSearchParams(prev);

  params.delete(PARAM_MEDIA_TYPE);
  next.mediaTypes.forEach((mt) => params.append(PARAM_MEDIA_TYPE, mt));

  params.delete(PARAM_TAG_ID);
  next.tagIds.forEach((id) => params.append(PARAM_TAG_ID, id));

  params.delete(PARAM_CATEGORY_ID);
  next.categoryIds.forEach((id) => params.append(PARAM_CATEGORY_ID, id));

  if (next.isFavorite) {
    params.set(PARAM_IS_FAVORITE, 'true');
  } else {
    params.delete(PARAM_IS_FAVORITE);
  }

  params.delete(PARAM_STATUS);
  next.statuses.forEach((s) => params.append(PARAM_STATUS, s));

  if (next.title) {
    params.set(PARAM_TITLE, next.title);
  } else {
    params.delete(PARAM_TITLE);
  }

  return params;
}

export function useItemFilters(): UseItemFiltersResult {
  const [searchParams, setSearchParams] = useSearchParams();

  const filters = useMemo(() => parseFilterState(searchParams), [searchParams]);

  const update = useCallback(
    (updater: (current: ItemFilterState) => ItemFilterState) => {
      setSearchParams((prev) => {
        const current = parseFilterState(prev);
        const next = updater(current);
        return writeFilterState(prev, next);
      });
    },
    [setSearchParams],
  );

  const setMediaTypes = useCallback(
    (mediaTypes: MediaType[]) => update((current) => ({ ...current, mediaTypes })),
    [update],
  );

  const toggleTag = useCallback(
    (tagId: string) =>
      update((current) => ({ ...current, tagIds: toggleValue(current.tagIds, tagId) })),
    [update],
  );

  const toggleCategory = useCallback(
    (categoryId: string) =>
      update((current) => ({
        ...current,
        categoryIds: toggleValue(current.categoryIds, categoryId),
      })),
    [update],
  );

  const setIsFavorite = useCallback(
    (isFavorite: boolean) => update((current) => ({ ...current, isFavorite })),
    [update],
  );

  const toggleStatus = useCallback(
    (status: ItemStatus) =>
      update((current) => ({ ...current, statuses: toggleValue(current.statuses, status) })),
    [update],
  );

  const setTitle = useCallback(
    (title: string) => update((current) => ({ ...current, title })),
    [update],
  );

  return {
    filters,
    setMediaTypes,
    toggleTag,
    toggleCategory,
    setIsFavorite,
    toggleStatus,
    setTitle,
  };
}
