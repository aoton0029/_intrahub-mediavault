/**
 * TASK-0018: groups/episodes APIフック実装
 * 🔵 docs/tasks/frontend-collection-ui/TASK-0018.md REQ-015/016 より
 */
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from './client';
import type { ItemGroup, ItemEpisode, CreateGroupRequest, CreateEpisodeRequest } from '@/types';

// ========================================
// fetch関数群
// ========================================

/**
 * 【機能概要】: アイテムのグループ一覧を取得する
 * 【実装方針】: GET /items/:id/groups を apiClient で呼び出す
 * 【テスト対応】: TC-GRP-N-01
 * 🔵 REQ-015 GET /items/:id/groups より
 */
export async function fetchItemGroups(itemId: string): Promise<{ data: ItemGroup[] }> {
  return apiClient<ItemGroup[]>(`/items/${itemId}/groups`);
}

/**
 * 【機能概要】: グループを新規作成する
 * 【実装方針】: POST /items/:id/groups を apiClient で呼び出す
 * 【テスト対応】: TC-GRP-N-02
 * 🔵 REQ-015 POST /items/:id/groups より
 */
export async function createGroup(
  itemId: string,
  body: CreateGroupRequest
): Promise<{ data: ItemGroup }> {
  return apiClient<ItemGroup>(`/items/${itemId}/groups`, {
    method: 'POST',
    body,
  });
}

/**
 * 【機能概要】: グループの話数一覧を取得する
 * 【実装方針】: GET /groups/:group_id/episodes を apiClient で呼び出す
 * 【テスト対応】: TC-GRP-N-03
 * 🔵 REQ-016 GET /groups/:group_id/episodes より
 */
export async function fetchGroupEpisodes(groupId: string): Promise<{ data: ItemEpisode[] }> {
  return apiClient<ItemEpisode[]>(`/groups/${groupId}/episodes`);
}

/**
 * 【機能概要】: 話数を新規作成する
 * 【実装方針】: POST /groups/:group_id/episodes を apiClient で呼び出す
 * 【テスト対応】: TC-GRP-N-04
 * 🔵 REQ-016 POST /groups/:group_id/episodes より
 */
export async function createEpisode(
  groupId: string,
  body: CreateEpisodeRequest
): Promise<{ data: ItemEpisode }> {
  return apiClient<ItemEpisode>(`/groups/${groupId}/episodes`, {
    method: 'POST',
    body,
  });
}

// ========================================
// Queryフック群
// ========================================

/**
 * 【機能概要】: アイテムのグループ一覧を取得するQueryフック
 * 【実装方針】: queryKey=['items','groups',itemId] で itemId 単位にキャッシュ分離
 *              enabled: !!itemId で空IDの場合はfetchしない
 * 【テスト対応】: TC-GRP-N-01, TC-GRP-N-05, TC-GRP-B-01, TC-GRP-E-01
 * 🔵 REQ-015・architecture.md「リソース別フック」方針より
 */
export function useItemGroupsQuery(itemId: string) {
  return useQuery({
    queryKey: ['items', 'groups', itemId],
    queryFn: () => fetchItemGroups(itemId),
    enabled: !!itemId, // 【空ID制御】: itemIdが空の場合はfetchしない 🔵
  });
}

/**
 * 【機能概要】: グループを作成するMutationフック
 * 【実装方針】: 成功時に ['items','groups',itemId] を invalidateQueries してキャッシュを更新
 * 【テスト対応】: TC-GRP-N-02, TC-GRP-E-02
 * 🔵 REQ-015・items.ts Mutationパターンより
 */
export function useCreateGroupMutation(itemId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (body: CreateGroupRequest) => createGroup(itemId, body),
    onSuccess: () => {
      // 【キャッシュ無効化】: グループ作成成功後に一覧を再取得させる 🔵
      queryClient.invalidateQueries({ queryKey: ['items', 'groups', itemId] });
    },
  });
}

/**
 * 【機能概要】: グループの話数一覧を取得するQueryフック
 * 【実装方針】: queryKey=['groups','episodes',groupId] で groupId 単位にキャッシュ分離
 *              enabled: !!groupId で空IDの場合はfetchしない
 * 【テスト対応】: TC-GRP-N-03, TC-GRP-B-02, TC-GRP-E-03
 * 🔵 REQ-016より
 */
export function useGroupEpisodesQuery(groupId: string) {
  return useQuery({
    queryKey: ['groups', 'episodes', groupId],
    queryFn: () => fetchGroupEpisodes(groupId),
    enabled: !!groupId, // 【空ID制御】: groupIdが空の場合はfetchしない 🔵
  });
}

/**
 * 【機能概要】: 話数を作成するMutationフック
 * 【実装方針】: 成功時に ['groups','episodes',groupId] を invalidateQueries してキャッシュを更新
 * 【テスト対応】: TC-GRP-N-04
 * 🔵 REQ-016・items.ts Mutationパターンより
 */
export function useCreateEpisodeMutation(groupId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (body: CreateEpisodeRequest) => createEpisode(groupId, body),
    onSuccess: () => {
      // 【キャッシュ無効化】: 話数作成成功後に一覧を再取得させる 🔵
      queryClient.invalidateQueries({ queryKey: ['groups', 'episodes', groupId] });
    },
  });
}
