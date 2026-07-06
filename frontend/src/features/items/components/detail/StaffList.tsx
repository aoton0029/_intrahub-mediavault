import { useState } from 'react';
import { FiPlus, FiUsers } from 'react-icons/fi';
import {
  useStaffListQuery,
  useCreateItemStaffMutation,
  useItemStaffQuery,
  useRemoveItemStaffMutation,
} from '../../api-detail';

export function StaffList({ itemId }: { itemId: string }) {
  const [adding, setAdding] = useState(false);
  const [staffId, setStaffId] = useState('');
  const [role, setRole] = useState('');
  const [characterName, setCharacterName] = useState('');

  const itemStaffQuery = useItemStaffQuery(itemId);
  const staffListQuery = useStaffListQuery();
  const createItemStaff = useCreateItemStaffMutation(itemId);
  const removeItemStaff = useRemoveItemStaffMutation(itemId);

  const staff = itemStaffQuery.data ?? [];
  const staffNameById = new Map((staffListQuery.data ?? []).map((s) => [s.id, s.name]));

  const handleSubmit = () => {
    if (!staffId || !role.trim()) return;
    createItemStaff.mutate({
      staff_id: staffId,
      role,
      character_name: characterName || undefined,
    });
    setAdding(false);
    setStaffId('');
    setRole('');
    setCharacterName('');
  };

  return (
    <div className="mb-7 max-w-[680px]">
      <h3 className="mb-2.5 flex items-center gap-1.5 border-b border-border-soft pb-1.5 text-xs uppercase tracking-[0.05em] text-text-faint">
        <FiUsers className="h-4 w-4 text-text-faint" />
        スタッフ
      </h3>
      {staff.map((entry) => (
        <div
          key={entry.id}
          className="flex items-center justify-between border-b border-border-soft py-1.5 text-[12.5px] last:border-b-0"
        >
          <div>
            <span className="text-text-primary">{staffNameById.get(entry.staff_id) ?? entry.staff_id}</span>
            <span className="ml-2 font-mono text-[11px] text-text-faint">
              {entry.role}
              {entry.character_name ? `（${entry.character_name}役）` : ''}
            </span>
          </div>
          <button
            type="button"
            onClick={() => removeItemStaff.mutate(entry.id)}
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
          className="mt-2.5 inline-flex items-center gap-1.5 rounded-app border border-border bg-transparent px-2.5 py-1 text-xs text-text-muted hover:bg-bg-surface-hover hover:text-text-primary"
        >
          <FiPlus className="h-3.5 w-3.5" />
          スタッフを追加
        </button>
      )}
      {adding && (
        <div className="mt-2.5 flex flex-wrap items-center gap-1.5">
          <select
            value={staffId}
            onChange={(e) => setStaffId(e.target.value)}
            className="rounded-app border border-border bg-bg-input px-2 py-1 text-xs text-text-primary outline-none"
          >
            <option value="" disabled>
              スタッフを選択
            </option>
            {(staffListQuery.data ?? []).map((s) => (
              <option key={s.id} value={s.id}>
                {s.name}
              </option>
            ))}
          </select>
          <input
            value={role}
            onChange={(e) => setRole(e.target.value)}
            placeholder="役割（例: 監督）"
            className="w-32 rounded-app border border-border bg-bg-input px-2 py-1 text-xs text-text-primary outline-none"
          />
          <input
            value={characterName}
            onChange={(e) => setCharacterName(e.target.value)}
            placeholder="キャラ名（任意）"
            className="w-32 rounded-app border border-border bg-bg-input px-2 py-1 text-xs text-text-primary outline-none"
          />
          <button
            type="button"
            onClick={handleSubmit}
            className="rounded-app border border-accent bg-accent px-2.5 py-1 text-xs font-medium text-white hover:bg-accent-strong"
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
      )}
    </div>
  );
}
