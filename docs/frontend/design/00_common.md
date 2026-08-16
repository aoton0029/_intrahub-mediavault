# 00. 共通設計（App Shell / トークン / 共通コンポーネント / インタラクション）

全19画面の設計書はこのドキュメントを前提とする。ここで定義したコンポーネント名・トークン名・状態管理方針を各画面ドキュメントから参照する。

参照元: `docs/frontend/ui/_shared.css`, `docs/frontend/ui/_shared.js`, `docs/frontend/ui/*.html`

---

## 1. アプリシェル構成

```
<AppShell>                     // .app-shell (grid: sidebar + main)
  <Sidebar>                    // .sidebar
    <Brand />                  // .brand (.dot + アプリ名)
    <NavSection label="...">   // .nav-section
      <NavItem active? count? indent? />  // .nav-item
    </NavSection>
    <ThemeToggle />            // .theme-toggle [data-theme-toggle]
  </Sidebar>
  <main className="main">
    <Titlebar>                 // .titlebar (sticky)
      <Breadcrumb />           // .breadcrumb
      <h1 />
      {actions}                 // 例: 「編集する」btn-accent
    </Titlebar>
    <div className="content">{children}</div>  // .content
  </main>
</AppShell>
```

- `Sidebar` のナビ項目・件数（`.count`）は各画面共通。ルーティングは `react-router-dom` v7 の `<Outlet>` を `AppShell` 内に置き、`AppShell` は共通レイアウトルートとして実装する想定。
- `ThemeToggle`: 後述 §4-1 参照。

---

## 2. Tailwind v4 `@theme` トークン対応表

Tailwind v4はCSS第一設定（`tailwind.config.js`不要）。`_shared.css` の `:root`（ダーク値）をデフォルトとして採用し、`[data-theme="light"]` をオーバーライドとして扱う（ダークをデフォルトにする方針で確定）。

新規グローバルCSS（例: `frontend/src/index.css`）に以下を定義する想定:

```css
@import "tailwindcss";

@theme {
  /* color */
  --color-bg-app: #1e1e1e;
  --color-bg-sidebar: #161616;
  --color-bg-surface: #262626;
  --color-bg-surface-hover: #2c2c2c;
  --color-bg-input: #1c1c1c;
  --color-border: #383838;
  --color-border-soft: #2e2e2e;
  --color-text-primary: #dcddde;
  --color-text-muted: #8a8a8d;
  --color-text-faint: #5c5c5f;
  --color-accent: #8b6cf6;
  --color-accent-strong: #a48bf8;
  --color-accent-soft: rgba(139, 108, 246, 0.15);
  --color-favorite: #e0a85a;
  --color-status-progress: #5aa9e0;
  --color-status-done: #5ac98a;
  --color-status-none: #6b6b6e;
  --color-danger: #e0615a;

  /* font */
  --font-ui: 'Inter', -apple-system, sans-serif;
  --font-display: 'Source Serif 4', Georgia, serif;
  --font-mono: 'JetBrains Mono', monospace;

  /* layout */
  --spacing-sidebar-w: 232px;
  --radius-app: 6px;
}

:root[data-theme="light"] {
  --color-bg-app: #f2f2f3;
  --color-bg-sidebar: #ebebec;
  --color-bg-surface: #ffffff;
  --color-bg-surface-hover: #f0f0f1;
  --color-bg-input: #ffffff;
  --color-border: #dcdcde;
  --color-border-soft: #e6e6e8;
  --color-text-primary: #202022;
  --color-text-muted: #6b6b6e;
  --color-text-faint: #98989b;
  --color-accent: #6d4fd6;
  --color-accent-strong: #5a3ec4;
  --color-accent-soft: rgba(109, 79, 214, 0.10);
  --color-favorite: #b97a2e;
  --color-status-progress: #2f7dc4;
  --color-status-done: #2f9d68;
  --color-status-none: #9a9a9d;
  --color-danger: #c3453f;
}
```

- `--color-*` は Tailwind の `bg-bg-app` / `text-text-primary` / `border-border-soft` 等のユーティリティとして自動生成される。
- `status-progress`/`status-done`/`status-none` は `StatusSwitcher` の色分けに使用（§3参照）。
- 【要確認】ライトモード切替がCSS変数のオーバーライドで実現されている（`prefers-color-scheme`ではなく`data-theme`属性による明示的トグル）ため、Tailwindの`dark:`バリアントは使わず、`[data-theme="light"]`セレクタベースのCSSに寄せる。

---

## 3. 共通コンポーネント一覧

| `_shared.css` クラス / モックパターン | 提案コンポーネント | 備考 |
|---|---|---|
| `.media-card` / `.is-compact` | `<MediaCard variant="default" \| "compact">` | `.cover`/`.badge`/`.fav`/`.title`/`.meta`/`.rating` を内包 |
| `.media-card.search-result` | `<MediaCard variant="search-result">` | 「取り込み済み」時 `.btn[disabled]` |
| `.card-grid` / `.is-compact` | `<MediaGrid density="default" \| "compact">` | `MediaCard[]` をgrid配置 |
| `.lit-list` / `.lit-row` | `<LiteratureList>` / `<LiteratureRow>` | 書誌情報中心の行リスト（論文・学術書） |
| `.filter-toolbar`（`.filter-bar` + `.sort-search-group`） | `<FilterToolbar>` | `.chip`/`.chip-add`/`.filter-select`/`.sort-select`/`.search-box` を内包 |
| `.load-more-sentinel` + `.spinner` | `<LoadMoreSentinel>` + `useInfiniteScroll()` | IntersectionObserverでページ追加読込 |
| `.status-switcher` / `.status-popover` / `[data-status-trigger]` | `<StatusSwitcher value onChange>` | ポップオーバー開閉はローカルstate |
| `.rating-stars` / `.star-btn` `[data-rating]` | `<RatingStars value onChange readOnly?>` | hover中のプレビュー状態を別管理 |
| `.rating-stars-mini` | `<RatingStarsMini value>` | カード内の非インタラクティブ表示 |
| `.favorite-toggle` `[data-favorite-toggle]` | `<FavoriteToggle value onChange>` | |
| タグ/カテゴリ pill追加削除（`.tag-pill`, `[data-tag-add]`, `[data-remove-tag]` 等） | `<TagList kind="tag" \| "category" items onAdd onRemove>` | インライン入力 → Enter確定/Escape取消/blur取消 |
| `.mylist-cover.n1〜n4` | `<MylistCover count={1-4} covers={string[]}>` | 収録作品数に応じたコラージュ |
| `.modal-overlay` / `.modal` | `<Modal open onClose title>` | マイリスト作成・削除確認等の子要素を内包 |
| `.detail-layout` | `<DetailLayout rail={...} main={...}>` | grid: `.detail-rail` + `.detail-main` |
| `.detail-rail`（`.doc-cover`/`.doc-title`/`.rail-facts`/`.rail-section`×n） | `<DetailRail>` | §6参照 |
| `.doc-section` | `<DetailSection icon title>` | 概要・関連作品など汎用セクション枠 |
| `.prop-row` / `.prop-group` | `<PropertyList items={{key,label,value}[]}>` | 種別固有情報の key-value 表示 |
| `.group-block` / `.group-header` / `.episode-row` | `<GroupList groups>` / `<EpisodeRow>` | シーズン・巻構成（anime/drama/manga/novel） |
| スタッフセクション `.prop-list-item` | `<StaffList members>` | anime/movie/drama のみ |
| 配信セクション `.prop-list-item` | `<StreamingLinks links>` | anime/movie/drama のみ |
| `.resource-tabs`（CSSラジオタブ） | `<ResourceTabs tabs={links\|files\|trailers}>` | React側はタブ選択をstateで管理 |
| 関連作品 `.result-row` | `<RelatedWorksList items>` | 全詳細画面共通 |
| `.form-section-title` / `.form-grid` / `.form-field` / `.form-actions` | `<FormSection>` / `<FormGrid>` / `<FormField>` / `<FormActions>` | react-hook-form + zod と組み合わせ |
| `.settings-shell` / `.settings-tabs` / `.settings-tab` | `<SettingsShell tabs>` | CSSラジオタブ→タブstate |
| `.kv-card` | `<ApiKeyCard provider keyMasked onEdit>` | 設定画面のAPIキー行 |
| `.empty-state` | `<EmptyState title description action?>` | 検索結果0件・APIキー未設定など |

---

## 4. アイコン（react-icons）

全画面のインラインSVGアイコンは **`react-icons/fi`**（Feather Icons）で実装する。モックHTML内の各SVGには実装対象のコンポーネント名がコメントで明記されているため（例: `<!-- react-icons/fi: FiCheckCircle -->`）、実装時は該当コメントの直後のSVGをそのコンポーネントに置き換える。独自SVG・他アイコンセットの追加は行わない。

代表例（`_shared.css` の `.icon` クラス相当、`width/height: 16px` を基本とする）:

| 用途 | react-icons/fi コンポーネント |
|---|---|
| ステータス: 未着手 | `FiCircle` |
| ステータス: 進行中 | `FiPlayCircle` |
| ステータス: 完了 | `FiCheckCircle` |
| ステータス切替の展開シェブロン | `FiChevronDown` |
| 評価スター | `FiStar`（塗り分けは `fill`/`stroke` の切替、`.is-full` 相当をpropで制御） |
| お気に入り | `FiHeart` |
| カレンダー（登録日等） | `FiCalendar` |
| 外部API連携（登録方法） | `FiLink` |
| 手動登録 | `FiEdit3` |
| タグ | `FiTag` |
| カテゴリ | `FiFolder` |
| マイリスト | `FiBookmark` |
| 概要セクション | `FiFileText` |
| 種別固有情報（movie） | `FiFilm` |
| 種別固有情報（drama） | `FiTv` |
| 種別固有情報（manga/novel/academic_book） | `FiBookOpen` |
| 種別固有情報（game） | `FiMonitor` |
| シーズン/巻構成 | `FiLayers` |
| スタッフ | `FiUsers` |
| 関連作品 | `FiGitBranch` |
| 配信 | `FiTv` |
| リソース（リンク/ファイル/トレーラー） | `FiPaperclip`（セクション見出し）、`FiLink2`（リンクタブ）、`FiPaperclip`（ファイルタブ）、`FiFilm`（トレーラータブ） |
| 追加ボタン | `FiPlus` |
| 削除・解除ボタン | `FiTrash2` |
| 解除（マイリスト/関連作品） | `FiCornerUpLeft` 相当のアイコン(モックSVGは箱アイコンのため実装時に `FiPackage`/`FiX` 等、モックのpath形状に近いものを選定。要確認) |
| 並び替え | `FiArrowUpDown 相当（モックはカスタムpath。react-iconsに厳密一致が無い場合は最も近いソート系アイコンを選定）` |
| 検索 | `FiSearch` |

`package.json` には `react-icons` ^5.7.0（および `lucide-react`）が既に導入済み（`00_common.md` 作成時点の調査結果）。本設計では `lucide-react` は使用せず、モックのコメント指定に厳密に従い `react-icons/fi` に統一する。

---

## 5. インタラクション → React状態への変換方針

`_shared.js` は `document.addEventListener` によるイベント委譲でDOM操作を行っているが、Reactではコンポーネントローカルstate + イベントハンドラに置き換える。

| `_shared.js` の挙動 | React化方針 |
|---|---|
| 5-1. テーマ切替（`[data-theme-toggle]` クリックで `localStorage['mediavault-theme']` 読み書き + `document.documentElement.setAttribute('data-theme', ...)`） | `useTheme()` フック（`useState` 初期値を `localStorage` から読込、`useEffect` で `<html data-theme>` に反映）。`ThemeToggle` はこのフックの `toggle()` を呼ぶのみ |
| 5-2. お気に入りトグル（`.is-active` クラス toggle） | `FavoriteToggle` の `value`/`onChange` をpropsで受け取る制御コンポーネント。実際の永続化はAPI呼び出し（`PATCH /items/{id}`）を呼ぶ親側で行う |
| 5-3. ステータス切替ポップオーバー（`hidden` 属性トグル、外側クリックで閉じる） | `StatusSwitcher` 内で `useState<boolean>` によるopen管理 + `useEffect` でdocumentへの外側クリックリスナー登録（コンポーネントスコープ） |
| 5-4. 評価スター（`mouseover`/`mouseout` でプレビュー、`click` で確定） | `RatingStars` 内で `hoverValue` と確定値 `value` を分離管理。`mouseleave` でプレビュー解除 |
| 5-5. タグ/カテゴリ追加（動的input生成、Enter確定/Escape取消/blur取消） | `TagList` 内で `isAdding` state + 制御 `<input>`。`onKeyDown` でEnter/Escapeをハンドリング、`onBlur` で遅延キャンセル（元実装の `setTimeout(cancel, 100)` はEnter確定とのクリック競合回避のため、React版でも同等の遅延またはクリック対象判定を検討） |
| 5-6. タグ/カテゴリ削除（`.tag-pill.remove()`） | `TagList` の `onRemove(id)` コールバック、実データは親のstate/APIキャッシュから除去 |
| 5-7. リソースタブ・設定タブ（CSSのみ `input[type=radio]:checked ~ ...`） | React版は素直に `useState<TabKey>` によるタブ切替に置き換える（CSS-onlyトリックは不要） |

---

## 6. 詳細画面共通パターン（`DetailLayout`）

全8詳細画面（16, 17, 18, 20〜24）が準拠する正準構成。各画面ドキュメントはこのセクションと差分のみを記述する。

### レール（`DetailRail`、左カラム・sticky）

1. `.doc-cover`（表紙）
2. `.doc-title`（h1） + `.doc-original`（原題、任意）
3. `.rail-facts`
   - `StatusSwitcher`（`not_started` / `in_progress` / `done`、色は `--color-status-*`）
   - `RatingStars`
   - `FavoriteToggle`
   - 登録日等 `.meta-item`
   - 外部API ID等 `.meta-item.muted`
4. `.rail-divider`
5. `RailSection` ×3: タグ（`TagList kind="tag"`） / カテゴリ（`TagList kind="category"`） / マイリスト（所属リスト + 解除ボタン + 追加リンク）

### メイン（`DetailMain`、右カラム、`.doc-section` の並び）

正準順序:

1. **概要** — 全画面共通
2. **種別固有情報**（`PropertyList`） — anime以外の7画面で存在。フィールドは画面ごとに異なる（各画面ドキュメントに記載）
3. **エピソード/巻構成**（`GroupList`） — anime, drama, manga, novel のみ
4. **スタッフ**（`StaffList`） — anime, movie, drama のみ
5. **関連作品**（`RelatedWorksList`） — 全画面共通
6. **配信**（`StreamingLinks`） — anime, movie, drama のみ（映像系メディアのみ）
7. **リソース**（`ResourceTabs`: リンク/ファイル/トレーラー） — 全画面共通。academic_book/paperでは「出版社ページ」等ラベルが変わる

### 画面別の任意セクション有無マトリクス

| 画面 | 種別固有情報 | 構成(Group) | スタッフ | 配信 |
|---|---|---|---|---|
| anime (16) | ✗ | ✓ | ✓ | ✓ |
| movie (20) | ✓(6項目) | ✗ | ✓ | ✓ |
| drama (21) | ✓(7項目) | ✓ | ✓ | ✓ |
| manga (22) | ✓(4項目) | ✓ | ✗ | ✗ |
| novel (23) | ✓(4項目) | ✓ | ✗ | ✗ |
| game (24) | ✓(5項目) | ✗ | ✗ | ✗ |
| academic_book (17) | ✓(4項目) | ✗ | ✗ | ✗ |
| paper (18) | ✓(5項目) | ✗ | ✗ | ✗ |

各画面固有のフィールド一覧・API推測は個別ドキュメントに記載する。

---

## 7. API連携についての注記

`docs/frontend/PRD.md` の「バックエンドAPI」節は現状未記載。各画面ドキュメントの「API連携」章に記載するエンドポイントは、PRDの機能一覧（共通機能・メディア別機能）からの推測であり、実装時にバックエンド（`backend/`）側の実際のAPI仕様と突き合わせて確定させる必要がある。推測箇所には `【要確認】(PRDのバックエンドAPIセクションは未記載のため推測)` と明記する。

正式なAPI仕様（リクエスト/レスポンス、エラーコード等）は `docs/backend/mediavault-api/`（ハブは [index.md](../backend/mediavault-api/index.md)）に存在する。各画面ドキュメントの「API連携」節末尾には対応する `mediavault-api/*.md` への `参照:` リンクを付与しており、実装時はそこを辿って表示項目・取得可能情報を確認し、`【要確認】` を解消する運用とする。
