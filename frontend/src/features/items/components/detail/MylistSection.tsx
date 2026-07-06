import { useState } from 'react';
import { FiBookmark, FiPlus } from 'react-icons/fi';
import {
  useAddItemToMylistMutation,
  useItemMylistsQuery,
  useMylistsQuery,
  useRemoveItemFromMylistMutation,
} from '../../api-detail';

export function MylistSection({ itemId }: { itemId: string }) {
  const [adding, setAdding] = useState(false);
  const itemMylistsQuery = useItemMylistsQuery(itemId);
  const mylistsQuery = useMylistsQuery();
  const addToMylist = useAddItemToMylistMutation(itemId);
  const removeFromMylist = useRemoveItemFromMylistMutation(itemId);

  const memberships = itemMylistsQuery.data ?? [];
  const memberIds = new Set(memberships.map((m) => m.id));
  const options = (mylistsQuery.data ?? []).filter((m) => !memberIds.has(m.id));

  return (
    <div>
      <h3 className="mb-2.5 flex items-center gap-1.5 text-[11px] uppercase tracking-[0.05em] text-text-faint">
        <FiBookmark className="h-3.5 w-3.5 text-text-faint" />
        マイリスト
      </h3>
      {memberships.map((mylist) => (
        <div
          key={mylist.id}
          className="flex items-center justify-between border-b border-border-soft py-1.5 text-[12.5px] last:border-b-0"
        >
          <span className="text-text-primary">{mylist.name}</span>
          <button
            type="button"
            onClick={() => removeFromMylist.mutate(mylist.id)}
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
          className="mt-1.5 inline-flex items-center gap-1.5 rounded-app border border-border bg-transparent px-2.5 py-1 text-xs text-text-muted hover:bg-bg-surface-hover hover:text-text-primary"
        >
          <FiPlus className="h-3.5 w-3.5" />
          マイリストに追加
        </button>
      )}
      {adding && (
        <select
          autoFocus
          defaultValue=""
          onChange={(e) => {
            if (e.target.value) addToMylist.mutate(e.target.value);
            setAdding(false);
          }}
          onBlur={() => setAdding(false)}
          className="mt-1.5 w-full rounded-app border border-border bg-bg-input px-2.5 py-1 text-xs text-text-primary outline-none"
        >
          <option value="" disabled>
            マイリストを選択
          </option>
          {options.map((m) => (
            <option key={m.id} value={m.id}>
              {m.name}
            </option>
          ))}
        </select>
      )}
    </div>
  );
}
