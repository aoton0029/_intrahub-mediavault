import { ApiClientError } from '@/lib/api-client';
import { useImportItemMutation } from '../api';
import {
  externalProviderLabels,
  mediaTypeLabels,
  type ExternalSearchResult,
  type GeneralMediaType,
} from '../types';
import { getCoverImageUrl, getOriginalTitle, getReleaseYear } from '../lib/externalSearchResult';

export function ExternalResultCard({
  result,
  mediaType,
}: {
  result: ExternalSearchResult;
  mediaType: GeneralMediaType;
}) {
  const importMutation = useImportItemMutation();

  const originalTitle = getOriginalTitle(result);
  const coverImageUrl = getCoverImageUrl(result);
  const releaseYear = getReleaseYear(result);

  const alreadyImported =
    importMutation.isError &&
    importMutation.error instanceof ApiClientError &&
    importMutation.error.code === 'ITEM_ALREADY_IMPORTED';

  function handleImport() {
    importMutation.mutate({
      media_type: mediaType,
      external_id: result.external_id,
      title: result.title,
      provider: result.provider,
      raw_data: result.raw_data,
    });
  }

  return (
    <div className="flex h-full flex-col overflow-hidden rounded-app border border-border-soft bg-bg-surface transition-[border-color,transform] hover:-translate-y-0.5 hover:border-accent">
      <div
        className="relative flex aspect-[2/3] items-end justify-end bg-[linear-gradient(160deg,#33304a,#232323)] p-1.5"
        style={coverImageUrl ? { backgroundImage: `url(${coverImageUrl})`, backgroundSize: 'cover' } : undefined}
      >
        <span className="absolute top-1.5 left-1.5 rounded bg-black/55 px-1.5 py-0.5 font-mono text-[10px] tracking-wide text-text-muted">
          {mediaTypeLabels[mediaType]}
        </span>
      </div>
      <div className="flex flex-1 flex-col p-2.5 pt-2.5 pb-3">
        <p className="m-0 mb-1 line-clamp-2 font-display text-[13.5px] font-semibold text-text-primary">
          {result.title}
        </p>
        {originalTitle && originalTitle !== result.title && (
          <div className="mb-1.5 text-[11px] text-text-faint">{originalTitle}</div>
        )}
        <div className="flex items-center gap-1.5 text-[11px] text-text-faint">
          {releaseYear && <span>{releaseYear}年</span>}
          {releaseYear && <span>・</span>}
          <span>{externalProviderLabels[result.provider]}</span>
        </div>
        <button
          type="button"
          onClick={handleImport}
          disabled={alreadyImported || importMutation.isPending || importMutation.isSuccess}
          className="mt-auto w-full rounded-app border border-accent bg-accent px-3.5 py-1 text-sm font-medium text-white hover:bg-accent-strong disabled:cursor-default disabled:border-border disabled:bg-bg-surface disabled:text-text-faint disabled:opacity-50"
        >
          {alreadyImported
            ? '取り込み済み'
            : importMutation.isSuccess
              ? '取り込み済み'
              : importMutation.isPending
                ? '取り込み中…'
                : '取り込む'}
        </button>
      </div>
    </div>
  );
}
