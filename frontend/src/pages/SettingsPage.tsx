import { useRef, useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { toast } from 'sonner';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { useApiCredentialsQuery, useUpsertApiCredentialMutation } from '@/api/settings';
import { useImportBooklogMutation, useImportSteamMutation } from '@/api/import';
import type { ApiProvider, ImportSummary } from '@/types';

const PROVIDERS: { id: ApiProvider; label: string }[] = [
  { id: 'tmdb', label: 'TMDB' },
  { id: 'igdb', label: 'IGDB' },
  { id: 'ndl', label: 'NDL' },
  { id: 'steam', label: 'Steam' },
  { id: 'open_library', label: 'Open Library' },
  { id: 'ani_list', label: 'AniList' },
];

const apiKeySchema = z.object({ apiKey: z.string().min(1, 'APIキーを入力してください') });
type ApiKeyForm = z.infer<typeof apiKeySchema>;

function ProviderRow({ provider, maskedKey }: { provider: ApiProvider; label: string; maskedKey?: string }) {
  const { register, handleSubmit, reset, formState: { errors } } = useForm<ApiKeyForm>({
    resolver: zodResolver(apiKeySchema),
  });
  const mutation = useUpsertApiCredentialMutation();
  const [editing, setEditing] = useState(false);

  function onSubmit(values: ApiKeyForm) {
    mutation.mutate(
      { provider, apiKey: values.apiKey },
      {
        onSuccess: () => {
          toast.success(`${provider} のAPIキーを保存しました`);
          reset();
          setEditing(false);
        },
        onError: (err) => {
          toast.error(err.message);
        },
      }
    );
  }

  const inputId = `api-key-${provider}`;

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="grid grid-cols-[120px_1fr_auto] items-start gap-3 py-2">
      <Label htmlFor={inputId} className="pt-2">{PROVIDERS.find((p) => p.id === provider)?.label ?? provider}</Label>
      <div className="flex flex-col gap-1">
        {editing || !maskedKey ? (
          <Input
            id={inputId}
            {...register('apiKey')}
            type="text"
            placeholder="APIキーを入力"
            autoComplete="off"
            aria-describedby={errors.apiKey ? `${inputId}-error` : undefined}
          />
        ) : (
          <div
            className="flex h-9 items-center rounded-md border bg-muted px-3 text-sm text-muted-foreground cursor-pointer"
            onClick={() => setEditing(true)}
          >
            {maskedKey}
          </div>
        )}
        {errors.apiKey && <p id={`${inputId}-error`} className="text-xs text-destructive">{errors.apiKey.message}</p>}
      </div>
      <div className="flex gap-2 pt-0.5">
        {editing || !maskedKey ? (
          <>
            <Button type="submit" size="sm" disabled={mutation.isPending}>
              保存
            </Button>
            {maskedKey && (
              <Button type="button" size="sm" variant="ghost" onClick={() => { reset(); setEditing(false); }}>
                キャンセル
              </Button>
            )}
          </>
        ) : (
          <Button type="button" size="sm" variant="outline" onClick={() => setEditing(true)}>
            編集
          </Button>
        )}
      </div>
    </form>
  );
}

function ApiKeysTab() {
  const { data: credentials = [] } = useApiCredentialsQuery();

  return (
    <div className="space-y-1 divide-y">
      {PROVIDERS.map(({ id, label }) => {
        const cred = credentials.find((c) => c.provider === id);
        return (
          <ProviderRow key={id} provider={id} label={label} maskedKey={cred?.apiKeyMasked} />
        );
      })}
    </div>
  );
}

function ImportSummaryView({ summary }: { summary: ImportSummary }) {
  return (
    <div className="mt-4 space-y-3">
      <div className="flex gap-6 rounded-md border p-4">
        <div className="text-sm">成功: <span className="font-bold">{summary.successCount}件</span></div>
        <div className="text-sm">失敗: <span className="font-bold">{summary.failureCount}件</span></div>
      </div>
      {summary.failureCount > 0 && (
        <div className="rounded-md border border-destructive/30 p-3">
          <p className="mb-2 text-sm font-medium text-destructive">失敗一覧</p>
          <ul className="space-y-1">
            {summary.failures.map((f) => (
              <li key={f.rowNumber} className="text-xs text-muted-foreground">
                行 {f.rowNumber}: {f.reason}
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}

function ImportTab() {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [steamId, setSteamId] = useState('');

  const booklogMutation = useImportBooklogMutation();
  const steamMutation = useImportSteamMutation();

  function handleBooklogImport() {
    if (!selectedFile) return;
    booklogMutation.mutate(selectedFile, {
      onSuccess: (data) => {
        toast.success(`成功 ${data.successCount}件、失敗 ${data.failureCount}件`);
      },
      onError: (err) => {
        toast.error(err.message);
      },
    });
  }

  function handleSteamImport() {
    if (!steamId.trim()) return;
    steamMutation.mutate(steamId.trim(), {
      onSuccess: (data) => {
        toast.success(`成功 ${data.successCount}件、失敗 ${data.failureCount}件`);
        setSteamId('');
      },
      onError: (err) => {
        toast.error(err.message);
      },
    });
  }

  return (
    <div className="space-y-8">
      {/* ブクログCSVインポート */}
      <section className="space-y-3">
        <h2 className="text-base font-semibold">ブクログ CSVインポート</h2>
        <div className="space-y-2">
          <Label htmlFor="booklog-file">CSVファイル</Label>
          <input
            id="booklog-file"
            ref={fileInputRef}
            type="file"
            accept=".csv"
            className="block text-sm"
            onChange={(e) => setSelectedFile(e.target.files?.[0] ?? null)}
          />
        </div>
        <Button
          onClick={handleBooklogImport}
          disabled={!selectedFile || booklogMutation.isPending}
        >
          {booklogMutation.isPending ? (
            <span className="flex items-center gap-2">
              <svg className="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z" />
              </svg>
              インポート中…
            </span>
          ) : 'インポート実行'}
        </Button>
        {booklogMutation.data && <ImportSummaryView summary={booklogMutation.data} />}
      </section>

      {/* Steamライブラリインポート */}
      <section className="space-y-3">
        <h2 className="text-base font-semibold">Steamライブラリインポート</h2>
        <div className="space-y-2">
          <Label htmlFor="steam-id">Steam ID</Label>
          <Input
            id="steam-id"
            value={steamId}
            onChange={(e) => setSteamId(e.target.value)}
            placeholder="Steam IDを入力"
          />
        </div>
        <Button
          onClick={handleSteamImport}
          disabled={!steamId.trim() || steamMutation.isPending}
        >
          {steamMutation.isPending ? (
            <span className="flex items-center gap-2">
              <svg className="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z" />
              </svg>
              インポート中…
            </span>
          ) : 'インポート実行'}
        </Button>
        {steamMutation.data && <ImportSummaryView summary={steamMutation.data} />}
      </section>
    </div>
  );
}

function ExportTab() {
  return (
    <div className="space-y-6">
      <p className="text-sm text-muted-foreground">
        この機能は現在のバージョンでは未対応です。将来のバージョンで対応予定です。
      </p>
      <div className="space-y-4">
        <div className="space-y-2">
          <Button disabled>Obsidianへエクスポート</Button>
          <p className="text-xs text-muted-foreground">現在未対応</p>
        </div>
        <div className="space-y-2">
          <Button disabled>Notionへエクスポート</Button>
          <p className="text-xs text-muted-foreground">現在未対応</p>
        </div>
      </div>
    </div>
  );
}

export default function SettingsPage() {
  return (
    <div className="container mx-auto max-w-2xl py-8">
      <h1 className="mb-6 text-2xl font-bold">設定</h1>
      <Tabs defaultValue="api-keys">
        <TabsList className="mb-6">
          <TabsTrigger value="api-keys">APIキー</TabsTrigger>
          <TabsTrigger value="import">インポート</TabsTrigger>
          <TabsTrigger value="export">エクスポート</TabsTrigger>
        </TabsList>
        <TabsContent value="api-keys">
          <ApiKeysTab />
        </TabsContent>
        <TabsContent value="import">
          <ImportTab />
        </TabsContent>
        <TabsContent value="export">
          <ExportTab />
        </TabsContent>
      </Tabs>
    </div>
  );
}
