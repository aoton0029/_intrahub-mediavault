# 初期調査: HomePage表示がデザインカンプ(01_home.html)と大きくかけ離れている

## バグの基本情報

- 概要: `docs/frontend/ui/01_home.html`（デザインカンプ）と `http://localhost` で実際にレンダリングされる `frontend/src/pages/HomePage.tsx` 等の見た目が大きく乖離している。
- 種類: UI/表示バグ
- 発生状況: 常に発生（`http://localhost` を開くと必ず発生）
- エラー情報: なし（コンソールエラーの報告なし。見た目のみの乖離）
- 再現手順: ブラウザで `http://localhost` を開くだけ
- 出力ファイル: `.dcs/20260703161847_home_page_ui_mismatch/initial_investigation.md`

## 関連コンポーネント・ファイル

- `docs/frontend/ui/01_home.html`（デザインカンプ本体）
- `docs/frontend/ui/_shared.css`（デザインカンプの正解CSS。`.sidebar`, `.nav-item`, `.media-card` 等の全クラス定義を持つ）
- `frontend/src/index.css`（実装側のグローバルCSS。Tailwind + shadcn + 一部`_shared.css`移植クラスのみを含む）
- `frontend/src/pages/HomePage.tsx`
- `frontend/src/pages/RootLayout.tsx`
- `frontend/src/components/common/Sidebar.tsx`
- `frontend/src/components/common/MediaCard.tsx`
- `frontend/src/components/common/FilterBar.tsx`
- `frontend/src/App.tsx`
- `docs/tasks/frontend-ui-compliance/overview.md`
- `docs/tasks/frontend-ui-compliance/TASK-0011.md`
- `docs/implements/frontend-ui-compliance/TASK-0011/*`

## 検索キーワードと結果

- `.sidebar|.brand|.nav-item|.nav-section|.count|.media-card|.cover|.badge|.fav|.status-dot|.body|.title` を `frontend/src/index.css` に対して grep
  - ヒットしたのは無関係な `.counter`（15行目付近）、`.titlebar` 系（TASK-0010実装分）、`@media (max-width:980px)` 内の `.sidebar` セレクタ（非表示指定のみ）の3件のみ。
  - **`.sidebar` 本体、`.brand`、`.nav-item`、`.nav-section`、`.nav-section-label`、`.count`、`.media-card`、`.cover`、`.badge`、`.fav`、`.body`、`.title`、`.meta` の実スタイル定義が index.css に一切存在しない。**
- `docs/frontend/ui/_shared.css` には上記すべてのクラスの完全なスタイル定義（背景色・レイアウト・フォント等）が存在する（75〜293行目）。
- `docs/tasks/frontend-ui-compliance/overview.md` では TASK-0007（Sidebar拡張）・TASK-0008（MediaCard拡張）が完了（`[x]`）と記録されているが、実際にはコンポーネント側（Sidebar.tsx/MediaCard.tsx）のマークアップ・クラス名付与のみが行われ、対応する index.css へのスタイル移植が漏れている。

## コードベース構造の理解

- ルーティングは `frontend/src/App.tsx` → `react-router-dom` の `RouterProvider` + `routes.ts`（未読だが `RootLayout` 配下に `HomePage` がマウントされる構成、`RootLayout.tsx` のコメントよりTASK-0009で `.app-shell` グリッド構造が確定済み）。
- `RootLayout.tsx` は `.app-shell` + `Sidebar` + `<main className="main"><Outlet /></main>` を描画しており、`.app-shell` / `.main` / `.app-shell.has-properties` は index.css に実装済み（456行目以降）。
- `HomePage.tsx` は `.titlebar` / `FilterBar`（`.filter-bar` クラス使用）/ `.card-grid` / `MediaCard` を描画しており、`.titlebar`・`.filter-bar`・`.chip`・`.search-box`・`.card-grid` は index.css に実装済み（TASK-0006, TASK-0010分）。
- 一方で `Sidebar.tsx`（`.sidebar`, `.brand`, `.dot`, `.nav-section`, `.nav-section-label`, `.nav-item`, `.indent`, `.count` を使用）と `MediaCard.tsx`（`.media-card`（Tailwindユーティリティで代替）, `.cover`, `.badge`, `.fav`, `.body`（実装では`<div className="flex flex-col gap-1 p-2">`に置換）, `.title`, `.meta`）に対応する固有CSSルールが index.css に存在しない。
- 結果として、モックアップでは暗色背景・固定幅232pxの縦ナビ、ホバー時のハイライト、ブランドロゴのドット、件数バッジなどが表示されるべきサイドバーが、実装では**ブラウザデフォルトスタイルに近い無装飾のテキストリスト**として表示される可能性が高い（`.app-shell` のグリッド列幅自体は確保されるため領域は232pxだが、中身の視覚表現がほぼ皆無になる）。
- MediaCardはTailwindユーティリティクラスで一部視覚表現（角丸・ボーダー・ホバー時浮き上がり等）を代替実装しているためSidebarほど致命的ではないが、`.badge`のフォント（JetBrains Mono相当の見た目）や`.title`の2行クランプ、`.meta`のレイアウトなど、モック固有の細部スタイルは反映されない。

## 初期仮説

### 候補1: `Sidebar.tsx` が使用するクラス（`.sidebar`, `.brand`, `.nav-item`, `.nav-section`, `.count` 等）に対応するCSSルールが `frontend/src/index.css` に一切定義されていない ⭐⭐⭐

- **証拠のファイル:行番号**:
  - `frontend/src/components/common/Sidebar.tsx:68-93`（`nav`要素に`className="sidebar"`、`.brand`, `.dot`, `.nav-section`, `.nav-section-label`, `.nav-item`, `.indent`, `.count` を使用）
  - `frontend/src/index.css`（全348行を確認したが、`.sidebar`本体・`.brand`・`.nav-item`・`.nav-section`・`.nav-section-label`・`.count`のスタイル定義が存在しない。544行目付近の `@media (max-width: 980px) { .sidebar, .properties { display: none; } }` のみが `.sidebar` に言及）
  - `docs/frontend/ui/_shared.css:75-131`（モックアップ側の正解定義：`.sidebar`に`background: var(--bg-sidebar)`、`padding`、`.nav-item`に`padding`・`border-radius`・hover/active色等の完全な視覚仕様がある）
- **バグ症状との関連**: サイドバーはページ左側常時表示される主要領域であり、モックでは暗色背景・アイコン風ブランド・階層インデント・ホバーハイライトを持つ「デザイン性の高いナビ」だが、実装では対応CSSが皆無のためブラウザデフォルト（下線なしリンクや素のdiv/nav）に近い見た目になり、"デザインカンプと大きくかけ離れている"という報告内容と直接一致する。
- **検証内容**: ブラウザで `http://localhost` を開き、DevToolsで`.sidebar`要素に適用されているComputed Styleを確認し、`background-color`が`var(--bg-sidebar)`（#161616）になっていないこと、`.nav-item`にpadding/border-radius/hover色が適用されていないことを確認する。
- **除外できない理由**: 実際に`npm run build`ないし`npm run dev`を起動してブラウザで目視確認するステップが本調査では未実施のため、他のグローバルCSS（Tailwindのデフォルトリセット等）による偶発的な見た目の一致・相違までは断定できない。

### 候補2: `MediaCard.tsx` がモック固有クラス（`.cover`, `.badge`, `.fav`, `.body`, `.title`, `.meta`）を使用しているが、index.cssに対応スタイルがなく、Tailwindユーティリティクラスによる代替実装で完全には再現されていない ⭐⭐

- **証拠のファイル:行番号**:
  - `frontend/src/components/common/MediaCard.tsx:40-81`（`cover`, `badge`, `fav`, `title`, `status-dot`等のクラス名は付与するが、実際の見た目はTailwindユーティリティ（`aspect-[2/3]`, `rounded-lg`, `border`等）で代替。モックの`.body`（10px 10px 12pxのpadding）や`.title`（Source Serif 4フォント・2行クランプ）、`.meta`（ステータスドット＋テキストの横並び）に相当するクラス・構造がコンポーネント内に存在しない）
  - `frontend/src/index.css`（`.card-grid`のみ定義され、`.media-card .cover/.badge/.fav/.body/.title/.meta`の定義なし）
  - `docs/frontend/ui/_shared.css:223-293`（モック側の完全なカード内装飾定義）
- **バグ症状との関連**: カードグリッドはHomePageの主要コンテンツであり、フォント（Source Serif 4によるタイトル）、バッジのモノスペースフォント、メタ情報のステータスドット表示など、モックの「らしさ」を構成する細部が実装では再現されずTailwindの汎用的な見た目になっている可能性がある。
- **検証内容**: ブラウザで一覧画面のカードを表示し、タイトルのフォントファミリーがSource Serif 4になっているか、バッジがモノスペースフォント・黒半透明背景になっているかをDevToolsで確認する。
- **除外できない理由**: Tailwindユーティリティによる代替実装がどの程度視覚的近似性を保てているかは実機確認なしでは判断できず、候補1ほど致命的な乖離ではない可能性がある。

### 候補3（除外検討のみ・記載保留）: ルーティングmiss／古いビルドキャッシュ／別レイアウト使用の可能性

- `frontend/src/App.tsx` は `RouterProvider` 経由で `routes.ts`（未確認）を参照しており、`RootLayout.tsx` のコメント（TASK-0009関連）から `HomePage` が `RootLayout` 配下にマウントされる構成であることは確認できたが、`routes.ts` 自体は未読のため、ルーティング誤設定の可能性を完全には排除できていない。ただし本調査時点でコード上の具体的な誤りの証拠は見つかっていないため、確度⭐⭐未満として記載を保留する（次段階調査で `frontend/src/routes.ts`（または相当ファイル）を確認する必要あり）。

## 次段階の調査方針

1. `frontend/src/routes.ts`（または相当のルーティング定義ファイル）を読み、`/` に `HomePage` が正しくマウントされているか、`RootLayout` が正しく親要素として使われているかを確認する。
2. `npm run dev` 相当でローカルサーバーを起動し、実際に `http://localhost` をブラウザで開いて `.sidebar` / `.media-card` 系のComputed Styleを確認し、候補1・候補2の仮説を実機で検証する。
3. `docs/tasks/frontend-ui-compliance/TASK-0007.md`（Sidebar拡張）・`TASK-0008.md`（MediaCard拡張）・対応する `docs/implements/frontend-ui-compliance/TASK-0007/`・`TASK-0008/` 配下のgreen-phase/refactor-phase文書を確認し、CSS移植作業がタスク完了条件に含まれていたか、含まれていたのに実施漏れなのか、そもそもタスクスコープ外だったのかを特定する。
4. `frontend/src/components/common/Sidebar.test.tsx`・`MediaCard.test.tsx` の既存テストを確認し、テストがクラス名の付与のみを検証していてスタイル内容（CSS定義の存在）まではテストされていないためにこの欠落が検出されずにいた可能性を確認する。
5. `frontend/src/index.css` のTASK-0001〜0002由来のコメント（🔵🟡マーカー）と `docs/tasks/frontend-ui-compliance/TASK-0001.md`・`TASK-0002.md` を突き合わせ、`_shared.css` の全クラスを移植する方針だったか、一部クラスのみ移植する方針だったかを設計文書（`architecture.md`）で確認する。

## 制限事項

- 実際にローカルサーバーを起動してブラウザで目視確認するステップは本調査では実施していない（静的なコード比較のみ）。
- `frontend/src/routes.ts`（ルーティング定義）、`TASK-0007.md`/`TASK-0008.md`本文、対応する`docs/implements/`配下のフェーズ文書は未確認であり、候補1・候補2が「実装漏れ」なのか「意図的な設計変更」なのかの最終判断は次段階調査が必要。
- Tailwindのユーティリティクラスがどこまでモックの見た目を代替できているかは実機のレンダリング結果でしか判断できないため、候補2の深刻度は推測の域を出ない。
- `vite.config.ts` / `package.json` のビルド設定、開発サーバーのポート・プロキシ設定は本調査では確認しておらず、候補3（ビルドキャッシュ等）を完全には排除できていない。
