/**
 * TASK-0018: SeasonGroupList - シーズン単位グループ一覧（話数登録UI付き）
 * 🔵 REQ-101: anime/drama用 season グループ表示コンポーネント
 */
import { useItemGroupsQuery, useCreateEpisodeMutation } from '@/api/groups';

interface SeasonGroupListProps {
  /** グループを取得するアイテムID */
  itemId: string;
}

/**
 * 【機能概要】: シーズン単位のグループ一覧を表示し、話数登録UIを提供する
 * 【設計方針】: anime/drama の詳細画面で使用。REQ-101により話数登録ボタンを表示する。
 *              EDGE-004の対象外（volumeではないため話数登録ボタンを表示してよい）。
 * 【保守性】: EpisodeRegisterButtonを内部コンポーネントとして分離し、SeasonGroupListの責任を限定
 * 🔵 REQ-101より
 */
export default function SeasonGroupList({ itemId }: SeasonGroupListProps) {
  const { data, isLoading, isError } = useItemGroupsQuery(itemId);

  // 【ローディング表示】: データ取得中はスピナーを表示 🟡
  if (isLoading) {
    return <div aria-live="polite">読み込み中...</div>;
  }

  // 【エラー表示】: API失敗時はエラーメッセージを表示 🟡
  if (isError) {
    return <div role="alert">シーズン情報の取得に失敗しました</div>;
  }

  const groups = data?.data ?? [];

  return (
    // 【テスト識別子】: TC-GS-N-01/02/07 で data-testid="season-group-list" を検証 🔵
    <div data-testid="season-group-list">
      {groups.length === 0 ? (
        // 【空状態表示】: グループ未登録時のメッセージ 🟡
        <p>シーズンが登録されていません</p>
      ) : (
        groups.map((group) => (
          <div key={group.id}>
            <span>{group.groupName}</span>
            {/* 【話数登録UI】: REQ-101 season配下に話数登録UIを表示 🔵 */}
            <EpisodeRegisterButton groupId={group.id} />
          </div>
        ))
      )}
    </div>
  );
}

interface EpisodeRegisterButtonProps {
  /** 話数を追加するグループID */
  groupId: string;
}

/**
 * 【機能概要】: 話数登録ボタンコンポーネント
 * 【設計方針】: SeasonGroupList内でのみ使用する内部コンポーネント。
 *              VolumeGroupListには絶対に含めない（EDGE-004）。
 * 【保守性】: 話数登録ロジックをSeasonGroupListから分離して責任を明確化
 * 🔵 REQ-101より
 */
function EpisodeRegisterButton({ groupId }: EpisodeRegisterButtonProps) {
  const mutation = useCreateEpisodeMutation(groupId);

  const handleClick = () => {
    // 【TODO】: Refactor後に話数番号入力ダイアログを実装予定 🟡
    // 現時点では最小実装として episodeNumber=1 で作成
    mutation.mutate({ episodeNumber: 1 });
  };

  return (
    // 【テスト識別子】: TC-GS-N-01/02/07 で data-testid="episode-register-button" を検証 🔵
    <button
      data-testid="episode-register-button"
      onClick={handleClick}
      disabled={mutation.isPending}
      aria-label="話数を追加する"
    >
      {mutation.isPending ? '追加中...' : '話数を追加'}
    </button>
  );
}
