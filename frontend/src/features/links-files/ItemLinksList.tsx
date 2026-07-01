import { useState } from 'react';
import {
  useItemLinksQuery,
  useCreateItemLinkMutation,
  useUpdateItemLinkMutation,
  useDeleteItemLinkMutation,
} from '@/api/links-files';
import type { ItemLink } from '@/types';

interface ItemLinksListProps {
  itemId: string;
}

export default function ItemLinksList({ itemId }: ItemLinksListProps) {
  const { data, isLoading, isError } = useItemLinksQuery(itemId);
  const createMutation = useCreateItemLinkMutation(itemId);
  const deleteMutation = useDeleteItemLinkMutation(itemId);

  const [addingUrl, setAddingUrl] = useState('');
  const [addingLabel, setAddingLabel] = useState('');
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editUrl, setEditUrl] = useState('');
  const [editLabel, setEditLabel] = useState('');
  const updateMutation = useUpdateItemLinkMutation(itemId);

  if (isLoading) return <div aria-live="polite">読み込み中...</div>;
  if (isError) return <div role="alert">リンク情報の取得に失敗しました</div>;

  const links = data?.data ?? [];

  const handleAdd = () => {
    if (!addingUrl.trim()) return;
    createMutation.mutate(
      { url: addingUrl.trim(), label: addingLabel.trim() || addingUrl.trim() },
      {
        onSuccess: () => {
          setAddingUrl('');
          setAddingLabel('');
        },
      }
    );
  };

  const startEdit = (link: ItemLink) => {
    setEditingId(link.id);
    setEditUrl(link.url);
    setEditLabel(link.label);
  };

  const handleUpdate = () => {
    if (!editingId) return;
    updateMutation.mutate(
      { linkId: editingId, body: { url: editUrl.trim(), label: editLabel.trim() } },
      { onSuccess: () => setEditingId(null) }
    );
  };

  return (
    <div data-testid="item-links-list">
      <h4>配信サイトリンク</h4>
      {links.length === 0 ? (
        <p>リンクが登録されていません</p>
      ) : (
        <ul>
          {links.map((link) => (
            <li key={link.id} data-testid="link-item">
              {editingId === link.id ? (
                <>
                  <input
                    value={editUrl}
                    onChange={(e) => setEditUrl(e.target.value)}
                    aria-label="URL"
                    data-testid="link-edit-url"
                  />
                  <input
                    value={editLabel}
                    onChange={(e) => setEditLabel(e.target.value)}
                    aria-label="ラベル"
                    data-testid="link-edit-label"
                  />
                  <button
                    onClick={handleUpdate}
                    disabled={updateMutation.isPending}
                    data-testid="link-save-button"
                  >
                    保存
                  </button>
                  <button onClick={() => setEditingId(null)}>キャンセル</button>
                </>
              ) : (
                <>
                  <a href={link.url} target="_blank" rel="noopener noreferrer">
                    {link.label}
                  </a>
                  <button
                    onClick={() => startEdit(link)}
                    aria-label="リンクを編集する"
                    data-testid="link-edit-button"
                  >
                    編集
                  </button>
                  <button
                    onClick={() => deleteMutation.mutate(link.id)}
                    disabled={deleteMutation.isPending}
                    aria-label="リンクを削除する"
                    data-testid="link-delete-button"
                  >
                    削除
                  </button>
                </>
              )}
            </li>
          ))}
        </ul>
      )}
      <div data-testid="link-add-form">
        <input
          value={addingUrl}
          onChange={(e) => setAddingUrl(e.target.value)}
          placeholder="URL"
          aria-label="新しいURL"
          data-testid="link-add-url"
        />
        <input
          value={addingLabel}
          onChange={(e) => setAddingLabel(e.target.value)}
          placeholder="ラベル（省略可）"
          aria-label="新しいラベル"
          data-testid="link-add-label"
        />
        <button
          onClick={handleAdd}
          disabled={createMutation.isPending || !addingUrl.trim()}
          data-testid="link-add-button"
        >
          {createMutation.isPending ? '追加中...' : 'リンクを追加'}
        </button>
      </div>
    </div>
  );
}
