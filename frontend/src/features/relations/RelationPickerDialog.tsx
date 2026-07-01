import { useState } from 'react';
import { useCreateItemRelationMutation } from '@/api/relations';
import type { RelationType } from '@/types';

interface RelationPickerDialogProps {
  itemId: string;
  relationType: RelationType;
  open: boolean;
  onClose: () => void;
}

export default function RelationPickerDialog({
  itemId,
  relationType,
  open,
  onClose,
}: RelationPickerDialogProps) {
  const [relatedItemId, setRelatedItemId] = useState('');
  const mutation = useCreateItemRelationMutation(itemId);

  if (!open) return null;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!relatedItemId.trim()) return;

    mutation.mutate(
      { relatedItemId: relatedItemId.trim(), relationType },
      {
        onSuccess: () => {
          setRelatedItemId('');
          onClose();
        },
      }
    );
  };

  return (
    <div role="dialog" aria-modal="true" aria-label="関連アイテムを選択">
      <form onSubmit={handleSubmit}>
        <label>
          関連アイテムID
          <input
            value={relatedItemId}
            onChange={(e) => setRelatedItemId(e.target.value)}
            placeholder="アイテムIDを入力"
            aria-label="関連アイテムID"
          />
        </label>
        {mutation.isError && (
          <div role="alert">関連付けの追加に失敗しました</div>
        )}
        <button type="submit" disabled={mutation.isPending || !relatedItemId.trim()}>
          {mutation.isPending ? '追加中...' : '関連付けを追加'}
        </button>
        <button type="button" onClick={onClose}>
          キャンセル
        </button>
      </form>
    </div>
  );
}
