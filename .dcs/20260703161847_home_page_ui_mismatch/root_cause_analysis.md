# 根本原因分析: HomePage表示がデザインカンプ(01_home.html)と大きくかけ離れている

**実施日時**: 2026-07-03

[← インデックスに戻る](./index.md) | [初期調査](./initial_investigation.md) | [フロー分析](./flow_analysis.md)

---

## 分析サマリー

`docs/frontend/ui/01_home.html` のデザインカンプに準拠して実装されたはずの `Sidebar.tsx` / `MediaCard.tsx` が出力するクラス名（`.sidebar`, `.brand`, `.nav-item`, `.nav-section`, `.count`, `.dot`, `.indent`, `.media-card .cover/.badge/.fav/.body/.title/.meta`, `.status-dot` 等）に対応する実CSSルールが `frontend/src/index.css` に一切存在しない。これはビルドエラーやテスト失敗を起こさない「静的CSS定義の欠落」であり、TASK-0007（Sidebar拡張）・TASK-0008（MediaCard拡張）が `overview.md` 上で完了マークされているにもかかわらず、他タスク（TASK-0004,0005,0009,0010,0011）で必ず行われている「`_shared.css` からのCSS移植＋信頼性コメント付与」という作業だけがこの2タスクで欠落したまま見過ごされたことが根本原因である。結果としてSidebarはブラウザのUser Agent Stylesheet任せの無装飾リストとして表示され、これが報告されている「デザインカンプと大きくかけ離れている」症状と一致する。

---

## 原因候補の評価

### 候補1: `.sidebar`/`.brand`/`.nav-item`/`.nav-section`/`.count`/`.dot`/`.indent` のCSSルールが `index.css` に一切存在しない ⭐⭐⭐⭐⭐

- **証拠**:
  - `frontend/src/components/common/Sidebar.tsx:68-93` — `nav`要素に `className="sidebar"`、`.brand`, `.dot`, `.nav-section`, `.nav-section-label`, `.nav-item`, `.indent`, `.count` を素のクラス名として出力
  - `frontend/src/index.css` 全548行を確認したが、該当クラスのセレクタ定義は存在しない。`.sidebar`への唯一の言及は539-547行目の `@media (max-width: 980px) { .sidebar, .properties { display: none; } }` のみ（非表示指定であり、通常時のスタイルではない）
  - `docs/frontend/ui/_shared.css:75-131` に `.sidebar`/`.brand`/`.brand .dot`/`.nav-section`/`.nav-section-label`/`.nav-item`/`.nav-item:hover`/`.nav-item.active`/`.nav-item .count`/`.nav-item.indent` の完全な視覚仕様が定義されている
- **判定**: 確定（フロー分析ステップ4・6で実処理経路上も裏付け済み）

### 候補2: `.media-card .cover/.badge/.fav/.body/.title/.meta`/`.status-dot` のCSSルールが `index.css` に存在せず、Tailwindユーティリティで部分代替されているのみ ⭐⭐⭐

- **証拠**:
  - `docs/tasks/frontend-ui-compliance/TASK-0008.md:28-33` の完了条件、および `_shared.css:223-293` に `.media-card .cover`（グラデーション背景）、`.media-card .badge`（オーバーレイ・モノスペースフォント）、`.media-card .fav`（★色）、`.media-card .body`/`.title`（Source Serif 4・2行クランプ）/`.meta`、`.status-dot.done/.progress/.none` の完全仕様がある
  - `frontend/src/index.css` にはこれらに対応するセレクタが一切存在しない（`.card-grid`（457-461行目）のみ実装済み）
- **判定**: 継続。候補1ほど致命的ではないが同一パターンの欠落

### 候補3: `Sidebar.test.tsx`/`MediaCard.test.tsx` がクラス名文字列の存在確認のみでCSSルール実体を検証しない設計のため、欠落がCIをすり抜けた ⭐⭐⭐⭐

- **証拠**:
  - `frontend/src/components/common/Sidebar.test.tsx:26-213` 全12テストケースが `toBeInTheDocument`/`toHaveAttribute`/`className`文字列包含のみを検証（jsdom環境ではCSSファイルは評価されないため、CSS未定義でも成功する）
  - `docs/tasks/frontend-ui-compliance/TASK-0007.md:26-35` の完了条件6項目もすべて「〜が追加されている」「〜が適用される」というDOM構造・振る舞いの記述であり、「`index.css`にCSSルールを追加する」という項目は明記されていない
- **判定**: 継続。テスト設計と完了条件の両方が「CSS実体の存在」を要求しない構造になっている

### 候補4: TASK-0007/0008に対応する実装記録ディレクトリ（`docs/implements/frontend-ui-compliance/TASK-0007/`, `TASK-0008/`）が作成されていない ⭐⭐⭐

- **証拠**: `docs/implements/frontend-ui-compliance/` 配下には `TASK-0001`, `TASK-0002`, `TASK-0003`, `TASK-0011` のみ存在し、`TASK-0007`, `TASK-0008` のディレクトリが存在しない（コマンド確認済み）
- **判定**: 継続。TDDフロー（red/green/refactor記録）を経ずに完了マークされた傍証

---

## 根本原因の特定

### 発生メカニズム（ステップ図）

```
1. TASK-0007.md/TASK-0008.md 作成時
   └─ 「実装詳細」節に _shared.css のCSSスニペットを参考情報として記載
      ただし「完了条件」チェックリストは
        "〜が追加されている/適用される"（DOM・振る舞いの記述）のみで
        "index.css にCSSルールを追加する" という項目が独立して存在しない
        （TASK-0007.md:28-34, TASK-0008.md:28-33）
                │
                ▼
2. 実装フェーズ（TDD red/green/refactor）
   └─ Sidebar.tsx / MediaCard.tsx にクラス名を付与するのみで実装完了
      → docs/implements/frontend-ui-compliance/TASK-0007, TASK-0008 の
        実装記録ディレクトリ自体が作成されていない
        （他タスクでは記録ディレクトリが存在）
                │
                ▼
3. テスト実行
   └─ Sidebar.test.tsx / MediaCard.test.tsx はクラス名文字列の
      存在確認のみ（jsdomはCSSを評価しない）
      → CSS実体が無くても全テストがグリーン
                │
                ▼
4. overview.md での完了記録
   └─ テストグリーン + 完了条件のチェック項目（DOM構造ベース）を満たしたため
      TASK-0007, TASK-0008 が [x] 完了としてマークされる
      （他タスクは "_shared.css .xxx相当（TASK-00NN）" コメント付きで
       index.css に確実にCSSを追加してから完了マークされている）
                │
                ▼
5. ブラウザでの実レンダリング（http://localhost）
   └─ .sidebar/.brand/.nav-item等のセレクタがCSSOMに存在しないため
      User Agent Stylesheetのデフォルト値のみ適用
      → 暗色背景・階層インデント・ホバーハイライト・ブランドロゴ・
        件数バッジ等が一切表示されず、デザインカンプと乖離
```

### なぜ今まで発見されなかったか

1. **ビルド・型チェックが通る**: 存在しないCSSクラスを参照してもTypeScript/ESLint/vite buildはエラーにしない（CSS Modulesではなくグローバルクラス名文字列のため静的検証の対象外）。
2. **単体テストが通る**: `Sidebar.test.tsx`/`MediaCard.test.tsx` はReact Testing LibraryでDOM構造・クラス名文字列のみを検証し、jsdom環境ではスタイルシートの解決・Computed Styleの検証を行っていない。
3. **完了条件チェックリスト自体がCSS実体を要求していない**: TASK-0007.md/TASK-0008.mdの「完了条件」は「〜が表示される」「〜が適用される」という結果の記述であり、実装者が「クラス名を付与すればブラウザで自動的にその見た目になる」と誤解しやすい書き方になっている（実際には `_shared.css` のCSSスニペットは "実装詳細"節の参考情報にすぎず、それを `index.css` に転記する作業が独立したタスクとして明示されていない）。
4. **コードレビュー時の見落とし**: `index.css` 内に `--color-sidebar` 等のshadcn由来トークン（238-299行目付近）が存在するため、表面的な確認では「sidebar関連のスタイルは何かある」と誤認しやすい。しかしこれは `bg-sidebar` のような合成ユーティリティ用トークンであり、`class="sidebar"`（素のクラス名）には一切作用しない。
5. **視覚回帰テスト・スクリーンショット比較が導入されていない**: デザインカンプ(`01_home.html`)と実装のピクセル差分を機械的に検出する仕組みがなく、目視確認に依存していたため今回のような大きな乖離が長期間放置された。

---

## 詳細コード検証

### Sidebar.tsx が出力するクラス名（frontend/src/components/common/Sidebar.tsx:66-93 相当）

```tsx
<nav className="sidebar" aria-label="グローバルナビゲーション">
  <div className="brand">
    <span className="dot" />
    MediaVault
  </div>
  <div className="nav-section">
    <div className="nav-section-label">ライブラリ</div>
    {/* NavItemLink経由で以下を出力 */}
    <a className={cn('nav-item nav-link', isActive && 'active')} href="/">全体一覧</a>
    <a className={cn('nav-item nav-link indent', isActive && 'active')} href="/collections/general">
      一般メディア
    </a>
    {/* ... */}
  </div>
  <div style={{ marginTop: 'auto' }}>
    {/* 設定 navItem */}
  </div>
</nav>
```

### `_shared.css` 側の対応する正解定義（docs/frontend/ui/_shared.css:76-131）

```css
.sidebar {
  background: var(--bg-sidebar);
  border-right: 1px solid var(--border-soft);
  display: flex;
  flex-direction: column;
  padding: 14px 8px;
  overflow-y: auto;
}

.brand {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 8px 16px;
  font-family: var(--font-display);
  font-weight: 600;
  font-size: 16px;
  color: var(--text-primary);
}

.brand .dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--accent);
  flex-shrink: 0;
}

.nav-section {
  margin-bottom: 18px;
}

.nav-section-label {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--text-faint);
  padding: 4px 8px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: var(--radius);
  color: var(--text-muted);
  font-size: 13px;
  cursor: pointer;
  white-space: nowrap;
}

.nav-item:hover { background: var(--bg-surface-hover); color: var(--text-primary); }
.nav-item.active { background: var(--accent-soft); color: var(--accent-strong); }
.nav-item .count { margin-left: auto; color: var(--text-faint); font-family: var(--font-mono); font-size: 11px; }
.nav-item.indent { padding-left: 22px; }
```

### `index.css` 側の現状（frontend/src/index.css 全体を確認、該当箇所は存在しないため「不在」を確認）

`index.css` 内で `.sidebar` に言及するのはこの1箇所のみ（539-547行目、`@media (max-width: 980px)` ブロック）:

```css
@media (max-width: 980px) {
  .app-shell,
  .app-shell.has-properties {
    grid-template-columns: 1fr;
  }
  .sidebar,
  .properties {
    display: none;
  }
```

上記以外に `.sidebar`/`.brand`/`.nav-item`/`.nav-section`/`.count`/`.dot`/`.indent` の通常時スタイル定義は存在しない。

### MediaCard.tsx とスタイル対応（frontend/src/components/common/MediaCard.tsx:36-81 相当）

```tsx
<div className="media-card" ...>          {/* Tailwindユーティリティで外形のみ代替、.media-cardの本定義なし */}
  <div className="cover">...</div>          {/* index.cssに.cover定義なし */}
  <span className="badge">...</span>        {/* index.cssに.badge定義なし */}
  {item.isFavorite && <span className="fav">★</span>}  {/* index.cssに.fav定義なし */}
  <div className="body">                    {/* index.cssに.body定義なし */}
    <p className="title">{item.title}</p>   {/* index.cssに.title定義なし（Source Serif 4・2行クランプ未反映） */}
    <div className="meta">                  {/* index.cssに.meta定義なし */}
      <span className={cn('status-dot', statusClass)} />  {/* index.cssに.status-dot定義なし */}
    </div>
  </div>
</div>
```

`_shared.css:223-293` に対応する完全定義があるが（後述の修正案に転記）、`index.css`には一切移植されていない。

---

## 影響範囲

- **`Sidebar.tsx`（`RootLayout.tsx` 経由で全画面共通）**: HomePage・ItemDetailPage等、アプリ内の**すべての画面**でサイドバーの視覚表現が欠落する。最も影響が大きい。
- **`HomePage.tsx`**: `.card-grid` 内の `MediaCard` 一覧で `.cover`/`.badge`/`.fav`/`.title`/`.meta`/`.status-dot` の装飾が反映されず、カード一覧全体の見た目がモックと乖離する。
- **`ItemDetailPage.tsx`**: `RootLayout` 経由でSidebarを含むため同様に影響を受ける（詳細画面固有の`.doc`系スタイルはTASK-0011で実装済みのため無関係）。
- **その他 `MediaCard` を使用する画面（コレクション別一覧等）**: 同一コンポーネントを再利用するため同様に影響。

---

## テストケースの検証（既存テストのカバレッジ欠如）

- `frontend/src/components/common/Sidebar.test.tsx:26-213`: 全12テストケースが `toBeInTheDocument`/`toHaveAttribute`/`className`の文字列包含検証のみ。CSSファイルの読み込みやComputed Styleの検証は一切行われていない。jsdom環境では実際のCSSカスケード解決も行われないため、「クラス名が正しく付与されているか」を検証するテストは、スタイル定義そのものの欠落を検出できない構造になっている。
- `MediaCard.test.tsx`も同様の設計（`data-testid`・クラス名文字列・属性値の検証中心）と推定され、`.cover`/`.badge`/`.fav`等のCSS実体欠落を検出できない。
- **根本問題**: 「クラス名が付与されている」ことと「そのクラスにスタイルが定義されている」ことは独立した事実であり、現在のテストスイートは前者のみを検証している。CSS移植漏れという種類の不具合はこのテスト設計では原理的に検出不可能。

---

## 修正の方向性

`frontend/src/index.css` の 528行目 `.main {...}` ブロックの直後（`.properties` プレースホルダの前、または後）に、他タスクと同じ「信頼性コメント」付きで以下を追記する。挿入位置は532行目 `/* 【propertiesプレースホルダ】...*/` の直前が適切（`.app-shell`系ブロックと`@media`ブロックの間、TASK-0009コメントの直後）。

### 修正前（frontend/src/index.css:518-538）

```css
/* 【AppShellスタイル】: _shared.css .app-shell/.app-shell.has-properties相当（Sidebar + main の2カラム、
   アイテム詳細画面のみproperties列を含む3カラム）（TASK-0009 REQ-006） 🔵 */
.app-shell {
  display: grid;
  grid-template-columns: var(--sidebar-w) 1fr;
  height: 100vh;
}
.app-shell.has-properties {
  grid-template-columns: var(--sidebar-w) 1fr var(--properties-w);
}
.main {
  overflow-y: auto;
  min-width: 0;
}
/* 【propertiesプレースホルダ】: 空間確保のみ、中身は実装しない（TASK-0009スコープ外） 🔵 */
.properties {
  border-left: 1px solid var(--border-soft);
  background: var(--bg-sidebar);
  overflow-y: auto;
}
```

### 修正後（`.main` と `.properties` の間にSidebar/MediaCard分を追加）

```css
/* 【AppShellスタイル】: _shared.css .app-shell/.app-shell.has-properties相当（Sidebar + main の2カラム、
   アイテム詳細画面のみproperties列を含む3カラム）（TASK-0009 REQ-006） 🔵 */
.app-shell {
  display: grid;
  grid-template-columns: var(--sidebar-w) 1fr;
  height: 100vh;
}
.app-shell.has-properties {
  grid-template-columns: var(--sidebar-w) 1fr var(--properties-w);
}
.main {
  overflow-y: auto;
  min-width: 0;
}

/* 【Sidebarスタイル】: _shared.css .sidebar/.brand/.nav-section/.nav-item相当
   （暗色背景・ブランドロゴ・セクション見出し・件数バッジ・インデント階層）（TASK-0007 REQ-003） 🔵 */
.sidebar {
  background: var(--bg-sidebar);
  border-right: 1px solid var(--border-soft);
  display: flex;
  flex-direction: column;
  padding: 14px 8px;
  overflow-y: auto;
}

.brand {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 8px 16px;
  font-family: var(--font-display);
  font-weight: 600;
  font-size: 16px;
  color: var(--text-primary);
}

.brand .dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--accent);
  flex-shrink: 0;
}

.nav-section {
  margin-bottom: 18px;
}

.nav-section-label {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--text-faint);
  padding: 4px 8px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: var(--radius);
  color: var(--text-muted);
  font-size: 13px;
  cursor: pointer;
  white-space: nowrap;
}

.nav-item:hover { background: var(--bg-surface-hover); color: var(--text-primary); }
.nav-item.active { background: var(--accent-soft); color: var(--accent-strong); }
.nav-item .count { margin-left: auto; color: var(--text-faint); font-family: var(--font-mono); font-size: 11px; }
.nav-item.indent { padding-left: 22px; }

/* 【MediaCardスタイル】: _shared.css .media-card/.cover/.badge/.fav/.body/.title/.meta/.status-dot相当
   （カバー画像プレースホルダ・バッジオーバーレイ・お気に入り・ステータスドット）（TASK-0008 REQ-004） 🔵 */
.media-card {
  background: var(--bg-surface);
  border: 1px solid var(--border-soft);
  border-radius: var(--radius);
  overflow: hidden;
  cursor: pointer;
  transition: border-color 0.15s, transform 0.1s;
}
.media-card:hover { border-color: var(--accent); transform: translateY(-2px); }

.media-card .cover {
  aspect-ratio: 2/3;
  background: linear-gradient(160deg, #33304a, #232323);
  display: flex;
  align-items: flex-end;
  justify-content: flex-end;
  padding: 6px;
  position: relative;
}

.media-card .badge {
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.04em;
  background: rgba(0,0,0,0.55);
  color: var(--text-muted);
  padding: 2px 6px;
  border-radius: 4px;
  position: absolute;
  top: 6px;
  left: 6px;
}

.media-card .fav {
  color: var(--favorite);
  font-size: 14px;
}

.media-card .body {
  padding: 10px 10px 12px;
}

.media-card .title {
  font-family: var(--font-display);
  font-size: 13.5px;
  font-weight: 600;
  margin: 0 0 4px;
  color: var(--text-primary);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.media-card .meta {
  font-size: 11px;
  color: var(--text-faint);
  display: flex;
  align-items: center;
  gap: 6px;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  display: inline-block;
}
.status-dot.done { background: var(--status-done); }
.status-dot.progress { background: var(--status-progress); }
.status-dot.none { background: var(--status-none); }

/* 【propertiesプレースホルダ】: 空間確保のみ、中身は実装しない（TASK-0009スコープ外） 🔵 */
.properties {
  border-left: 1px solid var(--border-soft);
  background: var(--bg-sidebar);
  overflow-y: auto;
}
```

**補足**: `--bg-sidebar`, `--border-soft`, `--accent`, `--accent-soft`, `--accent-strong`, `--text-primary`, `--text-muted`, `--text-faint`, `--font-display`, `--font-mono`, `--radius`, `--favorite`, `--status-done`, `--status-progress`, `--status-none` 等のCSSカスタムプロパティは、TASK-0001/0002で `index.css:15-92` の `:root` に既に定義済みであることを確認済み（`.titlebar`等の既存実装が同トークンを問題なく参照している）ため、追加のトークン定義は不要。

---

## 再発防止策

1. **テストにComputed Style検証を追加する**: `Sidebar.test.tsx`/`MediaCard.test.tsx`に、jsdom + `getComputedStyle`（またはVitestの`@testing-library/jest-dom`拡張）を用いて、主要クラス（`.sidebar`の`background-color`、`.nav-item`の`padding`等）に期待通りのスタイルが適用されていることを確認するテストケースを追加する。ただしjsdomはCSSファイルを自動読込しないため、テストのセットアップで対象CSSを明示的に読み込む仕組み（vitest configでのCSS処理設定）が必要。
2. **視覚回帰テストの導入**: Playwright等でデザインカンプ(`01_home.html`)と実装ページのスクリーンショット比較（pixel diff）を行うテストをCIに組み込み、レイアウト崩れやスタイル欠落を機械的に検出できるようにする。
3. **CSS移植のチェックリスト化**: 今後のUI準拠タスク（TDDタスク）のテンプレートに、「完了条件」とは別に「`_shared.css`の該当クラスをすべて`index.css`に移植し、`_shared.css .xxx相当（TASK-00NN REQ-xxx）`形式のコメントを付与したか」という明示的な必須チェック項目を追加する。TASK-0004/0005/0009/0010/0011で実施されている命名規則を全タスク共通のルールとして`architecture.md`等に明文化する。
4. **タスク完了条件のレビュー基準の見直し**: 「〜が表示される」という結果ベースの完了条件だけでなく、「対応するCSSルールが`index.css`の該当箇所に存在すること」を完了条件のチェック項目として独立させ、レビュー時に`git diff`で`index.css`の変更有無を確認するプロセスを追加する。
5. **`docs/implements/`配下の実装記録ディレクトリ作成の徹底**: TDDタスクは全て`docs/implements/frontend-ui-compliance/TASK-00NN/`にred/green/refactorの記録を残すことを完了条件の必須項目とし、記録が存在しないタスクは`overview.md`で完了マークできないルールとする。

---

*この根本原因分析結果に基づいて、修正実装を行うことを推奨します。*
