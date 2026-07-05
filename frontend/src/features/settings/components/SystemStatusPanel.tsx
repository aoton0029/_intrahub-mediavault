import { useHealthQuery } from '@/features/settings/api';

export function SystemStatusPanel() {
  const health = useHealthQuery();

  return (
    <div>
      <h2 className="mb-1 font-display text-[17px]">システム状態</h2>
      <p className="mb-[18px] text-[12.5px] text-text-muted">アプリケーションの動作状況を確認します。</p>

      <div className="mb-2.5 flex items-center justify-between rounded-app border border-border-soft bg-bg-surface p-3.5 px-4">
        <div>
          <div className="text-[13.5px] font-semibold">データベース接続</div>
          <div className="mt-0.5 font-mono text-xs text-text-faint">GET /health</div>
        </div>
        {health.isLoading && <span className="text-xs text-text-faint">確認中…</span>}
        {health.isError && (
          <span className="rounded-full bg-danger/15 px-2 py-0.5 font-mono text-xs text-danger">
            status: error
          </span>
        )}
        {health.data && (
          <span className="rounded-full bg-status-done/15 px-2 py-0.5 font-mono text-xs text-status-done">
            status: {health.data.status}
          </span>
        )}
      </div>
    </div>
  );
}
