import { useItemRelationsQuery, useDeleteItemRelationMutation } from '@/api/relations';

interface DlcListProps {
  itemId: string;
}

export default function DlcList({ itemId }: DlcListProps) {
  const { data, isLoading, isError } = useItemRelationsQuery(itemId);
  const deleteMutation = useDeleteItemRelationMutation(itemId);

  if (isLoading) {
    return <div aria-live="polite">読み込み中...</div>;
  }

  if (isError) {
    return <div role="alert">DLC情報の取得に失敗しました</div>;
  }

  const dlcs = (data?.data ?? []).filter((r) => r.relationType === 'dlc');

  return (
    <div data-testid="dlc-list">
      <h3>DLC一覧</h3>
      {dlcs.length === 0 ? (
        <p>DLCが登録されていません</p>
      ) : (
        <ul>
          {dlcs.map((relation) => (
            <li key={relation.id} data-testid="dlc-item">
              <span>{relation.relatedItemId}</span>
              <button
                onClick={() => deleteMutation.mutate(relation.id)}
                disabled={deleteMutation.isPending}
                aria-label="DLCの関連付けを削除する"
              >
                削除
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
