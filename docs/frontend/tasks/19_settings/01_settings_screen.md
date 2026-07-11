# 01. 設定画面（SettingsPage）

対応: 設計書 §1〜§7

## 前提ファイル

- 参照: `docs/frontend/design/19_settings.md`, `docs/frontend/ui/19_settings.html`, `docs/frontend/ui/_shared.css`, `frontend/src/index.css`, `docs/backend/mediavault-api/settings.md`, `docs/backend/mediavault-api/import.md`, `docs/backend/mediavault-api/health.md`
- 参照（既存実装、直接import対象）: `frontend/src/components/shared/SettingsShell.tsx`, `frontend/src/components/shared/ApiKeyCard.tsx`, `frontend/src/components/shared/Forms.tsx`（`FormSection`/`FormGrid`/`FormField`/`FormActions`）, `frontend/src/components/layout/AppShell.tsx`, `frontend/src/routes.tsx`
- 出力: `frontend/src/pages/SettingsPage.tsx`, `frontend/src/hooks/useSettingsData.ts`（API呼び出しhook）, `frontend/src/components/shared/ApiKeyCard.tsx`（拡張）, `frontend/src/routes.tsx`（`/settings`のプレースホルダを`SettingsPage`に差し替え）
- 参照範囲はこの節に列挙したファイルとそこから直接importされるファイルに限る。それ以外のファイルを探すための横断的な探索は行わない。

## タスク一覧

- [x] UIに表示するアイテム（ラベル・見出し・ボタン文言・メッセージ等）は日本語を優先して使用する
- [x] アイコンは`react-icons`を積極的に使用する（見出し・ボタン・ステータス表示・空状態等、視覚的な手がかりが有効な箇所には極力アイコンを添える。`ApiKeyCard`の既存`FiLink`利用パターンを踏襲する）
- [x] `ApiKeyCard`（`frontend/src/components/shared/ApiKeyCard.tsx`）を拡張し、パスワード入力欄+保存ボタンをインライン表示するモードを追加する。既存の`provider`/`keyMasked`/`onEdit`propsは変更せず、新規に`variant: "edit-link" | "inline-save"`（デフォルト`"edit-link"`で後方互換維持）、`inline-save`時は`onSave: (value: string) => void`、`saving?: boolean`を受け取れるようにする。`jikan`のように入力欄自体を出さない行は`requiresKey: false`を渡すと`.field-hint`の「設定不要」ラベルのみを表示する
- [x] `frontend/src/hooks/useSettingsData.ts` を実装する。以下を提供する:
  - `saveApiKey(provider: string, apiKey: string): Promise<ApiCredential>` — `PUT /settings/api-keys/{provider}`（`docs/backend/mediavault-api/settings.md`）を呼ぶ。エラー時は`INVALID_PROVIDER`(400)を含むエラーメッセージをそのまま呼び出し元に伝える
  - `importBooklog(file: File): Promise<ImportSummary>` — `POST /import/booklog`（multipart、フィールド名`file`）
  - `importSteam(steamId: string): Promise<ImportSummary>` — `POST /import/steam`（`docs/backend/mediavault-api/import.md`のエラー`VALIDATION_ERROR`/`STEAM_API_KEY_INVALID`/`EXTERNAL_API_TIMEOUT`をエラーメッセージとして呼び出し元に伝える）
  - `fetchHealth(): Promise<HealthStatus>` — `GET /health`
  - `ImportSummary`型は`{ successCount: number; failureCount: number; failures: { row: number; reason: string }[] }`とし、バックエンドレスポンスの`success_count`/`failure_count`/`failures[].row_number`をこの型にマッピングする
- [x] `frontend/src/pages/SettingsPage.tsx` を実装する。`SettingsShell`の`tabs`に以下3件を渡す:
  - `api`（label: "API連携"）— `ApiKeysPanel`。プロバイダ一覧は`tmdb`("TMDB"), `igdb`("IGDB"), `ndl`("NDL(国立国会図書館)"), `steam`("Steam"), `annict`("Annict"), `rakuten`("楽天"), `jikan`("Jikan(MyAnimeList)"、`requiresKey: false`)の固定7行とする（`GET /settings/api-keys`一覧取得APIが存在しないため、既存キーの値・最終更新日時は取得できない前提で空欄入力+保存ボタンのみとする）。各行は`ApiKeyCard`の`inline-save`モードを使い、保存ボタン押下で`saveApiKey(provider, value)`を呼び、成功時はトースト等の簡易フィードバック（例: 一時的な成功メッセージ表示）を出す。失敗時はエラーメッセージを表示する
  - `import`（label: "データインポート"）— `ImportPanel`。「Booklogからインポート」セクション（`<input type="file" accept=".csv">` + 「アップロードして取り込む」ボタン、`importBooklog`呼び出し）と「Steamからインポート」セクション（Steam ID入力 + 「ライブラリを取り込む」ボタン、`importSteam`呼び出し）を持つ。両フォームとも`FormField`/`FormActions`を利用する
  - `system`（label: "システム状態"）— `SystemStatusPanel`。マウント時に`fetchHealth`を呼び、`kv-card`内に「データベース接続」ラベルと`.tag-pill`（`status: ok`時は`--color-status-done`、それ以外はエラー系の色）でステータスを表示する
- [x] `ImportPanel`内に`ImportResultList`（直近のインポート結果表示）を実装する。`importBooklog`/`importSteam`のいずれかが成功したら、その結果（`ImportSummary`）を画面state（`useState`）に保持し、「直近のインポート結果」セクションに`meta-bar`（成功/失敗件数）+ `prop-list-item[]`（`{row}行目` / `reason: {reason}`）で表示する。結果が無い初期状態ではこのセクション自体を表示しない
- [x] `frontend/src/routes.tsx` の `path: "settings"` の`element`を現在のプレースホルダ`<div>設定画面のプレースホルダ</div>`から`<SettingsPage />`に差し替える（`handle: { title: "設定" }`は維持）
- [x] `frontend/src/index.css`: `_shared.css`に対応クラスが無い場合のみ追加してよい。既存クラスの値は変更しない

## テストリスト

- [x] `SettingsPage.test.tsx`: 初期表示で3タブ（API連携/データインポート/システム状態）が`SettingsShell`経由で描画され、初期タブがAPI連携であること
- [x] `SettingsPage.test.tsx`: API連携タブに7プロバイダ行が表示され、`jikan`行のみ入力欄が無く「設定不要」ラベルのみであること
- [x] `SettingsPage.test.tsx`: いずれかのプロバイダ行でパスワード入力→保存ボタンクリックで`saveApiKey`が該当providerと入力値で呼ばれること（`useSettingsData`をモック）
- [x] `SettingsPage.test.tsx`: データインポートタブでCSVファイル選択→「アップロードして取り込む」クリックで`importBooklog`が呼ばれ、成功時に「直近のインポート結果」セクションが件数・失敗理由一覧とともに表示されること
- [x] `SettingsPage.test.tsx`: データインポートタブでSteam ID入力→「ライブラリを取り込む」クリックで`importSteam`が呼ばれること
- [x] `SettingsPage.test.tsx`: システム状態タブでマウント時に`fetchHealth`が呼ばれ、`status: ok`時に`.tag-pill`が完了系の色で表示されること
- [x] `ApiKeyCard.test.tsx`（既存テストファイルへの追記可）: `variant="inline-save"`かつ`requiresKey=true`の場合にpassword inputと保存ボタンが描画され、クリックで`onSave`が入力値付きで呼ばれること。`requiresKey=false`の場合は入力欄が描画されず「設定不要」の`field-hint`のみが表示されること
- [x] `tests/e2e/settings.spec.ts`: `yarn dev`起動下で`SettingsPage`を実描画し、`docs/frontend/ui/19_settings.html`と主要構造（`.settings-tabs`のタブ3件、`.settings-panel`内の初期パネルがAPI連携パネルであること、各`.kv-card`の並び、`.tag-pill`の存在）が一致することをDOM構造アサーション（`getByRole`/`locator`）で確認する

> Codexメモ: APIキー一覧取得APIは未実装前提のため、各プロバイダは固定行 + 「未設定」表示で実装。
> 成功フィードバックはトーストではなく行ごとの一時メッセージで実装し、既存CSSの範囲に収めた。
