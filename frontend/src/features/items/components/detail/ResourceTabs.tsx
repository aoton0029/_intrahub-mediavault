import { useState } from 'react';
import { FiFilm, FiLink2, FiPaperclip, FiPlus, FiTrash2 } from 'react-icons/fi';
import {
  useCreateItemFileMutation,
  useCreateItemLinkMutation,
  useCreateItemTrailerMutation,
  useItemFilesQuery,
  useItemLinksQuery,
  useItemTrailersQuery,
  useRemoveItemFileMutation,
  useRemoveItemLinkMutation,
  useRemoveItemTrailerMutation,
} from '../../api-detail';
import type { FileType } from '../../types-detail';

type Tab = 'links' | 'files' | 'trailers';

function LinksPanel({ itemId }: { itemId: string }) {
  const [adding, setAdding] = useState(false);
  const [url, setUrl] = useState('');
  const [label, setLabel] = useState('');
  const linksQuery = useItemLinksQuery(itemId);
  const createLink = useCreateItemLinkMutation(itemId);
  const removeLink = useRemoveItemLinkMutation(itemId);

  return (
    <div>
      {(linksQuery.data ?? []).map((link) => (
        <div
          key={link.id}
          className="flex items-center justify-between border-b border-border-soft py-1.5 text-[12.5px] last:border-b-0"
        >
          <span className="text-text-primary">{link.label}</span>
          <div className="flex items-center gap-2">
            <span className="font-mono text-xs text-text-faint">{link.url}</span>
            <button
              type="button"
              onClick={() => removeLink.mutate(link.id)}
              className="rounded-app border border-border bg-bg-surface px-2.5 py-1 text-xs text-danger hover:border-danger hover:bg-danger/10"
            >
              <FiTrash2 className="mr-1 inline h-3 w-3" />
              削除
            </button>
          </div>
        </div>
      ))}
      {!adding ? (
        <button
          type="button"
          onClick={() => setAdding(true)}
          className="mt-2.5 inline-flex items-center gap-1.5 rounded-app border border-border bg-transparent px-2.5 py-1 text-xs text-text-muted hover:bg-bg-surface-hover hover:text-text-primary"
        >
          <FiPlus className="h-3.5 w-3.5" />
          リンクを追加
        </button>
      ) : (
        <div className="mt-2.5 flex items-center gap-2">
          <input
            value={label}
            onChange={(e) => setLabel(e.target.value)}
            placeholder="ラベル"
            className="w-28 rounded-app border border-border bg-bg-input px-2 py-1 text-xs text-text-primary outline-none"
          />
          <input
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            placeholder="https://..."
            className="flex-1 rounded-app border border-border bg-bg-input px-2 py-1 text-xs text-text-primary outline-none"
          />
          <button
            type="button"
            onClick={() => {
              if (!url.trim() || !label.trim()) return;
              createLink.mutate({ url, label });
              setAdding(false);
              setUrl('');
              setLabel('');
            }}
            className="rounded-app border border-accent bg-accent px-2.5 py-1 text-xs font-medium text-white hover:bg-accent-strong"
          >
            追加
          </button>
        </div>
      )}
    </div>
  );
}

function FilesPanel({ itemId }: { itemId: string }) {
  const [adding, setAdding] = useState(false);
  const [path, setPath] = useState('');
  const [label, setLabel] = useState('');
  const [fileType, setFileType] = useState<FileType>('pdf');
  const filesQuery = useItemFilesQuery(itemId);
  const createFile = useCreateItemFileMutation(itemId);
  const removeFile = useRemoveItemFileMutation(itemId);

  return (
    <div>
      {(filesQuery.data ?? []).map((file) => (
        <div
          key={file.id}
          className="flex items-center justify-between border-b border-border-soft py-1.5 text-[12.5px] last:border-b-0"
        >
          <span className="text-text-primary">{file.label ?? file.path}</span>
          <div className="flex items-center gap-2">
            <span className="font-mono text-xs text-text-faint">{file.file_type}</span>
            <button
              type="button"
              onClick={() => removeFile.mutate(file.id)}
              className="rounded-app border border-border bg-bg-surface px-2.5 py-1 text-xs text-danger hover:border-danger hover:bg-danger/10"
            >
              <FiTrash2 className="mr-1 inline h-3 w-3" />
              削除
            </button>
          </div>
        </div>
      ))}
      {!adding ? (
        <button
          type="button"
          onClick={() => setAdding(true)}
          className="mt-2.5 inline-flex items-center gap-1.5 rounded-app border border-border bg-transparent px-2.5 py-1 text-xs text-text-muted hover:bg-bg-surface-hover hover:text-text-primary"
        >
          <FiPlus className="h-3.5 w-3.5" />
          ファイルを追加
        </button>
      ) : (
        <div className="mt-2.5 flex items-center gap-2">
          <select
            value={fileType}
            onChange={(e) => setFileType(e.target.value as FileType)}
            className="rounded-app border border-border bg-bg-input px-2 py-1 text-xs text-text-primary outline-none"
          >
            <option value="pdf">pdf</option>
            <option value="image">image</option>
            <option value="other">other</option>
          </select>
          <input
            value={label}
            onChange={(e) => setLabel(e.target.value)}
            placeholder="ラベル（任意）"
            className="w-28 rounded-app border border-border bg-bg-input px-2 py-1 text-xs text-text-primary outline-none"
          />
          <input
            value={path}
            onChange={(e) => setPath(e.target.value)}
            placeholder="ファイルパス"
            className="flex-1 rounded-app border border-border bg-bg-input px-2 py-1 text-xs text-text-primary outline-none"
          />
          <button
            type="button"
            onClick={() => {
              if (!path.trim()) return;
              createFile.mutate({ path, label: label || undefined, file_type: fileType });
              setAdding(false);
              setPath('');
              setLabel('');
            }}
            className="rounded-app border border-accent bg-accent px-2.5 py-1 text-xs font-medium text-white hover:bg-accent-strong"
          >
            追加
          </button>
        </div>
      )}
    </div>
  );
}

function TrailersPanel({ itemId }: { itemId: string }) {
  const [adding, setAdding] = useState(false);
  const [url, setUrl] = useState('');
  const [label, setLabel] = useState('');
  const trailersQuery = useItemTrailersQuery(itemId);
  const createTrailer = useCreateItemTrailerMutation(itemId);
  const removeTrailer = useRemoveItemTrailerMutation(itemId);

  return (
    <div>
      {(trailersQuery.data ?? []).map((trailer) => (
        <div
          key={trailer.id}
          className="flex items-center justify-between border-b border-border-soft py-1.5 text-[12.5px] last:border-b-0"
        >
          <span className="text-text-primary">{trailer.label ?? 'トレーラー'}</span>
          <div className="flex items-center gap-2">
            <span className="font-mono text-xs text-text-faint">{trailer.url}</span>
            <button
              type="button"
              onClick={() => removeTrailer.mutate(trailer.id)}
              className="rounded-app border border-border bg-bg-surface px-2.5 py-1 text-xs text-danger hover:border-danger hover:bg-danger/10"
            >
              <FiTrash2 className="mr-1 inline h-3 w-3" />
              削除
            </button>
          </div>
        </div>
      ))}
      {!adding ? (
        <button
          type="button"
          onClick={() => setAdding(true)}
          className="mt-2.5 inline-flex items-center gap-1.5 rounded-app border border-border bg-transparent px-2.5 py-1 text-xs text-text-muted hover:bg-bg-surface-hover hover:text-text-primary"
        >
          <FiPlus className="h-3.5 w-3.5" />
          トレーラーを追加
        </button>
      ) : (
        <div className="mt-2.5 flex items-center gap-2">
          <input
            value={label}
            onChange={(e) => setLabel(e.target.value)}
            placeholder="ラベル（任意）"
            className="w-28 rounded-app border border-border bg-bg-input px-2 py-1 text-xs text-text-primary outline-none"
          />
          <input
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            placeholder="https://..."
            className="flex-1 rounded-app border border-border bg-bg-input px-2 py-1 text-xs text-text-primary outline-none"
          />
          <button
            type="button"
            onClick={() => {
              if (!url.trim()) return;
              createTrailer.mutate({ url, label: label || undefined });
              setAdding(false);
              setUrl('');
              setLabel('');
            }}
            className="rounded-app border border-accent bg-accent px-2.5 py-1 text-xs font-medium text-white hover:bg-accent-strong"
          >
            追加
          </button>
        </div>
      )}
    </div>
  );
}

export function ResourceTabs({ itemId }: { itemId: string }) {
  const [tab, setTab] = useState<Tab>('links');

  const tabs: { key: Tab; label: string; icon: typeof FiLink2 }[] = [
    { key: 'links', label: 'リンク', icon: FiLink2 },
    { key: 'files', label: 'ファイル', icon: FiPaperclip },
    { key: 'trailers', label: 'トレーラー', icon: FiFilm },
  ];

  return (
    <div className="max-w-[680px]">
      <h3 className="mb-2.5 flex items-center gap-1.5 border-b border-border-soft pb-1.5 text-xs uppercase tracking-[0.05em] text-text-faint">
        <FiPaperclip className="h-4 w-4 text-text-faint" />
        リソース
      </h3>
      <div className="mb-3 flex gap-1.5">
        {tabs.map(({ key, label, icon: Icon }) => (
          <button
            key={key}
            type="button"
            onClick={() => setTab(key)}
            className={`inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs ${
              tab === key
                ? 'border-accent bg-accent-soft text-accent-strong'
                : 'border-border bg-bg-surface text-text-muted hover:border-accent hover:text-text-primary'
            }`}
          >
            <Icon className="h-3.5 w-3.5" />
            {label}
          </button>
        ))}
      </div>
      {tab === 'links' && <LinksPanel itemId={itemId} />}
      {tab === 'files' && <FilesPanel itemId={itemId} />}
      {tab === 'trailers' && <TrailersPanel itemId={itemId} />}
    </div>
  );
}
