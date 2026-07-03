# 最終レポート: HomePage表示がデザインカンプ(01_home.html)と大きくかけ離れている

**実施日**: 2026-07-03
**分析者**: Claude Code

[← インデックスに戻る](./index.md) | [初期調査](./initial_investigation.md) | [処理フロー分析](./flow_analysis.md) | [根本原因分析](./root_cause_analysis.md)

---

## 1. エグゼクティブサマリー

`docs/frontend/ui/01_home.html`（デザインカンプ）をベースに実装したはずのUIが、`http://localhost` で実際に表示されるページと大きくかけ離れている問題を調査した結果、根本原因は **`frontend/src/index.css` に `Sidebar.tsx`／`MediaCard.tsx` が出力するクラス名に対応する実CSSルールが一切存在しないこと** であると特定した。

これはビルドエラーにもテスト失敗にもならない「静的CSS定義の欠落」であり、TASK-0007（Sidebar拡張）・TASK-0008（MediaCard拡張）が `overview.md` 上で完了マークされているにもかかわらず、他タスク（TASK-0004, 0005, 0009, 0010, 0011）で必ず行われている「`_shared.css` からのCSS移植＋信頼性コメント付与」という作業だけがこの2タスクで欠落したまま見過ごされたことが直接の原因である。

修正は `frontend/src/index.css` に不足しているCSSブロックを追記するだけで完結し、コンポーネント側（TSX）の変更は不要。本レポート末尾に、そのまま適用できる修正コードを完全な形で記載する。

---

## 2. バグの詳細情報

| 項目 | 内容 |
|---|---|
| 概要 | `docs/frontend/ui/01_home.html` と `http://localhost` の実表示が大きく乖離 |
| 種類 | UI/表示バグ |
| 発生状況 | 常に発生（`http://localhost` を開くと必ず発生） |
| エラー情報 | なし（コンソールエラーなし。見た目のみの乖離） |
| 再現手順 | ブラウザで `http://localhost` を開くだけ |
| 影響範囲 | アプリ内の全画面（Sidebarは`RootLayout`経由で共通表示） |

---

## 3. 分析結果のサマリー

### 3.1 初期調査（[initial_investigation.md](./initial_investigation.md)）

`frontend/src/index.css` を全文grepし、`.sidebar`/`.brand`/`.nav-item`/`.nav-section`/`.count`/`.media-card`系のクラス定義が実質的に存在しないこと（唯一のヒットは`@media`内の`display:none`指定のみ）を確認した。デザインカンプの正解CSSである `docs/frontend/ui/_shared.css` には該当クラスの完全な定義が存在する。

### 3.2 処理フロー分析（[flow_analysis.md](./flow_analysis.md)）

`App.tsx` → `routes.tsx` → `RootLayout.tsx` → `Sidebar.tsx` → `index.css` → ブラウザレンダリングという実処理経路を辿り、以下を確定した。

- ルーティング設定（`frontend/src/routes.tsx:16-40`）に誤りはなく、候補として挙げていた「ルーティングmiss」は**除外**した。
- `Sidebar.tsx` が出力する `sidebar`/`brand`/`nav-item`等は素のクラス名であり、Tailwindのユーティリティとしては解釈されない。
- `index.css` 内の `--color-sidebar` 等のshadcn由来トークンは `bg-sidebar` のような合成ユーティリティ生成用のものであり、`class="sidebar"`（素のクラス名）には一切作用しない（誤認しやすいポイントとして明記）。
- 他タスク（TASK-0004/0005/0009/0010/0011）は移植コメント付きでCSSが実装済みだが、TASK-0007由来のコメントのみが皆無という一貫した欠落パターンを確認した。

### 3.3 根本原因分析（[root_cause_analysis.md](./root_cause_analysis.md)）

TASK-0007.md/TASK-0008.mdの完了条件が「〜が追加されている／適用される」という結果ベースの記述にとどまり、「`index.css`へのCSS移植」を独立必須項目としていなかったこと、および `Sidebar.test.tsx`/`MediaCard.test.tsx` がクラス名文字列の存在確認のみでCSS実体を検証しない設計であったことが、欠落を検出できないままテストをグリーンにし、`overview.md`で完了マークされる結果につながったことを確定した。

---

## 4. 根本原因の詳細

### 4.1 技術的説明

`Sidebar.tsx`・`MediaCard.tsx` が出力するクラス名（`.sidebar`, `.brand`, `.nav-item`, `.nav-section`, `.count`, `.dot`, `.indent`, `.media-card .cover/.badge/.fav/.body/.title/.meta`, `.status-dot` 等）は、Tailwindのユーティリティクラス命名規則（`bg-`, `text-`, `flex`等）に一致しない「素のクラス名」である。これらはグローバルCSS（`frontend/src/index.css`）側にセレクタとしての定義が存在しない限り、ブラウザのUser Agent Stylesheet（`nav`/`div`/`a`要素等の初期値）しか適用されない。

`frontend/src/index.css` を全548行確認した結果、該当クラスの通常時スタイル定義は一切存在せず、唯一の`.sidebar`言及は以下の非表示指定のみであった（frontend/src/index.css:539-547付近）。

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

一方、モックアップの正解CSSである `docs/frontend/ui/_shared.css:75-131`（Sidebar分）・`_shared.css:223-293`（MediaCard分）には該当クラスの完全な視覚仕様（暗色背景・ホバー色・フォント・レイアウト等）が定義されている。他タスク（TASK-0004, 0005, 0009, 0010, 0011）に対応するCSSブロックには「`_shared.css .xxx相当（TASK-00NN REQ-xxx）」という信頼性コメント付きで移植が行われているのに対し、TASK-0007（Sidebar）・TASK-0008（MediaCard）分だけこのパターンが完全に欠落している。

### 4.2 発生メカニズム図

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

### 4.3 なぜ今まで発見されなかったか

1. **ビルド・型チェックが通る**: グローバルクラス名文字列は静的検証の対象外であり、存在しないCSSクラスを参照してもTypeScript/ESLint/vite buildはエラーにしない。
2. **単体テストが通る**: `Sidebar.test.tsx`/`MediaCard.test.tsx` はDOM構造・クラス名文字列のみを検証し、jsdom環境ではスタイルシートの解決・Computed Styleの検証を行っていない。
3. **完了条件チェックリスト自体がCSS実体を要求していない**: TASK-0007.md/TASK-0008.mdの「完了条件」は結果ベースの記述であり、`_shared.css` のCSSスニペットは"実装詳細"節の参考情報にすぎず、`index.css`への転記が独立タスクとして明示されていない。
4. **コードレビュー時の見落とし**: `index.css` 内の `--color-sidebar` 等のshadcn由来トークン（index.css:259-266付近）の存在により、「sidebar関連のスタイルは何かある」と誤認しやすい。
5. **視覚回帰テストが未導入**: デザインカンプと実装のピクセル差分を機械的に検出する仕組みがなく、目視確認に依存していた。

---

## 5. 修正方針（そのまま適用可能なコード）

`frontend/src/index.css` の `.main {...}` ブロックと `.properties` ブロックの間（TASK-0009コメントブロックの直後、`/* 【propertiesプレースホルダ】...*/` の直前）に、以下を追記する。

### 修正前（frontend/src/index.css:518-538 相当）

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

## 6. テスト戦略

修正適用後、以下の観点で確認する。

1. **手動確認**: `npm run dev` 相当でローカルサーバーを起動し、`http://localhost` をブラウザで開き、Sidebarが暗色背景・ブランドロゴ・階層インデント・ホバーハイライトを持つこと、MediaCardがカバー画像プレースホルダ・バッジ・お気に入り・ステータスドットを表示することをDevToolsのComputed Styleで確認する。
2. **既存テストの回帰確認**: `Sidebar.test.tsx`（全12ケース）・`MediaCard.test.tsx`はクラス名の存在確認のみのため、CSS追記による影響は受けず、そのままグリーンで通ることを確認する（既存テストのカバレッジがCSS実体を検証しないこと自体が別途対応すべき課題、6-3参照）。
3. **今後追加すべきテスト**（再発防止策と重複するため詳細は次節）: Computed Style検証テスト、視覚回帰テスト（Playwright pixel diff）を新規追加し、同種の欠落を機械的に検出できるようにする。

---

## 7. 影響範囲

| 対象 | 影響内容 |
|---|---|
| `Sidebar.tsx`（`RootLayout.tsx`経由で全画面共通） | HomePage・ItemDetailPage等、アプリ内の**すべての画面**でサイドバーの視覚表現が欠落。最も影響が大きい。 |
| `HomePage.tsx` | `.card-grid`内の`MediaCard`一覧で`.cover`/`.badge`/`.fav`/`.title`/`.meta`/`.status-dot`の装飾が反映されず、カード一覧全体がモックと乖離。 |
| `ItemDetailPage.tsx` | `RootLayout`経由でSidebarを含むため同様に影響（詳細画面固有の`.doc`系スタイルはTASK-0011で実装済みのため無関係）。 |
| その他`MediaCard`を使用する画面 | 同一コンポーネントを再利用するため同様に影響。 |
| 修正の副作用 | `index.css`へのCSS追記のみでコンポーネント側（TSX）は無変更のため、既存の振る舞い・ロジックへの影響はない。 |

---

## 8. 再発防止策

1. **テストにComputed Style検証を追加する**: `Sidebar.test.tsx`/`MediaCard.test.tsx`に、jsdom + `getComputedStyle`を用いて、主要クラス（`.sidebar`の`background-color`、`.nav-item`の`padding`等）に期待通りのスタイルが適用されていることを確認するテストケースを追加する。ただしjsdomはCSSファイルを自動読込しないため、テストのセットアップで対象CSSを明示的に読み込む仕組み（vitest configでのCSS処理設定）が必要。
2. **視覚回帰テストの導入**: Playwright等でデザインカンプ(`01_home.html`)と実装ページのスクリーンショット比較（pixel diff）を行うテストをCIに組み込み、レイアウト崩れやスタイル欠落を機械的に検出できるようにする。
3. **CSS移植のチェックリスト化**: 今後のUI準拠タスクのテンプレートに、「完了条件」とは別に「`_shared.css`の該当クラスをすべて`index.css`に移植し、`_shared.css .xxx相当（TASK-00NN REQ-xxx）`形式のコメントを付与したか」という明示的な必須チェック項目を追加する。TASK-0004/0005/0009/0010/0011で実施されている命名規則を全タスク共通のルールとして`architecture.md`等に明文化する。
4. **タスク完了条件のレビュー基準の見直し**: 「〜が表示される」という結果ベースの完了条件だけでなく、「対応するCSSルールが`index.css`の該当箇所に存在すること」を完了条件のチェック項目として独立させ、レビュー時に`git diff`で`index.css`の変更有無を確認するプロセスを追加する。
5. **`docs/implements/`配下の実装記録ディレクトリ作成の徹底**: TDDタスクは全て`docs/implements/frontend-ui-compliance/TASK-00NN/`にred/green/refactorの記録を残すことを完了条件の必須項目とし、記録が存在しないタスクは`overview.md`で完了マークできないルールとする。

---

## 9. タイムライン

| 段階 | 内容 |
|---|---|
| 初期調査 | `index.css`の全文grepにより、`.sidebar`/`.brand`/`.nav-item`/`.nav-section`/`.count`/`.media-card`系のクラス定義が実質不在であることを特定。候補1（Sidebar未定義）・候補2（MediaCard未定義）・候補3（ルーティング誤設定、保留）を提示。 |
| 処理フロー分析 | `App.tsx`→`routes.tsx`→`RootLayout.tsx`→`Sidebar.tsx`→`index.css`の実処理経路を追跡し、候補3（ルーティング誤設定）を証拠をもって除外。他タスクとのCSS移植パターンの一貫性欠如、`--color-sidebar`トークンの誤認可能性を新たに発見。 |
| 根本原因分析 | TASK-0007.md/TASK-0008.mdの完了条件記述、テスト設計（`Sidebar.test.tsx`）、実装記録ディレクトリの不在を突き合わせ、根本原因（完了条件の記述方式とテスト設計の両方がCSS実体の存在を要求しない構造）を確定。具体的な修正コードと再発防止策を策定。 |
| 最終レポート | 3段階の分析結果を統合し、コピペ適用可能な修正コード・テスト戦略・再発防止策を含む本レポートを作成。 |

---

## 10. 結論

本バグの根本原因は、`frontend/src/index.css` に `Sidebar.tsx`・`MediaCard.tsx` が使用するクラス名（`.sidebar`, `.brand`, `.nav-item`, `.nav-section`, `.count`, `.dot`, `.indent`, `.media-card .cover/.badge/.fav/.body/.title/.meta`, `.status-dot`等）に対応する実CSSルールが一切存在しないことである。これはTASK-0007（Sidebar拡張）・TASK-0008（MediaCard拡張）において、他タスクで一貫して行われていた「`_shared.css`からのCSS移植＋信頼性コメント付与」という作業のみが欠落したまま、結果ベースの完了条件とクラス名文字列のみを検証するテストによって「完了」と誤って記録されたことに起因する。

修正は本レポート第5節に記載した具体的なCSSブロックを `frontend/src/index.css` に追記するのみで完結し、コンポーネント側の変更は不要である。あわせて、Computed Style検証テストの追加・視覚回帰テストの導入・タスク完了条件へのCSS移植チェック項目の明文化を行うことで、同種の「静的CSS定義の欠落」が再発しテストをすり抜ける事態を防止できる。

---

*本レポートは初期調査・処理フロー分析・根本原因分析の3段階の分析結果を統合した最終成果物である。*
