import { apiProviders } from '@/features/settings/types';
import { ApiKeyRow } from './ApiKeyRow';

export function ApiKeyPanel() {
  return (
    <div>
      <h2 className="mb-1 font-display text-[17px]">API連携</h2>
      <p className="mb-[18px] text-[12.5px] text-text-muted">
        外部データソースのAPIキーを登録します。各プロバイダごとに個別のキーを保存できます。
      </p>

      {apiProviders.map((provider) => (
        <ApiKeyRow key={provider} provider={provider} />
      ))}

      <div className="mb-2.5 flex items-center justify-between rounded-app border border-border-soft bg-bg-surface p-3.5 px-4">
        <div>
          <div className="text-[13.5px] font-semibold">Jikan(MyAnimeList)</div>
          <div className="mt-0.5 font-mono text-xs text-text-faint">
            provider: jikan ・ APIキー不要(認証なしで利用可能)
          </div>
        </div>
        <span className="text-xs text-text-faint">設定不要</span>
      </div>
    </div>
  );
}
