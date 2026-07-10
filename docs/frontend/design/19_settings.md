# 19. 設定

対応モック: `docs/frontend/ui/19_settings.html`

## 1. 画面概要 / ルート

API連携キー管理・データインポート・システム状態確認をタブで切り替える画面。ルート: `/settings`（タブは `/settings?tab=api|import|system` のクエリパラメータ、または `/settings/api` 等のネストルートで管理）。サイドバー「設定」がactive。

## 2. レイアウト構成

```
<AppShell>
  <Titlebar title="設定" />
  <Content>
    <SettingsShell activeTab={tab} onTabChange={setTab}>
      <SettingsTabs>
        <SettingsTab id="api" label="API連携" />
        <SettingsTab id="import" label="データインポート" />
        <SettingsTab id="system" label="システム状態" />
      </SettingsTabs>

      <SettingsPanel>
        {tab === 'api' && <ApiKeysPanel providers={providers} />}
        {tab === 'import' && <ImportPanel />}
        {tab === 'system' && <SystemStatusPanel />}
      </SettingsPanel>
    </SettingsShell>
  </Content>
</AppShell>
```

CSSのみのラジオタブ（`_shared.css` の `#tab-api:checked ~ ...` パターン）はReact実装では不要。`useState<'api'|'import'|'system'>` によるタブ切替に置き換える（`00_common.md` §4-7参照）。

## 3. 表示データ / Props型

```ts
interface ApiKeyEntry {
  provider: 'tmdb' | 'igdb' | 'ndl' | 'steam' | 'open_library' | 'ani_list' | 'jikan';
  displayName: string;         // 例: "TMDB", "NDL(国立国会図書館)"
  lastUpdatedAt?: string;      // 未設定時は「未設定」表示
  requiresKey: boolean;        // jikanのみ false（認証不要のため入力欄自体を出さない）
}

interface ImportSummary {
  successCount: number;
  failureCount: number;
  failures: { row: number; reason: string }[];
}

interface HealthStatus {
  database: 'ok' | 'error';
}
```

## 4. 画面固有コンポーネント

- `ApiKeysPanel` / `ImportPanel` / `SystemStatusPanel`: タブごとのパネル。共通の `ApiKeyCard`（`00_common.md`）を利用
- `ImportResultList`: 直近のインポート結果表示（`meta-bar` + `prop-list-item[]`）

## 5. インタラクション仕様

- APIキー保存: プロバイダごとに `password` inputに入力し「保存」ボタンで個別送信（一括保存ではない）。jikanのみ入力欄なし、「設定不要」ラベルのみ
- CSV/Steamインポート: それぞれ独立したフォーム（ファイル選択 or Steam ID入力）+ 送信ボタン
- インポート結果は「直近のインポート結果」セクションに成功/失敗件数と失敗理由リストを表示

## 6. API連携

- APIキー保存: `PUT /settings/api-keys/{provider}`（モックHTMLコメントより高確度）
- APIキー一覧取得: 【要確認】`GET /settings/api-keys` はドキュメントに無く、既存キーの値・設定済みかどうかは取得できない前提。6プロバイダ固定の空欄入力+保存ボタンのみを先行実装し、バックエンド実装時に一覧取得APIの追加が必要（モックHTMLコメントに明記）
- Booklogインポート: `POST /import/booklog`（multipart、CSVファイル）
- Steamインポート: `POST /import/steam`（Steam ID）
- システム状態: `GET /health`

参照: [settings.md](../../backend/mediavault-api/settings.md), [import.md](../../backend/mediavault-api/import.md), [health.md](../../backend/mediavault-api/health.md)

## 7. Tailwindスタイリング上の注意

- `kv-card` はAPI連携・インポート・システム状態の3パネルすべてで再利用する共通の行カード
- ヘルスステータスの `tag-pill` 色は `ok` 時 `--color-status-done`
