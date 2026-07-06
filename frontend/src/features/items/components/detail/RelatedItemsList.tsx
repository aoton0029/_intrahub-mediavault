import { useState } from 'react';
import { useQueries } from '@tanstack/react-query';
import { FiGitBranch, FiPlus } from 'react-icons/fi';
import { apiFetch } from '@/lib/api-client';
import { useInfiniteItemsQuery } from '../../api';
import {
  useCreateItemRelationMutation,
  useItemRelationsQuery,
  useRemoveItemRelationMutation,
} from '../../api-detail';
import type { ItemDetail, RelationType } from '../../types-detail';

export function RelatedItemsList({ itemId }: { itemId: string }) {
  const [adding, setAdding] = useState(false);
  const [titleQuery, setTitleQuery] = useState('');
  const [relatedItemId, setRelatedItemId] = useState('');
  const [relationType, setRelationType] = useState<RelationType>('reference');

  const relationsQuery = useItemRelationsQuery(itemId);
  const relations = relationsQuery.data ?? [];

  const searchQuery = useInfiniteItemsQuery({ title: titleQuery });
  const searchResults = searchQuery.data?.pages.flatMap((page) => page.data) ?? [];

  const relatedItemQueries = useQueries({
    queries: relations.map((relation) => ({
      queryKey: ['item', relation.related_item_id],
      queryFn: async () =>
        (await apiFetch<ItemDetail>(`/items/${relation.related_item_id}`)).data,
    })),
  });
  const relatedTitleById = new Map(
    relations.map((relation, index) => [
      relation.related_item_id,
      relatedItemQueries[index]?.data?.title,
    ]),
  );

  const createRelation = useCreateItemRelationMutation(itemId);
  const removeRelation = useRemoveItemRelationMutation(itemId);

  const handleSubmit = () => {
    if (!relatedItemId) return;
    createRelation.mutate({ item_id: itemId, related_item_id: relatedItemId, relation_type: relationType });
    setAdding(false);
    setTitleQuery('');
    setRelatedItemId('');
  };

  return (
    <div className="mb-7 max-w-[680px]">
      <h3 className="mb-2.5 flex items-center gap-1.5 border-b border-border-soft pb-1.5 text-xs uppercase tracking-[0.05em] text-text-faint">
        <FiGitBranch className="h-4 w-4 text-text-faint" />
        関連作品
      </h3>
      {relations.map((relation) => (
        <div
          key={relation.id}
          className="mb-2.5 flex items-center gap-3.5 rounded-app border border-border-soft bg-bg-surface p-3.5"
        >
          <div className="h-[72px] w-[52px] flex-shrink-0 rounded bg-[linear-gradient(160deg,#33304a,#232323)]" />
          <div className="min-w-0 flex-1">
            <div className="mb-0.5 font-display text-sm font-semibold text-text-primary">
              {relatedTitleById.get(relation.related_item_id) ?? relation.related_item_id}
            </div>
            <div className="font-mono text-xs text-text-faint">{relation.relation_type}</div>
          </div>
          <button
            type="button"
            onClick={() => removeRelation.mutate(relation.id)}
            className="rounded-app border border-border bg-bg-surface px-2.5 py-1 text-xs text-danger hover:border-danger hover:bg-danger/10"
          >
            解除
          </button>
        </div>
      ))}

      {!adding && (
        <button
          type="button"
          onClick={() => setAdding(true)}
          className="inline-flex items-center gap-1.5 rounded-app border border-border bg-transparent px-2.5 py-1 text-xs text-text-muted hover:bg-bg-surface-hover hover:text-text-primary"
        >
          <FiPlus className="h-3.5 w-3.5" />
          関連作品を追加
        </button>
      )}
      {adding && (
        <div className="rounded-app border border-border-soft bg-bg-surface p-3">
          <input
            value={titleQuery}
            onChange={(e) => {
              setTitleQuery(e.target.value);
              setRelatedItemId('');
            }}
            placeholder="タイトルで検索…"
            className="mb-2 w-full rounded-app border border-border bg-bg-input px-2.5 py-1.5 text-xs text-text-primary outline-none"
          />
          {titleQuery && (
            <div className="mb-2 max-h-40 overflow-y-auto">
              {searchResults
                .filter((result) => result.id !== itemId)
                .map((result) => (
                  <button
                    key={result.id}
                    type="button"
                    onClick={() => setRelatedItemId(result.id)}
                    className={`block w-full truncate rounded px-2 py-1 text-left text-xs ${
                      relatedItemId === result.id
                        ? 'bg-accent-soft text-accent-strong'
                        : 'text-text-muted hover:bg-bg-surface-hover'
                    }`}
                  >
                    {result.title}
                  </button>
                ))}
            </div>
          )}
          <div className="flex items-center gap-2">
            <select
              value={relationType}
              onChange={(e) => setRelationType(e.target.value as RelationType)}
              className="rounded-app border border-border bg-bg-input px-2 py-1 text-xs text-text-primary outline-none"
            >
              <option value="reference">reference</option>
              <option value="dlc">dlc</option>
            </select>
            <button
              type="button"
              onClick={handleSubmit}
              disabled={!relatedItemId}
              className="rounded-app border border-accent bg-accent px-2.5 py-1 text-xs font-medium text-white hover:bg-accent-strong disabled:opacity-50"
            >
              追加
            </button>
            <button
              type="button"
              onClick={() => setAdding(false)}
              className="rounded-app border border-border bg-transparent px-2.5 py-1 text-xs text-text-muted hover:bg-bg-surface-hover"
            >
              キャンセル
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
