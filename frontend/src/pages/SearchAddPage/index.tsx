import { useState } from 'react';
import { Link } from 'react-router-dom';
import { useSetTitlebar } from '@/components/layout/useTitlebar';
import { useExternalSearchQuery } from '@/features/items/api';
import { ApiClientError } from '@/lib/api-client';
import { ExternalResultCard } from '@/features/items/components/ExternalResultCard';
import { SearchBox } from '@/features/items/components/SearchBox';
import { ErrorState } from '@/features/items/components/ErrorState';
import {
  externalProviderLabels,
  mediaTypeLabels,
  mediaTypeProvider,
  type GeneralMediaType,
} from '@/features/items/types';

const GENERAL_MEDIA_TYPES: GeneralMediaType[] = ['anime', 'movie', 'drama', 'manga', 'novel', 'game'];

export function SearchAddPage() {
  const [mediaType, setMediaType] = useState<GeneralMediaType>('anime');
  const [q, setQ] = useState('');
  const [submittedQ, setSubmittedQ] = useState('');

  const query = useExternalSearchQuery(mediaType, submittedQ);

  useSetTitlebar({
    title: '検索して追加',
    action: (
      <Link
        to="/search-add/manual"
        className="rounded-app border border-transparent px-3.5 py-1.5 text-sm text-text-muted hover:bg-bg-surface-hover hover:text-text-primary"
      >
        手動で入力する
      </Link>
    ),
  });

  const apiKeyMissing =
    query.isError && query.error instanceof ApiClientError && query.error.code === 'API_KEY_NOT_CONFIGURED';
  const otherError = query.isError && !apiKeyMissing;

  return (
    <>
      <div className="mb-5 flex flex-wrap items-center gap-2">
        <div className="inline-flex items-center gap-1.5 rounded-app border border-border bg-bg-input px-2.5 py-1 text-xs text-text-muted">
          種別
          <select
            value={mediaType}
            onChange={(e) => {
              setMediaType(e.target.value as GeneralMediaType);
              setSubmittedQ('');
            }}
            className="bg-transparent text-inherit outline-none"
          >
            {GENERAL_MEDIA_TYPES.map((value) => (
              <option key={value} value={value}>
                {mediaTypeLabels[value]}
              </option>
            ))}
          </select>
        </div>

        <div className="min-w-0 flex-1">
          <SearchBox
            value={q}
            onChange={setQ}
            placeholder="作品名で検索…"
            ariaLabel="作品名で検索"
          />
        </div>

        <button
          type="button"
          onClick={() => setSubmittedQ(q)}
          className="rounded-app border border-accent bg-accent px-3.5 py-1.5 text-sm font-medium text-white hover:bg-accent-strong"
        >
          検索
        </button>
      </div>

      {apiKeyMissing && (
        <div className="mt-6 py-16 text-center text-text-faint">
          <h3 className="mb-1.5 font-display font-semibold text-text-muted">
            APIキーが設定されていません
          </h3>
          <p>
            この種別の検索には {externalProviderLabels[mediaTypeProvider[mediaType]]} のAPIキーが必要です。設定画面から登録してください。
          </p>
          <Link
            to="/settings"
            className="mt-3 inline-block rounded-app border border-accent bg-accent px-3.5 py-1 text-sm font-medium text-white hover:bg-accent-strong"
          >
            設定を開く
          </Link>
        </div>
      )}

      {otherError && <ErrorState onRetry={() => query.refetch()} />}

      {!query.isError && query.isLoading && (
        <div className="py-16 text-center text-text-faint">検索中…</div>
      )}

      {!query.isError && !query.isLoading && submittedQ.trim().length === 0 && (
        <div className="py-16 text-center text-text-faint">作品名を入力して検索してください。</div>
      )}

      {!query.isError && !query.isLoading && submittedQ.trim().length > 0 && query.data?.length === 0 && (
        <div className="py-16 text-center text-text-faint">該当する検索結果がありません。</div>
      )}

      {!query.isError && query.data && query.data.length > 0 && (
        <div className="grid grid-cols-[repeat(auto-fill,minmax(168px,1fr))] gap-4">
          {query.data.map((result) => (
            <ExternalResultCard key={result.external_id} result={result} mediaType={mediaType} />
          ))}
        </div>
      )}
    </>
  );
}
