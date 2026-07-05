import { useState } from 'react';
import { useSetTitlebar } from '@/components/layout/useTitlebar';
import { ApiKeyPanel } from '@/features/settings/components/ApiKeyPanel';
import { ImportPanel } from '@/features/settings/components/ImportPanel';
import { SystemStatusPanel } from '@/features/settings/components/SystemStatusPanel';

type SettingsTab = 'api' | 'import' | 'system';

const TABS: { key: SettingsTab; label: string }[] = [
  { key: 'api', label: 'API連携' },
  { key: 'import', label: 'データインポート' },
  { key: 'system', label: 'システム状態' },
];

export function SettingsPage() {
  const [tab, setTab] = useState<SettingsTab>('api');

  useSetTitlebar({ title: '設定' });

  return (
    <div className="grid grid-cols-[200px_1fr] gap-0">
      <div className="flex flex-col gap-0.5 border-r border-border-soft pr-4">
        {TABS.map((t) => (
          <button
            key={t.key}
            type="button"
            onClick={() => setTab(t.key)}
            className={`mb-0.5 rounded-app px-2.5 py-2 text-left text-sm ${
              tab === t.key
                ? 'bg-accent-soft text-accent-strong'
                : 'text-text-muted hover:bg-bg-surface-hover hover:text-text-primary'
            }`}
          >
            {t.label}
          </button>
        ))}
      </div>

      <div className="max-w-none pl-7">
        {tab === 'api' && <ApiKeyPanel />}
        {tab === 'import' && <ImportPanel />}
        {tab === 'system' && <SystemStatusPanel />}
      </div>
    </div>
  );
}
