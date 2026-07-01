import { useState } from 'react';
import {
  useItemTrailersQuery,
  useCreateItemTrailerMutation,
  useUpdateItemTrailerMutation,
  useDeleteItemTrailerMutation,
} from '@/api/links-files';
import type { ItemTrailer } from '@/types';

interface ItemTrailersListProps {
  itemId: string;
}

export default function ItemTrailersList({ itemId }: ItemTrailersListProps) {
  const { data, isLoading, isError } = useItemTrailersQuery(itemId);
  const createMutation = useCreateItemTrailerMutation(itemId);
  const updateMutation = useUpdateItemTrailerMutation(itemId);
  const deleteMutation = useDeleteItemTrailerMutation(itemId);

  const [addingUrl, setAddingUrl] = useState('');
  const [addingLabel, setAddingLabel] = useState('');
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editUrl, setEditUrl] = useState('');
  const [editLabel, setEditLabel] = useState('');

  if (isLoading) return <div aria-live="polite">読み込み中...</div>;
  if (isError) return <div role="alert">トレーラー情報の取得に失敗しました</div>;

  const trailers = data?.data ?? [];

  const handleAdd = () => {
    if (!addingUrl.trim()) return;
    createMutation.mutate(
      { url: addingUrl.trim(), label: addingLabel.trim() || undefined },
      {
        onSuccess: () => {
          setAddingUrl('');
          setAddingLabel('');
        },
      }
    );
  };

  const startEdit = (trailer: ItemTrailer) => {
    setEditingId(trailer.id);
    setEditUrl(trailer.url);
    setEditLabel(trailer.label ?? '');
  };

  const handleUpdate = () => {
    if (!editingId) return;
    updateMutation.mutate(
      { trailerId: editingId, body: { url: editUrl.trim(), label: editLabel.trim() || undefined } },
      { onSuccess: () => setEditingId(null) }
    );
  };

  return (
    <div data-testid="item-trailers-list">
      <h4>トレーラー</h4>
      {trailers.length === 0 ? (
        <p>トレーラーが登録されていません</p>
      ) : (
        <ul>
          {trailers.map((trailer) => (
            <li key={trailer.id} data-testid="trailer-item">
              {editingId === trailer.id ? (
                <>
                  <input
                    value={editUrl}
                    onChange={(e) => setEditUrl(e.target.value)}
                    aria-label="URL"
                    data-testid="trailer-edit-url"
                  />
                  <input
                    value={editLabel}
                    onChange={(e) => setEditLabel(e.target.value)}
                    aria-label="ラベル"
                    data-testid="trailer-edit-label"
                  />
                  <button
                    onClick={handleUpdate}
                    disabled={updateMutation.isPending}
                    data-testid="trailer-save-button"
                  >
                    保存
                  </button>
                  <button onClick={() => setEditingId(null)}>キャンセル</button>
                </>
              ) : (
                <>
                  <a href={trailer.url} target="_blank" rel="noopener noreferrer">
                    {trailer.label ?? trailer.url}
                  </a>
                  <button
                    onClick={() => startEdit(trailer)}
                    aria-label="トレーラーを編集する"
                    data-testid="trailer-edit-button"
                  >
                    編集
                  </button>
                  <button
                    onClick={() => deleteMutation.mutate(trailer.id)}
                    disabled={deleteMutation.isPending}
                    aria-label="トレーラーを削除する"
                    data-testid="trailer-delete-button"
                  >
                    削除
                  </button>
                </>
              )}
            </li>
          ))}
        </ul>
      )}
      <div data-testid="trailer-add-form">
        <input
          value={addingUrl}
          onChange={(e) => setAddingUrl(e.target.value)}
          placeholder="URL"
          aria-label="新しいURL"
          data-testid="trailer-add-url"
        />
        <input
          value={addingLabel}
          onChange={(e) => setAddingLabel(e.target.value)}
          placeholder="ラベル（省略可）"
          aria-label="新しいラベル"
          data-testid="trailer-add-label"
        />
        <button
          onClick={handleAdd}
          disabled={createMutation.isPending || !addingUrl.trim()}
          data-testid="trailer-add-button"
        >
          {createMutation.isPending ? '追加中...' : 'トレーラーを追加'}
        </button>
      </div>
    </div>
  );
}
