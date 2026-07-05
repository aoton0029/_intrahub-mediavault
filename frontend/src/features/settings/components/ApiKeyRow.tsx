import { useState } from 'react';
import { useUpdateApiKeyMutation } from '@/features/settings/api';
import { ApiClientError } from '@/lib/api-client';
import { apiProviderLabels, type ApiProvider } from '@/features/settings/types';

export function ApiKeyRow({ provider }: { provider: ApiProvider }) {
  const [apiKey, setApiKey] = useState('');
  const mutation = useUpdateApiKeyMutation();

  const handleSave = () => {
    if (!apiKey) return;
    mutation.mutate(
      { provider, apiKey },
      {
        onSuccess: () => setApiKey(''),
      },
    );
  };

  const meta = mutation.isSuccess
    ? `最終更新: ${new Date(mutation.data.updated_at).toLocaleString('ja-JP')}`
    : '未設定';

  return (
    <div className="mb-2.5 flex items-center justify-between rounded-app border border-border-soft bg-bg-surface p-3.5 px-4">
      <div>
        <div className="text-[13.5px] font-semibold">{apiProviderLabels[provider]}</div>
        <div className="mt-0.5 font-mono text-xs text-text-faint">
          provider: {provider} ・ {meta}
        </div>
        {mutation.isError && (
          <div className="mt-0.5 font-mono text-xs text-danger">
            {mutation.error instanceof ApiClientError ? mutation.error.message : '保存に失敗しました'}
          </div>
        )}
      </div>
      <div className="flex gap-2">
        <input
          type="password"
          placeholder="APIキーを入力"
          value={apiKey}
          onChange={(e) => setApiKey(e.target.value)}
          className="w-56 rounded-app border border-border bg-bg-input px-2.5 py-1.5 text-[13px] text-text-primary outline-none focus:border-accent"
        />
        <button
          type="button"
          onClick={handleSave}
          disabled={!apiKey || mutation.isPending}
          className="rounded-app border border-accent bg-accent px-3 py-1 text-xs font-medium text-white hover:bg-accent-strong disabled:opacity-50"
        >
          {mutation.isPending ? '保存中…' : '保存'}
        </button>
      </div>
    </div>
  );
}
