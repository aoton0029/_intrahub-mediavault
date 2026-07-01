/**
 * TASK-0018: ChapterGroupList - 章単位グループ一覧（movie用、オプション）
 * 🟡 REQ-017/REQ-301: movie用 chapter グループ表示（最小実装）
 */
import { useItemGroupsQuery } from '@/api/groups';

interface ChapterGroupListProps {
  itemId: string;
}

/**
 * 【機能概要】: 章単位のグループ一覧を表示する（movie用オプション）
 * 【実装方針】: REQ-017/REQ-301はオプション要件のため最小実装。表示のみ。
 * 【テスト対応】: TC-GS-N-05
 * 🟡 REQ-017/REQ-301（オプション要件）より
 */
export default function ChapterGroupList({ itemId }: ChapterGroupListProps) {
  const { data, isLoading } = useItemGroupsQuery(itemId);

  if (isLoading) {
    return <div>読み込み中...</div>;
  }

  const groups = data?.data ?? [];

  return (
    // 【テスト識別子】: TC-GS-N-05 で data-testid="chapter-group-list" を検証 🟡
    <div data-testid="chapter-group-list">
      {groups.length === 0 && <p>章が登録されていません</p>}
      {groups.map((group) => (
        <div key={group.id}>
          <span>{group.groupName}</span>
        </div>
      ))}
    </div>
  );
}
