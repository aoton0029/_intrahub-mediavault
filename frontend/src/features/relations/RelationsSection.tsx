import { useState } from 'react';
import type { Item } from '@/types';
import { useItemRelationsQuery, useDeleteItemRelationMutation } from '@/api/relations';
import DlcList from './DlcList';
import RelationPickerDialog from './RelationPickerDialog';

interface RelationsSectionProps {
  item: Item;
}

export default function RelationsSection({ item }: RelationsSectionProps) {
  const [dialogOpen, setDialogOpen] = useState(false);
  const { data, isLoading, isError } = useItemRelationsQuery(item.id);
  const deleteMutation = useDeleteItemRelationMutation(item.id);

  const references = (data?.data ?? []).filter((r) => r.relationType === 'reference');

  return (
    <div data-testid="relations-section">
      {item.mediaType === 'game' && <DlcList itemId={item.id} />}

      <div data-testid="general-relations-list">
        <h3>関連アイテム</h3>
        {isLoading && <div aria-live="polite">読み込み中...</div>}
        {isError && <div role="alert">関連情報の取得に失敗しました</div>}
        {!isLoading && !isError && (
          <>
            {references.length === 0 ? (
              <p>関連アイテムが登録されていません</p>
            ) : (
              <ul>
                {references.map((relation) => (
                  <li key={relation.id} data-testid="relation-item">
                    <span>{relation.relatedItemId}</span>
                    <button
                      onClick={() => deleteMutation.mutate(relation.id)}
                      disabled={deleteMutation.isPending}
                      aria-label="関連付けを削除する"
                    >
                      削除
                    </button>
                  </li>
                ))}
              </ul>
            )}
            <button onClick={() => setDialogOpen(true)}>関連アイテムを追加</button>
          </>
        )}
      </div>

      <RelationPickerDialog
        itemId={item.id}
        relationType="reference"
        open={dialogOpen}
        onClose={() => setDialogOpen(false)}
      />
    </div>
  );
}
