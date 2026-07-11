import { useEffect, useState } from "react";
import { FiActivity, FiCheckCircle, FiDatabase, FiDownload, FiKey, FiUpload, FiXCircle } from "react-icons/fi";
import { ApiKeyCard } from "@/components/shared/ApiKeyCard";
import { FormActions, FormField, FormSection } from "@/components/shared/Forms";
import { SettingsShell } from "@/components/shared/SettingsShell";
import { useSettingsData, type HealthStatus, type ImportSummary } from "@/hooks/useSettingsData";

type ProviderKey = "tmdb" | "igdb" | "ndl" | "steam" | "annict" | "rakuten" | "jikan";

type ProviderEntry = {
  provider: ProviderKey;
  displayName: string;
  requiresKey: boolean;
  keyMasked: string;
};

const providers: ProviderEntry[] = [
  { provider: "tmdb", displayName: "TMDB", requiresKey: true, keyMasked: "provider: tmdb ・ 未設定" },
  { provider: "igdb", displayName: "IGDB", requiresKey: true, keyMasked: "provider: igdb ・ 未設定" },
  { provider: "ndl", displayName: "NDL(国立国会図書館)", requiresKey: true, keyMasked: "provider: ndl ・ 未設定" },
  { provider: "steam", displayName: "Steam", requiresKey: true, keyMasked: "provider: steam ・ 未設定" },
  { provider: "annict", displayName: "Annict", requiresKey: true, keyMasked: "provider: annict ・ 未設定" },
  { provider: "rakuten", displayName: "楽天", requiresKey: true, keyMasked: "provider: rakuten ・ 未設定" },
  { provider: "jikan", displayName: "Jikan(MyAnimeList)", requiresKey: false, keyMasked: "provider: jikan ・ APIキー不要(認証なしで利用可能)" },
];

function ApiKeysPanel() {
  const { saveApiKey } = useSettingsData();
  const [savingProvider, setSavingProvider] = useState<ProviderKey | null>(null);
  const [messages, setMessages] = useState<Partial<Record<ProviderKey, { type: "success" | "error"; text: string }>>>({});

  async function handleSave(provider: Exclude<ProviderKey, "jikan">, value: string) {
    if (!value.trim()) {
      setMessages((current) => ({
        ...current,
        [provider]: { type: "error", text: "APIキーを入力してください" },
      }));
      return;
    }

    setSavingProvider(provider);
    try {
      await saveApiKey(provider, value);
      setMessages((current) => ({
        ...current,
        [provider]: { type: "success", text: "APIキーを保存しました" },
      }));
      window.setTimeout(() => {
        setMessages((current) => {
          if (current[provider]?.type !== "success") {
            return current;
          }

          const next = { ...current };
          delete next[provider];
          return next;
        });
      }, 2500);
    } catch (error) {
      setMessages((current) => ({
        ...current,
        [provider]: {
          type: "error",
          text: error instanceof Error ? error.message : "APIキーの保存に失敗しました",
        },
      }));
    } finally {
      setSavingProvider(null);
    }
  }

  return (
    <div className="panel-api">
      <h2>
        <FiKey className="icon" />
        API連携
      </h2>
      <p className="desc">外部データソースのAPIキーを登録します。各プロバイダごとに個別のキーを保存できます。</p>

      {providers.map((entry) => (
        <div key={entry.provider}>
          <ApiKeyCard
            provider={entry.displayName}
            keyMasked={entry.keyMasked}
            variant="inline-save"
            requiresKey={entry.requiresKey}
            saving={savingProvider === entry.provider}
            onSave={entry.requiresKey ? (value) => void handleSave(entry.provider as Exclude<ProviderKey, "jikan">, value) : undefined}
          />
          {messages[entry.provider] ? (
            <p
              className={messages[entry.provider]?.type === "error" ? "field-error" : "field-hint"}
              style={{ marginTop: -2, marginBottom: 10, paddingLeft: 4 }}
              role="status"
            >
              {messages[entry.provider]?.text}
            </p>
          ) : null}
        </div>
      ))}
    </div>
  );
}

function ImportResultList({ result }: { result: ImportSummary }) {
  return (
    <>
      <div className="form-section-title">直近のインポート結果</div>
      <div className="kv-card" style={{ alignItems: "flex-start", flexDirection: "column", gap: 8 }}>
        <div className="meta-bar" style={{ marginBottom: 0, paddingBottom: 0, borderBottom: "none" }}>
          <span className="meta-item">
            <FiCheckCircle className="icon" />
            成功: {result.successCount}件
          </span>
          <span className="meta-item">
            <FiXCircle className="icon" />
            失敗: {result.failureCount}件
          </span>
        </div>
        {result.failures.map((failure) => (
          <div key={`${failure.row}-${failure.reason}`} className="prop-list-item">
            <span className="label">{failure.row}行目</span>
            <span className="sub">reason: {failure.reason}</span>
          </div>
        ))}
      </div>
    </>
  );
}

function ImportPanel() {
  const { importBooklog, importSteam } = useSettingsData();
  const [booklogFile, setBooklogFile] = useState<File | null>(null);
  const [steamId, setSteamId] = useState("");
  const [importResult, setImportResult] = useState<ImportSummary | null>(null);
  const [booklogError, setBooklogError] = useState<string | undefined>();
  const [steamError, setSteamError] = useState<string | undefined>();
  const [isImportingBooklog, setIsImportingBooklog] = useState(false);
  const [isImportingSteam, setIsImportingSteam] = useState(false);

  async function handleBooklogImport() {
    if (!booklogFile) {
      setBooklogError("CSVファイルを選択してください");
      return;
    }

    setBooklogError(undefined);
    setIsImportingBooklog(true);
    try {
      setImportResult(await importBooklog(booklogFile));
    } catch (error) {
      setBooklogError(error instanceof Error ? error.message : "Booklogの取り込みに失敗しました");
    } finally {
      setIsImportingBooklog(false);
    }
  }

  async function handleSteamImport() {
    if (!steamId.trim()) {
      setSteamError("Steam IDを入力してください");
      return;
    }

    setSteamError(undefined);
    setIsImportingSteam(true);
    try {
      setImportResult(await importSteam(steamId));
    } catch (error) {
      setSteamError(error instanceof Error ? error.message : "Steamライブラリの取り込みに失敗しました");
    } finally {
      setIsImportingSteam(false);
    }
  }

  return (
    <div className="panel-import">
      <h2>
        <FiDownload className="icon" />
        データインポート
      </h2>
      <p className="desc">外部サービスのデータを一括で取り込みます。</p>

      <FormSection title="Booklogからインポート">
        <div className="kv-card" style={{ alignItems: "flex-start", flexDirection: "column", gap: 12 }}>
          <FormField label="CSVファイル" hint="Booklogからエクスポートした読書記録CSVを選択してください" error={booklogError} full>
            <input
              aria-label="CSVファイル"
              type="file"
              accept=".csv"
              onChange={(event) => setBooklogFile(event.target.files?.[0] ?? null)}
            />
          </FormField>
          <FormActions>
            <button type="button" className="btn btn-accent btn-sm" disabled={isImportingBooklog} onClick={() => void handleBooklogImport()}>
              <FiUpload className="icon" />
              {isImportingBooklog ? "取り込み中..." : "アップロードして取り込む"}
            </button>
          </FormActions>
        </div>
      </FormSection>

      <FormSection title="Steamからインポート">
        <div className="kv-card" style={{ alignItems: "flex-start", flexDirection: "column", gap: 12 }}>
          <FormField label="Steam ID" error={steamError}>
            <input
              aria-label="Steam ID"
              type="text"
              placeholder="例: 76561198000000000"
              value={steamId}
              onChange={(event) => setSteamId(event.target.value)}
            />
          </FormField>
          <FormActions>
            <button type="button" className="btn btn-accent btn-sm" disabled={isImportingSteam} onClick={() => void handleSteamImport()}>
              <FiDownload className="icon" />
              {isImportingSteam ? "取り込み中..." : "ライブラリを取り込む"}
            </button>
          </FormActions>
        </div>
      </FormSection>

      {importResult ? <ImportResultList result={importResult} /> : null}
    </div>
  );
}

function SystemStatusPanel() {
  const { fetchHealth } = useSettingsData();
  const [healthStatus, setHealthStatus] = useState<HealthStatus | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    void fetchHealth()
      .then((result) => {
        if (!cancelled) {
          setHealthStatus(result);
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setHealthStatus({ database: "error" });
          setErrorMessage(error instanceof Error ? error.message : "システム状態の取得に失敗しました");
        }
      });

    return () => {
      cancelled = true;
    };
  }, [fetchHealth]);

  const databaseStatus = healthStatus?.database ?? "error";
  const statusColor = databaseStatus === "ok" ? "var(--color-status-done)" : "var(--color-danger)";

  return (
    <div className="panel-system">
      <h2>
        <FiActivity className="icon" />
        システム状態
      </h2>
      <p className="desc">アプリケーションの動作状況を確認します。</p>

      <div className="kv-card">
        <div>
          <div className="provider">データベース接続</div>
          <div className="key">GET /health</div>
        </div>
        <span className="tag-pill" style={{ color: statusColor }}>
          <FiDatabase className="icon" />
          status: {databaseStatus}
        </span>
      </div>
      {errorMessage ? <p className="field-error">{errorMessage}</p> : null}
    </div>
  );
}

export function SettingsPage() {
  return (
    <SettingsShell
      tabs={[
        {
          key: "api",
          label: "API連携",
          content: <ApiKeysPanel />,
        },
        {
          key: "import",
          label: "データインポート",
          content: <ImportPanel />,
        },
        {
          key: "system",
          label: "システム状態",
          content: <SystemStatusPanel />,
        },
      ]}
    />
  );
}
