# バグ処理フロー分析

**実施日時**: 2026-07-03
**分析者**: Claude Code

[← インデックスに戻る](./index.md) | [初期調査](./initial_investigation.md)

---

## 分析対象

### バグの概要
`docs/frontend/ui/01_home.html`（デザインカンプ）と `http://localhost` で実際にレンダリングされる HomePage の見た目が大きく乖離している。

### 分析の焦点
初期調査で特定された「`Sidebar.tsx` が使用するクラス（`.sidebar`/`.brand`/`.nav-item`/`.nav-section`/`.count` 等）に対応するCSSルールが `frontend/src/index.css` に一切定義されていない」という仮説を、実際のレンダリングフロー（エントリーポイント→ルーティング→コンポーネント→CSS適用）に沿って検証する。

---

## エントリーポイント

### frontend/src/App.tsx:1-15

**役割**: `http://localhost` へのアクセス時にマウントされるReactアプリのルート。`RouterProvider` に `./routes` の `router` を渡す。

**コードスニペット**:
```tsx
import { RouterProvider } from 'react-router-dom';
import { router } from './routes';
export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
      <Toaster richColors position="top-right" />
    </QueryClientProvider>
  );
}
```

**トリガー**: ブラウザで `http://localhost` を開くと、Viteのエントリー（`main.tsx`相当）経由で `App` がマウントされ、`createBrowserRouter` で解決されたルートがレンダリングされる。

---

## 処理フローの詳細

### ステップ1: ルーティング解決

**ファイル**: frontend/src/routes.tsx:16-40

**処理内容**:
`path: '/'` に `RootLayout` を割り当て、`index: true` の子ルートとして `HomePage` を配置。`/` へのアクセスは `RootLayout > HomePage` の構成で解決される。

**コードスニペット**:
```tsx
export const router = createBrowserRouter([
  {
    path: '/',
    element: <RootLayout />,
    children: [
      { index: true, element: <HomePage /> },
      ...
    ],
  },
]);
```

**注目点**: ルーティング設定自体に誤りはない。初期調査で保留していた「候補3: ルーティングmiss」は本ファイルの内容から**除外**できる（下記「初期調査候補の検証と除外」参照）。

---

### ステップ2: RootLayoutによる`.app-shell`構造の描画

**ファイル**: frontend/src/pages/RootLayout.tsx:16-27

**処理内容**:
`.app-shell` クラスのdivを描画し、内部に `<Sidebar />` と `<main className="main"><Outlet /></main>` を配置する。

**入力**: なし（`useLocation`でパスのみ参照）

**出力**: DOM構造 `<div class="app-shell"><nav class="sidebar">...</nav><main class="main"><HomePage/></main></div>`

**コードスニペット**:
```tsx
return (
  <div className={cn('app-shell', isItemDetail && 'has-properties')}>
    <Sidebar />
    <main className="main">
      <Outlet />
    </main>
    ...
```

**注目点**: `.app-shell` / `.main` は `frontend/src/index.css:520-531` に実装済みで、グリッド列幅（`--sidebar-w` 232px）は正しく確保される。問題は子要素の `Sidebar` 側にある。

---

### ステップ3: Sidebarコンポーネントのレンダリング

**ファイル**: frontend/src/components/common/Sidebar.tsx:66-93

**処理内容**:
`<nav className="sidebar">` を起点に、`.brand`（ブランドロゴ）、`.nav-section`/`.nav-section-label`（「ライブラリ」見出し）、複数の `.nav-item`（`NavItemLink`経由）、`.count`（バッジ、本データでは未使用）をDOMに出力する。

**出力（DOM）**:
```html
<nav class="sidebar" aria-label="グローバルナビゲーション">
  <div class="brand"><span class="dot"></span>MediaVault</div>
  <div class="nav-section">
    <div class="nav-section-label">ライブラリ</div>
    <a class="nav-item nav-link active" href="/">...</a>
    <a class="nav-item nav-link indent" href="/collections/general">...</a>
    ...
  </div>
  ...
</nav>
```

**注目点**: ここで出力される `sidebar`/`brand`/`dot`/`nav-section`/`nav-section-label`/`nav-item`/`nav-link`/`indent`/`count` はすべて**素のクラス名**（Tailwindユーティリティではない）であり、CSSファイル側に対応する `@layer`/通常セレクタとしての定義が存在しなければブラウザにスタイルが一切適用されない。

---

### ステップ4: グローバルCSSの読み込みとクラス解決

**ファイル**: frontend/src/index.css（Viteのエントリーで読み込まれる唯一のグローバルCSS。1-9行目で `@import "tailwindcss"` 等を読込）

**処理内容**:
ブラウザは `sidebar`/`brand`/`nav-item`等のクラスセレクタに対応するCSSルールをこのファイル（およびTailwindが生成するユーティリティ）から探索する。

**Grep結果（再確認）**:
- `.sidebar` 本体定義: **存在しない**。唯一の言及は544-547行目の `@media (max-width: 980px) { .sidebar, .properties { display: none; } }` のみで、非表示指定にすぎない。
- `.brand`, `.nav-item`, `.nav-section`, `.nav-section-label`, `.count`, `.dot`, `.indent`: **一切存在しない**（348-548行目の全カスタムクラス定義を確認したが該当なし）。
- 一方 `.titlebar`（372-389行目）、`.breadcrumb`（391-401行目）、`.btn`/`.btn-danger`（403-417行目）、`.doc*`（419-454行目）、`.card-grid`（456-461行目）、`.filter-bar`/`.chip`/`.search-box`（463-516行目）、`.app-shell`/`.main`/`.properties`（518-537行目）は**すべて実装済み**。

**注目点**: TASK-0005（EmptyState）・TASK-0004（TagPill）・TASK-0009（AppShell）・TASK-0010（CardGrid/Titlebar）・TASK-0011（Breadcrumb/Btn/Doc）に対応するCSSブロックにはそれぞれ「【○○スタイル】: _shared.css .xxx相当（TASK-00NN REQ-xxx）」というコメントが付与され、`_shared.css` からの移植が明示的にコミットされている。**TASK-0007（Sidebar拡張）・TASK-0008（MediaCard拡張）に対応するCSSブロックのコメントが index.css 中に一件も存在しない** — これは他タスクでは必ず行われている「CSS移植＋コメント記録」の作業がTASK-0007/0008でのみ欠落していることを示す一貫したパターンである。

---

### ステップ5: Tailwindによるクラス解釈の検証

**ファイル**: frontend/tailwind.config.*（Tailwind v4構成のため、`index.css` 内の `@import "tailwindcss"` と `@theme inline`（238-299行目）がユーティリティ生成の設定箇所）

**処理内容**:
Tailwind v4は `@theme` で定義されたトークンから `bg-*`/`text-*`等のユーティリティクラスを生成するが、`sidebar`/`brand`/`nav-item` のような素の（プレフィックスなしの）クラス名はTailwindの命名規則（`bg-`, `text-`, `flex` 等の既知パターン）に一致しないため、Tailwindのユーティリティとしては一切生成・解釈されない。

**注目点**: 例外的に `--color-sidebar`（259-266行目）という**shadcn由来の別概念のCSS変数**が定義されているが、これは `bg-sidebar` のような合成ユーティリティ生成用のトークンであり、`class="sidebar"`（プレフィックスなしの素のクラス名）には一切影響しない。つまり `--color-sidebar` の存在は `.sidebar` クラスのスタイル欠落を救済しない。この点は初期調査では未検証だった。

---

### ステップ6: ブラウザでの最終レンダリング

**処理内容**:
`.sidebar`/`.brand`/`.nav-item`等に一致するCSSルールがCSSOM上に存在しないため、ブラウザは該当要素にUser Agent Stylesheetのデフォルト値（`nav`/`div`/`a`要素の初期値）のみを適用する。結果として、暗色背景・パディング・ホバー時ハイライト・件数バッジ等の視覚仕様（`docs/frontend/ui/_shared.css:75-131`）が一切反映されず、無装飾のリンクリストとして表示される。

**注目点**: `.app-shell`（親のグリッド）自体は機能するため領域幅（232px）は確保されるが、その中身が「デザインカンプと大きくかけ離れる」原因になる。

---

## データ/スタイルクラスの対応関係追跡

| コンポーネント側クラス名 | 使用箇所（frontend/src） | `_shared.css`での定義 | `index.css`での定義 | 結果 |
|---|---|---|---|---|
| `sidebar` | Sidebar.tsx:68 | `_shared.css:76-83` | **なし**（544行目は`display:none`の`@media`のみ） | スタイル未適用 |
| `brand` / `.brand .dot` | Sidebar.tsx:70-71 | `_shared.css:85-102` | **なし** | スタイル未適用 |
| `nav-section` / `nav-section-label` | Sidebar.tsx:76-77 | `_shared.css:104-114` | **なし** | スタイル未適用 |
| `nav-item` / `.indent` / `.count` / `.active` | Sidebar.tsx:46,52 (NavItemLink) | `_shared.css:116-131` | **なし** | スタイル未適用（ホバー/アクティブ色も欠落） |
| `titlebar` | HomePage.tsx:41 | `_shared.css:139-156` | index.css:372-389（実装済み） | 正常 |
| `card-grid` | HomePage.tsx:64 | 対応定義あり | index.css:456-461（実装済み） | 正常 |
| `filter-bar`/`chip`/`search-box` | FilterBar（未読だが初期調査で確認済み） | 対応定義あり | index.css:463-516（実装済み） | 正常 |
| `media-card`/`cover`/`badge`/`fav`/`status-dot` | MediaCard.tsx:36,41,54,62,77 | `_shared.css`223行目以降に定義あり（初期調査より） | index.css内に対応クラス定義なし。ただしTailwindユーティリティ（`rounded-lg`,`border`,`aspect-[2/3]`等）で外形は代替済み | 部分的に代替。フォント（Source Serif 4のタイトル・JetBrains Monoのバッジ）等の細部は再現されない |

---

## 状態変化の追跡

本バグはデータ状態やAPIレスポンスに起因するものではなく、静的なCSS定義の欠落によるものであるため、状態遷移表は該当なし（`isLoading`/`isError`分岐等はスタイル欠落と無関係に機能する）。

---

## 条件分岐の分析

### 分岐: RootLayoutの`isItemDetail`判定（RootLayout.tsx:7-10, 19, 24）

この分岐は `.properties` 列の表示/非表示を制御するのみで、`.sidebar`のスタイル欠落とは無関係。ホームページ（`/`）表示時は `isItemDetail=false` となり `.has-properties` は付与されないが、これはバグの症状（Sidebarの見た目乖離）には影響しない。

---

## 異常系の処理

該当なし。本バグはエラーハンドリングやバリデーションの不備ではなく、静的CSS定義の欠落によるものであり、コンソールエラーも発生しない（初期調査記載の通り）。

---

## 非同期処理の分析

該当なし。CSSの読み込みはビルド時に静的にバンドルされるため、非同期タイミングの問題ではない。

---

## 初期調査候補の検証と除外

| 候補 | 検証結果 | 判定 | 理由 |
|------|---------|------|------|
| 候補1: Sidebarの`.sidebar`/`.brand`/`.nav-item`等に対応するCSSが`index.css`に存在しない | ✅ | 継続 | ステップ4で `index.css` 全行を再確認し、該当クラスの定義が一切存在しないことを確定。他タスク（TASK-0004,0005,0009,0010,0011）は移植コメント付きで実装されているのに対し、TASK-0007由来のコメントが皆無という一貫した欠落パターンも確認 |
| 候補2: MediaCardのモック固有クラス（`.cover`/`.badge`/`.fav`等）に対応スタイルがなく、Tailwindでの代替が不完全 | ✅ | 継続 | ステップ4・データ対応表で確認。ただしTailwindユーティリティで角丸・ボーダー・ホバー等の外形は代替されており、候補1ほど致命的ではない |
| 候補3: ルーティングmiss／古いビルドキャッシュ／別レイアウト使用の可能性 | ❌ | 除外 | `frontend/src/routes.tsx:16-40` を確認した結果、`/` → `RootLayout` → `index: true: HomePage` という設定に誤りはなく、`App.tsx` も `router` を正しく `RouterProvider` に渡している。ルーティング起因の証拠は見つからなかった |

### 除外した候補とその理由

1. **候補3（ルーティング誤設定）**: `routes.tsx`・`App.tsx` を実際に読み込み、パス設定・コンポーネント割り当てに誤りがないことを確認したため除外する。

---

## フロー分析からの所見

### 発見された問題点

#### 問題点1: `index.css` に `.sidebar`/`.brand`/`.nav-item`/`.nav-section`/`.count` 系の実スタイルが一切定義されていない
- **深刻度**: 高
- **位置**: frontend/src/index.css:1-548（該当クラス定義が存在しない）、対比: docs/frontend/ui/_shared.css:75-131
- **詳細**: Sidebar.tsx が出力するクラス名はTailwindユーティリティとして解釈されない素のクラス名であり、対応するCSSルールがなければブラウザのデフォルトスタイルしか適用されない。他タスク（TASK-0004/0005/0009/0010/0011）ではすべて「_shared.css移植＋信頼性コメント」付きでCSSが追加されているのに、TASK-0007（Sidebar拡張）分だけこのパターンが完全に欠落している。
- **バグとの関連**: サイドバーはページ左側に常時表示される主要領域であり、この欠落により「暗色背景・階層インデント・ホバーハイライト・ブランドロゴ」等デザインカンプの主要な視覚要素が実装で再現されず、"デザインカンプと大きくかけ離れている"という報告症状と直接一致する。
- **確度**: ⭐⭐⭐⭐⭐

#### 問題点2: `Sidebar.test.tsx` はクラス名の付与（`className`に文字列が含まれるか）のみを検証し、対応するCSSルールの存在やComputed Styleは一切検証していない
- **深刻度**: 中
- **位置**: frontend/src/components/common/Sidebar.test.tsx:26-213（全12テストケースが`toBeInTheDocument`/`toHaveAttribute`/`className`の文字列包含のみを検証）
- **詳細**: TC-09〜TC-12はTASK-0007の拡張要素（ブランドロゴ・セクション見出し・indentクラス・設定の配置）を検証しているが、いずれもDOM構造とクラス名文字列の存在確認にとどまり、jsdom環境でCSSファイルの中身やComputed Styleまでは検証していない（vitestの標準的なコンポーネントテストの性質上、CSS未読み込みでもテストは成功する）。
- **バグとの関連**: このテスト設計のため、TASK-0007完了時に「クラス名は付与されているがCSSルール本体が存在しない」という状態でも全テストがグリーンになり、CI/レビューでは検出されずに `overview.md` 上で「完了」としてマークされた可能性が高い。
- **確度**: ⭐⭐⭐⭐

#### 問題点3: `index.css` 内の `--color-sidebar` 等のshadcn由来トークン（238-299行目）は `.sidebar` クラスのスタイル欠落を救済しない
- **深刻度**: 低（誤解を招く可能性がある点の指摘）
- **位置**: frontend/src/index.css:259-266（`--color-sidebar`, `--color-sidebar-accent`等）
- **詳細**: これらはTailwindの `bg-sidebar` のような合成ユーティリティ生成用のトークンであり、`class="sidebar"`（素のクラス名）には作用しない。コードレビュー時に「sidebar関連のCSS変数はある」という表面的な確認だけでは、この欠落が見過ごされやすい構造になっている。
- **確度**: ⭐⭐⭐

### 次段階で詳しく調べるべき箇所

1. **`docs/tasks/frontend-ui-compliance/TASK-0007.md`・`TASK-0008.md` の完了条件本文**
   - 位置: docs/tasks/frontend-ui-compliance/TASK-0007.md（本分析では本文未読、タイトルとoverview.mdのチェック状態のみ確認済み）
   - 理由: 完了条件に「index.cssへのCSS移植」が明記されていたか、コンポーネント側のクラス名付与のみで完了とされる設計だったのかを確定する必要がある
   - 着目点: green-phase/refactor-phase文書（`docs/implements/frontend-ui-compliance/TASK-0007/`）にCSS追加作業が含まれていたかどうか
   - 確認した証拠: index.css内の他タスク（TASK-0004/0005/0009/0010/0011）は必ず「_shared.css .xxx相当」コメント付きでCSSが追加されているという一貫パターンが、TASK-0007/0008にだけ存在しないことをステップ4で確認済み

2. **`frontend/src/components/common/MediaCard.test.tsx` の検証範囲**
   - 位置: frontend/src/components/common/MediaCard.tsx:32-83（`cover`/`badge`/`fav`/`status-dot`クラス使用箇所）
   - 理由: Sidebar.test.tsxと同様、クラス名存在確認のみでCSS実体の欠落を検出できていない可能性がある
   - 着目点: フォント（`--font-display`, `--font-mono`）が実際に適用されているかのテストの有無
   - 確認した証拠: index.css内に`.media-card .cover/.badge/.fav/.body/.title/.meta`相当の定義が存在しないことは初期調査で確認済み（本分析のデータ対応表でも再確認）

---

*この処理フロー分析結果に基づいて、次段階で詳細な原因分析を実施します。*
