import { useState } from 'react';
import { FiLayers, FiPlus } from 'react-icons/fi';
import {
  useCreateItemEpisodeMutation,
  useCreateItemGroupMutation,
  useGroupEpisodesQuery,
  useItemGroupsQuery,
} from '../../api-detail';
import type { GroupType, ItemGroup } from '../../types-detail';

function GroupBlock({ group }: { group: ItemGroup }) {
  const [addingEpisode, setAddingEpisode] = useState(false);
  const [episodeTitle, setEpisodeTitle] = useState('');
  const episodesQuery = useGroupEpisodesQuery(group.id);
  const createEpisode = useCreateItemEpisodeMutation(group.id);
  const episodes = episodesQuery.data ?? [];
  const canAddEpisodes = group.group_type !== 'volume';

  const handleAddEpisode = () => {
    const nextNumber = (episodes[episodes.length - 1]?.episode_number ?? 0) + 1;
    createEpisode.mutate({ episode_number: nextNumber, title: episodeTitle || undefined });
    setAddingEpisode(false);
    setEpisodeTitle('');
  };

  return (
    <div className="mb-2.5 overflow-hidden rounded-app border border-border-soft">
      <div className="flex items-center justify-between bg-bg-surface px-3.5 py-2.5 text-sm font-semibold">
        <span>{group.group_name}</span>
        {canAddEpisodes && !addingEpisode && (
          <button
            type="button"
            onClick={() => setAddingEpisode(true)}
            className="flex items-center gap-1 rounded-app px-2 py-1 text-xs font-normal text-text-muted hover:bg-bg-surface-hover hover:text-text-primary"
          >
            <FiPlus className="h-3.5 w-3.5" />
            話数を追加
          </button>
        )}
      </div>
      {episodes.map((episode) => (
        <div
          key={episode.id}
          className="flex gap-2.5 border-t border-border-soft px-3.5 py-2 text-[12.5px] text-text-muted"
        >
          <span className="w-7 font-mono text-text-faint">
            {String(episode.episode_number).padStart(2, '0')}
          </span>
          <span>{episode.title}</span>
        </div>
      ))}
      {addingEpisode && (
        <div className="flex items-center gap-2 border-t border-border-soft px-3.5 py-2">
          <input
            value={episodeTitle}
            onChange={(e) => setEpisodeTitle(e.target.value)}
            placeholder="話数タイトル（任意）"
            className="flex-1 rounded-app border border-border bg-bg-input px-2 py-1 text-xs text-text-primary outline-none"
          />
          <button
            type="button"
            onClick={handleAddEpisode}
            className="rounded-app border border-accent bg-accent px-2.5 py-1 text-xs font-medium text-white hover:bg-accent-strong"
          >
            追加
          </button>
        </div>
      )}
    </div>
  );
}

export function GroupEpisodeList({ itemId }: { itemId: string }) {
  const [addingGroup, setAddingGroup] = useState(false);
  const [groupName, setGroupName] = useState('');
  const [groupType, setGroupType] = useState<GroupType>('season');

  const groupsQuery = useItemGroupsQuery(itemId);
  const createGroup = useCreateItemGroupMutation(itemId);
  const groups = groupsQuery.data ?? [];

  const handleAddGroup = () => {
    if (!groupName.trim()) return;
    createGroup.mutate({ group_type: groupType, group_name: groupName });
    setAddingGroup(false);
    setGroupName('');
  };

  return (
    <div className="mb-7 max-w-[680px]">
      <h3 className="mb-2.5 flex items-center gap-1.5 border-b border-border-soft pb-1.5 text-xs uppercase tracking-[0.05em] text-text-faint">
        <FiLayers className="h-4 w-4 text-text-faint" />
        シーズン構成
      </h3>
      {groups.map((group) => (
        <GroupBlock key={group.id} group={group} />
      ))}

      {!addingGroup && (
        <button
          type="button"
          onClick={() => setAddingGroup(true)}
          className="inline-flex items-center gap-1.5 rounded-app border border-border bg-transparent px-2.5 py-1 text-xs text-text-muted hover:bg-bg-surface-hover hover:text-text-primary"
        >
          <FiPlus className="h-3.5 w-3.5" />
          シーズンを追加
        </button>
      )}
      {addingGroup && (
        <div className="flex items-center gap-2">
          <select
            value={groupType}
            onChange={(e) => setGroupType(e.target.value as GroupType)}
            className="rounded-app border border-border bg-bg-input px-2 py-1 text-xs text-text-primary outline-none"
          >
            <option value="season">season</option>
            <option value="volume">volume</option>
            <option value="chapter">chapter</option>
          </select>
          <input
            value={groupName}
            onChange={(e) => setGroupName(e.target.value)}
            placeholder="名称（例: シーズン2）"
            className="rounded-app border border-border bg-bg-input px-2 py-1 text-xs text-text-primary outline-none"
          />
          <button
            type="button"
            onClick={handleAddGroup}
            className="rounded-app border border-accent bg-accent px-2.5 py-1 text-xs font-medium text-white hover:bg-accent-strong"
          >
            追加
          </button>
          <button
            type="button"
            onClick={() => setAddingGroup(false)}
            className="rounded-app border border-border bg-transparent px-2.5 py-1 text-xs text-text-muted hover:bg-bg-surface-hover"
          >
            キャンセル
          </button>
        </div>
      )}
    </div>
  );
}
