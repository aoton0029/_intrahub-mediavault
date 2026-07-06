import { useState } from 'react';
import { FiFolder, FiTag } from 'react-icons/fi';
import { useCategoriesQuery, useTagsQuery } from '../../api';
import type { CategoryRef, TagRef } from '../../types';
import {
  useAddItemCategoryMutation,
  useAddItemTagMutation,
  useRemoveItemCategoryMutation,
  useRemoveItemTagMutation,
} from '../../api-detail';

interface TagCategorySectionProps {
  itemId: string;
  kind: 'tag' | 'category';
  items: TagRef[] | CategoryRef[];
}

export function TagCategorySection({ itemId, kind, items }: TagCategorySectionProps) {
  const [adding, setAdding] = useState(false);

  const tagsQuery = useTagsQuery();
  const categoriesQuery = useCategoriesQuery();
  const addTag = useAddItemTagMutation(itemId);
  const removeTag = useRemoveItemTagMutation(itemId);
  const addCategory = useAddItemCategoryMutation(itemId);
  const removeCategory = useRemoveItemCategoryMutation(itemId);

  const isTag = kind === 'tag';
  const label = isTag ? 'タグ' : 'カテゴリ';
  const Icon = isTag ? FiTag : FiFolder;
  const candidates = (isTag ? tagsQuery.data : categoriesQuery.data) ?? [];
  const attachedIds = new Set(items.map((i) => i.id));
  const options = candidates.filter((c) => !attachedIds.has(c.id));

  const handleAdd = (id: string) => {
    if (!id) return;
    if (isTag) addTag.mutate(id);
    else addCategory.mutate(id);
    setAdding(false);
  };

  const handleRemove = (id: string) => {
    if (isTag) removeTag.mutate(id);
    else removeCategory.mutate(id);
  };

  return (
    <div className="mb-[18px]">
      <h3 className="mb-2.5 flex items-center gap-1.5 text-[11px] uppercase tracking-[0.05em] text-text-faint">
        <Icon className="h-3.5 w-3.5 text-text-faint" />
        {label}
      </h3>
      <div className="flex flex-wrap gap-1.5">
        {items.map((entry) => (
          <span
            key={entry.id}
            className="group inline-flex items-center gap-1 rounded bg-accent-soft px-1.5 py-0.5 font-mono text-[11px] text-accent-strong"
          >
            #{entry.name}
            <button
              type="button"
              aria-label={`${entry.name}を削除`}
              onClick={() => handleRemove(entry.id)}
              className="hidden text-text-faint hover:text-danger group-hover:inline"
            >
              ×
            </button>
          </span>
        ))}
        {!adding && (
          <button
            type="button"
            onClick={() => setAdding(true)}
            className="rounded border border-dashed border-border px-1.5 py-0.5 font-mono text-[11px] text-text-faint hover:border-accent hover:text-text-primary"
          >
            + 追加
          </button>
        )}
        {adding && (
          <select
            autoFocus
            defaultValue=""
            onChange={(e) => handleAdd(e.target.value)}
            onBlur={() => setAdding(false)}
            className="rounded border border-border bg-bg-input px-1.5 py-0.5 text-[11px] text-text-primary outline-none"
          >
            <option value="" disabled>
              選択してください
            </option>
            {options.map((c) => (
              <option key={c.id} value={c.id}>
                {c.name}
              </option>
            ))}
          </select>
        )}
      </div>
    </div>
  );
}
