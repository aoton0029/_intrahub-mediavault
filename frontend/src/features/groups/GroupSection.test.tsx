/**
 * TASK-0018: GroupSection コンポーネント テストファイル (Redフェーズ)
 * 🔵 docs/tasks/frontend-collection-ui/TASK-0018.md REQ-101/102/EDGE-004 より
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { createElement } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import type { Item, ItemGroup } from '@/types';
import GroupSection from './GroupSection';
import VolumeGroupList from './VolumeGroupList';
import SeasonGroupList from './SeasonGroupList';

// ===== APIフックのモック =====
vi.mock('@/api/groups', () => ({
  useItemGroupsQuery: vi.fn(),
  useCreateGroupMutation: vi.fn(),
  useGroupEpisodesQuery: vi.fn(),
  useCreateEpisodeMutation: vi.fn(),
}));

import { useItemGroupsQuery, useCreateEpisodeMutation } from '@/api/groups';

// ===== テストデータ =====

const mockSeasonGroup: ItemGroup = {
  id: 'group-001',
  itemId: 'item-001',
  groupType: 'season',
  groupName: 'Season 1',
  displayOrder: 1,
  createdAt: '2026-07-01T00:00:00Z',
  updatedAt: '2026-07-01T00:00:00Z',
};

const mockVolumeGroup: ItemGroup = {
  ...mockSeasonGroup,
  id: 'group-002',
  groupType: 'volume',
  groupName: '第1巻',
};

const baseItem = {
  id: 'item-001',
  title: 'Test Item',
  status: 'not_started' as const,
  isFavorite: false,
  source: 'manual' as const,
  createdAt: '2026-07-01T00:00:00Z',
  updatedAt: '2026-07-01T00:00:00Z',
};

const animeItem: Item = { ...baseItem, mediaType: 'anime', details: { genreList: [] } };
const dramaItem: Item = { ...baseItem, mediaType: 'drama', details: { genreList: [] } };
const mangaItem: Item = { ...baseItem, mediaType: 'manga', details: {} };
const novelItem: Item = { ...baseItem, mediaType: 'novel', details: {} };
const gameItem: Item = { ...baseItem, mediaType: 'game', details: { platformList: [] } };

// ===== テストユーティリティ =====

function renderWithQueryClient(ui: React.ReactElement) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    createElement(QueryClientProvider, { client: queryClient }, ui)
  );
}

// ===== GroupSection テスト =====

describe('GroupSection', () => {
  beforeEach(() => {
    // 【テスト前準備】: useItemGroupsQueryのデフォルトモック設定
    vi.mocked(useItemGroupsQuery).mockReturnValue({
      data: { data: [] },
      isLoading: false,
      isError: false,
    } as unknown as ReturnType<typeof useItemGroupsQuery>);
    // 【テスト前準備】: useCreateEpisodeMutationのデフォルトモック設定
    vi.mocked(useCreateEpisodeMutation).mockReturnValue({
      mutate: vi.fn(),
      mutateAsync: vi.fn(),
      isPending: false,
      isSuccess: false,
      isError: false,
      isIdle: true,
      reset: vi.fn(),
    } as unknown as ReturnType<typeof useCreateEpisodeMutation>);
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it('TC-GS-N-01: anime のとき SeasonGroupList と話数登録ボタンが表示される', () => {
    // 【テスト目的】: mediaType='anime'のとき SeasonGroupList が表示され話数登録UIが存在すること
    // 【テスト内容】: animeのItemを渡してGroupSectionをレンダリングし、UI分岐を確認
    // 【期待される動作】: data-testid="season-group-list"が存在し、話数登録ボタンがある
    // 🔵 信頼性レベル: REQ-101 anime/drama→SeasonGroupList より

    vi.mocked(useItemGroupsQuery).mockReturnValue({
      data: { data: [mockSeasonGroup] },
      isLoading: false,
      isError: false,
    } as unknown as ReturnType<typeof useItemGroupsQuery>);

    renderWithQueryClient(<GroupSection item={animeItem} />);

    // 【結果検証】: SeasonGroupListが表示される
    expect(screen.getByTestId('season-group-list')).toBeInTheDocument(); // 【確認内容】: SeasonGroupListが表示されている 🔵
    expect(screen.getByTestId('episode-register-button')).toBeInTheDocument(); // 【確認内容】: 話数登録ボタンが存在する 🔵
  });

  it('TC-GS-N-02: drama のとき SeasonGroupList が表示される', () => {
    // 【テスト目的】: mediaType='drama'のとき animeと同じくSeasonGroupListが表示される
    // 🔵 信頼性レベル: REQ-101 anime/drama→SeasonGroupList より

    vi.mocked(useItemGroupsQuery).mockReturnValue({
      data: { data: [mockSeasonGroup] },
      isLoading: false,
      isError: false,
    } as unknown as ReturnType<typeof useItemGroupsQuery>);

    renderWithQueryClient(<GroupSection item={dramaItem} />);

    expect(screen.getByTestId('season-group-list')).toBeInTheDocument(); // 【確認内容】: drama でも SeasonGroupList が表示される 🔵
    expect(screen.getByTestId('episode-register-button')).toBeInTheDocument(); // 【確認内容】: 話数登録ボタンが存在する 🔵
  });

  it('TC-GS-N-03: manga のとき VolumeGroupList が表示され話数登録ボタンは非表示', () => {
    // 【テスト目的】: mediaType='manga'のとき VolumeGroupList が表示され話数登録UIがないこと（EDGE-004）
    // 【テスト内容】: mangaのItemを渡して話数登録ボタンが存在しないことを確認
    // 【期待される動作】: data-testid="volume-group-list"が存在し、話数登録ボタンが存在しない
    // 🔵 信頼性レベル: REQ-102 manga/novel→VolumeGroupList、EDGE-004 より

    vi.mocked(useItemGroupsQuery).mockReturnValue({
      data: { data: [mockVolumeGroup] },
      isLoading: false,
      isError: false,
    } as unknown as ReturnType<typeof useItemGroupsQuery>);

    renderWithQueryClient(<GroupSection item={mangaItem} />);

    expect(screen.getByTestId('volume-group-list')).toBeInTheDocument(); // 【確認内容】: VolumeGroupListが表示されている 🔵
    expect(screen.queryByTestId('episode-register-button')).not.toBeInTheDocument(); // 【確認内容】: 話数登録ボタンが存在しない（EDGE-004） 🔵
  });

  it('TC-GS-N-04: novel のとき VolumeGroupList が表示され話数登録ボタンは非表示', () => {
    // 【テスト目的】: mediaType='novel'でも manga と同じく VolumeGroupList が表示される
    // 🔵 信頼性レベル: REQ-102 より

    vi.mocked(useItemGroupsQuery).mockReturnValue({
      data: { data: [mockVolumeGroup] },
      isLoading: false,
      isError: false,
    } as unknown as ReturnType<typeof useItemGroupsQuery>);

    renderWithQueryClient(<GroupSection item={novelItem} />);

    expect(screen.getByTestId('volume-group-list')).toBeInTheDocument(); // 【確認内容】: novel でも VolumeGroupList が表示される 🔵
    expect(screen.queryByTestId('episode-register-button')).not.toBeInTheDocument(); // 【確認内容】: 話数登録ボタンが存在しない 🔵
  });

  it('TC-GS-N-06: game のとき何も表示されない', () => {
    // 【テスト目的】: game等グループ管理対象外のmediaTypeではnullが返ること
    // 【期待される動作】: グループ関連UI要素が一切表示されない
    // 🟡 信頼性レベル: タスクノートの「game等は対象外」より

    const { container } = renderWithQueryClient(<GroupSection item={gameItem} />);

    expect(screen.queryByTestId('season-group-list')).not.toBeInTheDocument(); // 【確認内容】: SeasonGroupListが表示されない 🟡
    expect(screen.queryByTestId('volume-group-list')).not.toBeInTheDocument(); // 【確認内容】: VolumeGroupListが表示されない 🟡
    expect(container.firstChild).toBeNull(); // 【確認内容】: コンポーネントがnullを返す 🟡
  });
});

// ===== VolumeGroupList の EDGE-004 テスト =====

describe('VolumeGroupList (EDGE-004)', () => {
  beforeEach(() => {
    vi.mocked(useItemGroupsQuery).mockReturnValue({
      data: { data: [mockVolumeGroup] },
      isLoading: false,
      isError: false,
    } as unknown as ReturnType<typeof useItemGroupsQuery>);
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it('TC-GS-E-01: EDGE-004 - VolumeGroupList には話数登録ボタンが存在しない', () => {
    // 【テスト目的】: VolumeGroupListコンポーネント直接レンダリングでも話数登録ボタンが表示されない
    // 【テスト内容】: EDGE-004制御をUI単位で検証する
    // 【期待される動作】: episode-register-button も「話数を追加」テキストも存在しない
    // 🔵 信頼性レベル: EDGE-004 - UIでバックエンドのINVALID_GROUP_TYPE_FOR_EPISODESエラーを回避 より

    renderWithQueryClient(<VolumeGroupList itemId="item-001" />);

    // 【結果検証】: 話数登録UIが一切存在しないことを確認
    expect(screen.queryByTestId('episode-register-button')).not.toBeInTheDocument(); // 【確認内容】: 話数登録ボタンが表示されない 🔵
    expect(screen.queryByText(/話数を追加/)).not.toBeInTheDocument(); // 【確認内容】: 話数追加テキストが表示されない 🔵
    expect(screen.queryByText(/エピソード/)).not.toBeInTheDocument(); // 【確認内容】: エピソード関連テキストが表示されない 🔵
  });
});

// ===== SeasonGroupList テスト =====

describe('SeasonGroupList', () => {
  beforeEach(() => {
    vi.mocked(useItemGroupsQuery).mockReturnValue({
      data: { data: [mockSeasonGroup] },
      isLoading: false,
      isError: false,
    } as unknown as ReturnType<typeof useItemGroupsQuery>);
    vi.mocked(useCreateEpisodeMutation).mockReturnValue({
      mutate: vi.fn(),
      mutateAsync: vi.fn(),
      isPending: false,
      isSuccess: false,
      isError: false,
      isIdle: true,
      reset: vi.fn(),
    } as unknown as ReturnType<typeof useCreateEpisodeMutation>);
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it('TC-GS-N-07: SeasonGroupList には話数登録ボタンが存在する', () => {
    // 【テスト目的】: SeasonGroupListコンポーネントに話数登録ボタンが表示されることを確認
    // 【期待される動作】: episode-register-button が存在する
    // 🔵 信頼性レベル: REQ-101 season用話数登録UI付き より

    renderWithQueryClient(<SeasonGroupList itemId="item-001" />);

    expect(screen.getByTestId('season-group-list')).toBeInTheDocument(); // 【確認内容】: SeasonGroupListのルート要素が存在する 🔵
    expect(screen.getByTestId('episode-register-button')).toBeInTheDocument(); // 【確認内容】: 話数登録ボタンが表示される 🔵
  });
});
