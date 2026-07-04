import {
  useInfiniteQuery,
  useMutation,
  useQuery,
  useQueryClient,
  type InfiniteData,
  type QueryKey,
} from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { toast } from 'sonner';
import { deleteItem, getItemDetail, listItems, updateItemStatus } from './api';
import { ApiClientError } from '../../lib/apiClient';
import type { ApiOk, Item, ItemDetail, ItemFilterState, UpdateStatusRequest } from './types';

/**
 * TanStack Query v5フック群
 *
 * TASK-0005で実装したitems API関数をラップする。
 * 関連: docs/tasks/collection-browsing/TASK-0006.md
 */

const ITEMS_LIST_LIMIT = 20;

/**
 * listItemsの実際の解決型（PaginatedOk<Item[]>と宣言されているが、
 * ランタイムでは`data`はItem[]のフラットな配列）。
 */
type ItemsListPage = Omit<Awaited<ReturnType<typeof listItems>>, 'data'> & { data: Item[] };

/**
 * 一覧クエリキー生成。
 */
export function itemsListQueryKey(filterState: ItemFilterState): QueryKey {
  return ['items', filterState];
}

/**
 * 詳細クエリキー生成。
 */
export function itemDetailQueryKey(id: string): QueryKey {
  return ['item', id];
}

/**
 * 1. useItemsInfiniteQuery — 一覧の無限スクロール取得。
 *
 * queryKey: ['items', filterState]（フィルタ変化で自動再フェッチ、REQ-101/102/301）。
 * getNextPageParam: 累計取得件数がpagination.totalに到達していればundefined（EDGE-102）。
 */
export function useItemsInfiniteQuery(filterState: ItemFilterState) {
  return useInfiniteQuery<
    ItemsListPage,
    ApiClientError,
    InfiniteData<ItemsListPage, number>,
    QueryKey,
    number
  >({
    queryKey: itemsListQueryKey(filterState),
    queryFn: ({ pageParam }) =>
      listItems({
        ...filterStateToQuery(filterState),
        page: pageParam,
        limit: ITEMS_LIST_LIMIT,
      }) as unknown as Promise<ItemsListPage>,
    initialPageParam: 1,
    getNextPageParam: (lastPage, allPages) => {
      const fetchedCount = allPages.reduce((sum, page) => sum + page.data.length, 0);
      if (fetchedCount >= lastPage.pagination.total) {
        return undefined;
      }
      return lastPage.pagination.page + 1;
    },
  });
}

/**
 * ItemFilterState（配列を含む複合フィルタ）をListItemsQuery（単一値）へ変換する。
 * 現状のListItemsQueryは各フィールド単一値のみ対応のため、配列先頭要素を採用する。
 * 🟡 複合AND条件のクエリ変換仕様は未確定のため妥当な推測（note.md参照）。
 */
function filterStateToQuery(filterState: ItemFilterState) {
  return {
    media_type: filterState.mediaTypes[0],
    tag_id: filterState.tagIds[0],
    category_id: filterState.categoryIds[0],
    is_favorite: filterState.isFavorite ? true : undefined,
    status: filterState.statuses[0],
    title: filterState.title,
  };
}

/**
 * 2. useItemDetailQuery — 詳細取得。
 *
 * 404 (ITEM_NOT_FOUND) 時はisErrorがtrueとなり、呼び出し側でエラー表示に利用できる。
 */
export function useItemDetailQuery(id: string) {
  return useQuery<ApiOk<ItemDetail>, ApiClientError>({
    queryKey: itemDetailQueryKey(id),
    queryFn: () => getItemDetail(id),
    enabled: id !== '',
  });
}

interface UpdateItemStatusVariables {
  id: string;
  body: UpdateStatusRequest;
}

interface UpdateItemStatusContext {
  previousLists: Array<[QueryKey, InfiniteData<ItemsListPage, number> | undefined]>;
  previousDetail: ApiOk<ItemDetail> | undefined;
}

/**
 * 3. useUpdateItemStatus — status楽観的更新。
 *
 * onMutate: 一覧キャッシュ・詳細キャッシュを楽観的に更新し、直前状態をコンテキストに保持。
 * onError: コンテキストからロールバックし、sonnerでエラー通知（EDGE-003）。
 * onSuccess: サーバーレスポンスでキャッシュを確定する。
 */
export function useUpdateItemStatus() {
  const queryClient = useQueryClient();

  return useMutation<ApiOk<Item>, ApiClientError, UpdateItemStatusVariables, UpdateItemStatusContext>({
    mutationFn: ({ id, body }) => updateItemStatus(id, body),
    onMutate: async ({ id, body }) => {
      await queryClient.cancelQueries({ queryKey: ['items'] });
      await queryClient.cancelQueries({ queryKey: itemDetailQueryKey(id) });

      const listQueries = queryClient.getQueriesData<InfiniteData<ItemsListPage, number>>({
        queryKey: ['items'],
      });
      const previousLists = listQueries.map(
        ([key, data]) => [key, data] as [QueryKey, InfiniteData<ItemsListPage, number> | undefined],
      );

      const previousDetail = queryClient.getQueryData<ApiOk<ItemDetail>>(itemDetailQueryKey(id));

      for (const [key, data] of listQueries) {
        if (!data) continue;
        queryClient.setQueryData<InfiniteData<ItemsListPage, number>>(key, {
          ...data,
          pages: data.pages.map((page) => ({
            ...page,
            data: page.data.map((item) =>
              item.id === id ? { ...item, status: body.status } : item,
            ),
          })),
        });
      }

      if (previousDetail) {
        queryClient.setQueryData<ApiOk<ItemDetail>>(itemDetailQueryKey(id), {
          ...previousDetail,
          data: {
            ...previousDetail.data,
            status: body.status,
            consumed_date: body.consumed_date ?? previousDetail.data.consumed_date,
          },
        });
      }

      return { previousLists, previousDetail };
    },
    onError: (_err, { id }, context) => {
      if (context) {
        for (const [key, data] of context.previousLists) {
          queryClient.setQueryData(key, data);
        }
        if (context.previousDetail) {
          queryClient.setQueryData(itemDetailQueryKey(id), context.previousDetail);
        }
      }
      toast.error('ステータスの更新に失敗しました');
    },
    onSuccess: (response, { id }) => {
      queryClient.setQueryData<ApiOk<ItemDetail>>(itemDetailQueryKey(id), (old) =>
        old ? { ...old, data: { ...old.data, ...response.data } } : old,
      );
    },
    onSettled: (_data, _err, { id }) => {
      void queryClient.invalidateQueries({ queryKey: ['items'] });
      void queryClient.invalidateQueries({ queryKey: itemDetailQueryKey(id) });
    },
  });
}

/**
 * 4. useDeleteItem — アイテム削除。
 *
 * 成功時: navigate("/") で一覧画面へ遷移。
 * 404 (ITEM_NOT_FOUND) 時: sonnerでエラー通知し、一覧クエリをinvalidateQueries（EDGE-002）。
 */
export function useDeleteItem() {
  const queryClient = useQueryClient();
  const navigate = useNavigate();

  return useMutation<void, ApiClientError, string>({
    mutationFn: (id: string) => deleteItem(id),
    onSuccess: () => {
      navigate('/');
    },
    onError: (error) => {
      if (error instanceof ApiClientError && error.code === 'ITEM_NOT_FOUND') {
        toast.error('アイテムが見つかりませんでした');
        void queryClient.invalidateQueries({ queryKey: ['items'] });
        return;
      }
      toast.error('アイテムの削除に失敗しました');
    },
  });
}
