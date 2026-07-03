# MediaVault Frontend UIデザイン準拠PRD

## 概要
現行のフロントエンド実装(`frontend/`)を、`docs/frontend/ui/`に定義されたモックアップ（Obsidianライクな3ペイン・ダークUI）の見た目に準拠させるためのPRD。
`frontend/src/index.css`には「具体値は参照元CSS削除のため推測」との注記があり、モックアップのデザイントークンが実装に反映されていない状態にある。本PRDは、その乖離を解消しモックアップ通りの視覚デザインを実現することを目的とする。

対象範囲はデザイントークン・レイアウト構造・コンポーネントの視覚表現に限定する。機能要件（画面構成・API連携・データ項目等）は[docs/frontend/PRD.md](./PRD.md)を正とし、本PRDでは重複させない。

## 参照元
- 共通デザイントークン: [docs/frontend/ui/_shared.css](./ui/_shared.css)
- 全体一覧モックアップ: [docs/frontend/ui/01_home.html](./ui/01_home.html)
- アイテム詳細モックアップ: [docs/frontend/ui/02_item_detail.html](./ui/02_item_detail.html)
- 検索・追加モックアップ: [docs/frontend/ui/03_search_add.html](./ui/03_search_add.html)
- 手動追加・編集フォームモックアップ: [docs/frontend/ui/04_item_form.html](./ui/04_item_form.html)
- 設定モックアップ: [docs/frontend/ui/05_settings.html](./ui/05_settings.html)

## デザイントークン移行方針
`frontend/src/index.css`のトークンを、`_shared.css`の値に合わせて再定義する。

| 種別 | `_shared.css`（あるべき値） | 現状（`frontend/src/index.css`） |
|---|---|---|
| 背景色（アプリ全体） | `--bg-app: #1e1e1e` | `--bg-base: #0f1115` / shadcn既定の白ベース |
| 背景色（サイドバー） | `--bg-sidebar: #161616` | 対応トークンなし |
| 背景色（サーフェス） | `--bg-surface: #262626` | `--bg-surface: #1a1d23`（別値） |
| ボーダー | `--border: #383838` / `--border-soft: #2e2e2e` | `--border-default: #2d313a` |
| 文字色 | `--text-primary: #dcddde` / `--text-muted: #8a8a8d` / `--text-faint: #5c5c5f` | `--text-primary: #f5f5f5` / `--text-secondary: #9ca3af` |
| アクセント色 | `--accent: #8b6cf6`（単色・紫） | mediaType別8色（`--accent-anime`等）+ shadcn `oklch`系トークン |
| お気に入り色 | `--favorite: #e0a85a` | 対応トークンなし |
| ステータス色 | `--status-progress/done/none` | 対応トークンなし |
| フォント（UI） | Inter | Geist Variable |
| フォント（見出し/タイトル） | Source Serif 4 | なし（Geistで代用） |
| フォント（等幅） | JetBrains Mono | `ui-monospace, Consolas, monospace` |
| 角丸 | `--radius: 6px`固定 | `--radius: 0.625rem`ベースのshadcn段階スケール |
| サイドバー幅 | `--sidebar-w: 232px` | 固定値なし |
| プロパティパネル幅 | `--properties-w: 300px` | 未実装 |

**方針**: mediaType別アクセントカラー（`--accent-anime`等）は既存の実装機能として維持しつつ、UI全体の基調色（背景・文字・単一アクセント・フォント・角丸・レイアウト幅）を`_shared.css`の値に置き換える。shadcn由来の`oklch`系トークン（`--primary`, `--card`等）は`_shared.css`の対応する色に上書きする。

## 画面別要件

### 全体一覧 (`01_home.html` ⇄ `HomePage.tsx` / `Sidebar.tsx` / `MediaCard.tsx`)
モックアップにあり現状の実装に無い要素:
- サイドバーのブランドロゴ（ドットアイコン + "MediaVault"）
- 各ナビ項目の件数バッジ（例: 全体一覧128件）
- 一般メディア配下のサブカテゴリ・インデント階層（アニメ/映画/ドラマ/漫画/小説/ゲーム）
- 「ライブラリ」セクション見出しによるグルーピング（マイリスト/タグ・カテゴリ/スタッフ）
- 「設定」をサイドバー下部に固定配置するレイアウト
- 絵文字アイコン（📚🎬🎓📄⭐🏷️👤⚙️）
- タイトルバー右上の「＋ 作品を追加」ボタン
- フィルタバー（`.chip`によるステータス/タグの絞り込みチップ + 検索ボックス）
- MediaCardのカバー画像プレースホルダ（グラデーション背景）、media-typeバッジのオーバーレイ表示、★お気に入りアイコン、ステータスドット（進行中=青/完了=緑/未着手=グレー）の視覚表示

### アイテム詳細 (`02_item_detail.html` ⇄ `ItemDetailPage.tsx`)
モックアップにあり現状の実装に無い要素:
- 3ペインレイアウト（`app-shell has-properties`）— 右側にPropertiesパネルを追加した構成
- パンくずリスト（例: 「アニメ / 星屑のシンフォニア」）
- タイトルバー右の「編集」「削除」ボタン
- カバー画像（`.doc-cover`）、タイトル（`.doc-title`）、原題表示（`.doc-original`）
- 概要セクション（`.doc-section`）
- シーズン/話数のグループ表示（`.group-block` + `.group-header` + `.episode-row`）— アニメ・ドラマ向け
- リンク・ファイル・トレーラー一覧（`.prop-list-item`）
- **Properties パネル（シグネチャー要素、新規コンポーネント化が必要）**:
  - key-value行（media_type / status / consumed_date / rating / favorite / source / release_date / studio 等）
  - タグ一覧（`.tag-pill`によるハッシュタグ風表示）
  - カテゴリ一覧
  - 関連付け一覧（他アイテムへの参照）
  - スタッフ一覧（役割付き）
  - マイリスト所属一覧

### 検索・追加 (`03_search_add.html` ⇄ `SearchAddPage.tsx`)
モックアップにあり現状の実装に無い要素:
- 検索結果リスト（`.result-row`）— サムネイル（グラデーションプレースホルダ）+ タイトル + サブ情報（等幅フォント）のレイアウト

### 手動追加・編集フォーム (`04_item_form.html` ⇄ `ItemFormPage.tsx`)
モックアップにあり現状の実装に無い要素:
- 2カラムのフォームグリッド（`.form-grid` / `.form-field`、全幅項目は`.form-field.full`）
- 必須項目マーク（`.required`）
- 入力エラー時のボーダー色変更・エラーメッセージ表示（`.form-field.error` / `.field-error`、等幅フォント）
- フィールドヒント（`.field-hint`）
- フォーム下部の操作ボタン群（`.form-actions`、上部ボーダー区切り）

### 設定 (`05_settings.html` ⇄ `SettingsPage.tsx`)
モックアップにあり現状の実装に無い要素:
- 左タブ + 右パネルのレイアウト（`.settings-shell` = `.settings-tabs` + `.settings-panel`）
- タブ項目: APIキー管理 / インポート / エクスポート
- APIキー管理の`.kv-card`（プロバイダ名 + マスクされたキー表示 + 「更新」/「登録」ボタン。未設定時は`btn-accent`で強調）

## 共通コンポーネント要件
以下をモックアップの`_shared.css`定義に沿って実装（または新規コンポーネント化）する。

| コンポーネント | 対応するCSS定義 | 備考 |
|---|---|---|
| 3ペインApp Shell | `.app-shell`, `.app-shell.has-properties` | `RootLayout.tsx`をsidebar/main/propertiesのgridに変更。properties列は画面ごとに有無を切り替え |
| Properties パネル | `.properties`, `.prop-group`, `.prop-row`, `.prop-list-item`, `.prop-taglist` | 新規コンポーネント。詳細画面でのみ表示 |
| サイドバーナビ | `.sidebar`, `.brand`, `.nav-section`, `.nav-section-label`, `.nav-item`, `.nav-item.indent`, `.nav-item .count` | `Sidebar.tsx`を階層構造・件数バッジ対応に拡張 |
| MediaCard | `.media-card`, `.cover`, `.badge`, `.fav`, `.status-dot` | プレースホルダ画像・バッジ・お気に入り・ステータスドットを視覚表示に変更 |
| タグピル | `.tag-pill` | `#`プレフィックス付きのハッシュタグ風表示 |
| ボタン | `.btn`, `.btn-accent`, `.btn-ghost`, `.btn-sm`, `.btn-danger` | バリアント一式を実装 |
| フィルタバー | `.filter-bar`, `.chip`, `.search-box` | 一覧画面共通 |
| 設定タブ | `.settings-shell`, `.settings-tabs`, `.settings-tab`, `.settings-panel`, `.kv-card` | |
| 検索結果行 | `.result-row`, `.thumb`, `.info` | |
| フォームグリッド | `.form-grid`, `.form-field`, `.field-error`, `.field-hint`, `.form-actions` | |
| 空状態 | `.empty-state` | 一覧が0件の場合の表示 |
| タイトルバー/パンくず | `.titlebar`, `.breadcrumb` | 各画面共通ヘッダー |

## やらなくていいこと
- レスポンシブ対応は`_shared.css`の`@media (max-width: 980px)`定義（980px以下でsidebar/properties非表示）に準拠する範囲に留め、それ以上のモバイル最適化は対象外とする。
- ダークテーマ以外のテーマ切替（ライトモード等）は対象外。モックアップはダークテーマ固定を前提とする。
- モックアップに存在しない新規ビジュアル要素の追加（アニメーション拡張、独自装飾等）は対象外。
