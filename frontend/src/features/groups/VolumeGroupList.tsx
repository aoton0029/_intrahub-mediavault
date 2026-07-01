/**
 * TASK-0018: VolumeGroupList - 巻単位グループ一覧（話数登録UIなし）
 * 🔵 REQ-102 + EDGE-004: manga/novel用 volume グループ表示コンポーネント
 *
 * EDGE-004: volume グループには話数登録ボタンを絶対に表示しない。
 * これによりバックエンドの INVALID_GROUP_TYPE_FOR_EPISODES エラーを未然に回避する。
 * エラーハンドリングではなく、UI設計レベルで誤操作経路を排除する。
 */
import { useItemGroupsQuery } from '@/api/groups';

interface VolumeGroupListProps {
  /** グループを取得するアイテムID */
  itemId: string;
}

/**
 * 【機能概要】: 巻単位のグループ一覧を表示する（話数登録UIなし）
 * 【設計方針】: manga/novel の詳細画面で使用。
 *              EDGE-004に従い、話数登録UIは一切含めない設計とする。
 *              useCreateEpisodeMutation も import しない（誤実装防止）。
 * 【保守性】: groups.ts から useItemGroupsQuery のみを使用し、
 *             episode関連フックを依存関係に含めないことで設計を明確化
 * 🔵 REQ-102, EDGE-004より
 */
export default function VolumeGroupList({ itemId }: VolumeGroupListProps) {
  const { data, isLoading, isError } = useItemGroupsQuery(itemId);

  // 【ローディング表示】: データ取得中はスピナーを表示 🟡
  if (isLoading) {
    return <div aria-live="polite">読み込み中...</div>;
  }

  // 【エラー表示】: API失敗時はエラーメッセージを表示 🟡
  if (isError) {
    return <div role="alert">巻情報の取得に失敗しました</div>;
  }

  const groups = data?.data ?? [];

  return (
    // 【テスト識別子】: TC-GS-N-03/04/E-01 で data-testid="volume-group-list" を検証 🔵
    <div data-testid="volume-group-list">
      {groups.length === 0 ? (
        // 【空状態表示】: グループ未登録時のメッセージ 🟡
        <p>巻が登録されていません</p>
      ) : (
        groups.map((group) => (
          <div key={group.id}>
            <span>{group.groupName}</span>
            {/*
             * 【EDGE-004 設計注記】:
             * ここに話数登録ボタンを追加してはならない。
             * volume グループへの話数登録は INVALID_GROUP_TYPE_FOR_EPISODES を引き起こす。
             * 話数管理が必要な場合は SeasonGroupList (anime/drama) を使用すること。
             */}
          </div>
        ))
      )}
    </div>
  );
}
