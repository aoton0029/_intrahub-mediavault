import { useRef, useState } from 'react';
import { useImportBooklogMutation, useImportSteamMutation } from '@/features/settings/api';
import { ApiClientError } from '@/lib/api-client';
import type { ImportSummary } from '@/features/settings/types';

function ImportSummaryCard({ summary }: { summary: ImportSummary }) {
  return (
    <div className="mb-2.5 flex flex-col gap-2 rounded-app border border-border-soft bg-bg-surface p-3.5 px-4">
      <div className="flex gap-[18px] text-[12.5px] text-text-muted">
        <span>成功: {summary.success_count}件</span>
        <span>失敗: {summary.failure_count}件</span>
      </div>
      {summary.failures.map((failure, i) => (
        <div
          key={i}
          className="flex items-center justify-between border-b border-border-soft py-1.5 text-[12.5px] last:border-b-0"
        >
          <span className="text-text-primary">{failure.row_number}行目</span>
          <span className="font-mono text-xs text-text-faint">reason: {failure.reason}</span>
        </div>
      ))}
    </div>
  );
}

export function ImportPanel() {
  const [file, setFile] = useState<File | null>(null);
  const [steamId, setSteamId] = useState('');
  const fileInputRef = useRef<HTMLInputElement>(null);

  const booklogMutation = useImportBooklogMutation();
  const steamMutation = useImportSteamMutation();

  const handleBooklogUpload = () => {
    if (!file) return;
    booklogMutation.mutate(file, {
      onSuccess: () => {
        setFile(null);
        if (fileInputRef.current) fileInputRef.current.value = '';
      },
    });
  };

  const handleSteamImport = () => {
    if (!steamId) return;
    steamMutation.mutate(steamId);
  };

  const latestSummary = steamMutation.data ?? booklogMutation.data;
  const latestError = steamMutation.isError
    ? steamMutation.error
    : booklogMutation.isError
      ? booklogMutation.error
      : null;

  return (
    <div>
      <h2 className="mb-1 font-display text-[17px]">データインポート</h2>
      <p className="mb-[18px] text-[12.5px] text-text-muted">外部サービスのデータを一括で取り込みます。</p>

      <div className="mb-3 mt-0 border-b border-border-soft pb-1.5 text-xs uppercase tracking-wide text-text-faint">
        Booklogからインポート
      </div>
      <div className="mb-2.5 flex flex-col items-start gap-3 rounded-app border border-border-soft bg-bg-surface p-3.5 px-4">
        <div className="flex max-w-[420px] flex-col gap-1.5">
          <label className="text-xs text-text-muted">CSVファイル</label>
          <input
            ref={fileInputRef}
            type="file"
            accept=".csv"
            onChange={(e) => setFile(e.target.files?.[0] ?? null)}
            className="text-[13px] text-text-primary"
          />
          <span className="text-xs text-text-faint">
            Booklogからエクスポートした読書記録CSVを選択してください
          </span>
        </div>
        <button
          type="button"
          onClick={handleBooklogUpload}
          disabled={!file || booklogMutation.isPending}
          className="rounded-app border border-accent bg-accent px-3 py-1 text-xs font-medium text-white hover:bg-accent-strong disabled:opacity-50"
        >
          {booklogMutation.isPending ? '取り込み中…' : 'アップロードして取り込む'}
        </button>
      </div>

      <div className="mb-3 mt-7 border-b border-border-soft pb-1.5 text-xs uppercase tracking-wide text-text-faint">
        Steamからインポート
      </div>
      <div className="mb-2.5 flex flex-col items-start gap-3 rounded-app border border-border-soft bg-bg-surface p-3.5 px-4">
        <div className="flex max-w-[320px] flex-col gap-1.5">
          <label className="text-xs text-text-muted">Steam ID</label>
          <input
            type="text"
            placeholder="例: 76561198000000000"
            value={steamId}
            onChange={(e) => setSteamId(e.target.value)}
            className="rounded-app border border-border bg-bg-input px-2.5 py-2 text-[13px] text-text-primary outline-none focus:border-accent"
          />
        </div>
        <button
          type="button"
          onClick={handleSteamImport}
          disabled={!steamId || steamMutation.isPending}
          className="rounded-app border border-accent bg-accent px-3 py-1 text-xs font-medium text-white hover:bg-accent-strong disabled:opacity-50"
        >
          {steamMutation.isPending ? '取り込み中…' : 'ライブラリを取り込む'}
        </button>
      </div>

      <div className="mb-3 mt-7 border-b border-border-soft pb-1.5 text-xs uppercase tracking-wide text-text-faint">
        直近のインポート結果
      </div>
      {latestError && (
        <div className="mb-2.5 rounded-app border border-danger bg-bg-surface p-3.5 px-4 font-mono text-xs text-danger">
          {latestError instanceof ApiClientError ? latestError.message : '取り込みに失敗しました'}
        </div>
      )}
      {!latestError && latestSummary && <ImportSummaryCard summary={latestSummary} />}
      {!latestError && !latestSummary && (
        <div className="mb-2.5 rounded-app border border-border-soft bg-bg-surface p-3.5 px-4 text-[12.5px] text-text-faint">
          まだインポートは実行されていません。
        </div>
      )}
    </div>
  );
}
