# TASK-0001: デザイントークンの再定義（index.css）- TDD Greenフェーズ

## 1. 実施内容

`frontend/src/index.css` を以下の方針で編集し、design-tokens-red-phase.md の「Greenフェーズで実装すべき内容」を全て反映した。

1. **旧トークン削除**: `:root` から `--bg-base`, `--bg-elevated`, `--text-secondary`, `--border-default` の宣言を削除（`--bg-surface`, `--text-primary`, `--border`, `--accent` は変数名を維持しつつ値を新値へ置換） 🔵
2. **新トークン追加**: 背景系（`--bg-app`, `--bg-sidebar`, `--bg-surface`, `--bg-surface-hover`, `--bg-input`）、境界線系（`--border`, `--border-soft`）、文字色系（`--text-primary`, `--text-muted`, `--text-faint`）、単一アクセント色（`--accent`, `--accent-strong`, `--accent-soft`）、ステータス色（`--favorite`, `--status-progress`, `--status-done`, `--status-none`, `--danger`）を `:root` に追加 🔵
3. **フォント導入**: Google Fonts CDN `@import` を追加し、`--font-ui`, `--font-display`, `--font-mono` を定義 🟡
4. **レイアウト・角丸トークン**: `--sidebar-w: 232px`, `--properties-w: 300px`, `--radius: 6px` を追加。既存shadcn用 `--radius: 0.625rem` は `--radius-shadcn` にリネームして共存 🟡
5. **`@theme inline` 追随修正**:
   - `--color-bg-base`, `--color-bg-elevated`, `--color-text-secondary`, `--color-border-default` の参照先を、削除された旧トークンから新トークン（`--bg-app`, `--bg-surface-hover`, `--text-muted`, `--border`）に付け替え、未定義変数参照によるTailwindユーティリティ破壊を回避（design-tokens-requirements.md 4節エッジケース「`@theme inline`との不整合」に対応） 🟡
   - `--radius-sm`〜`--radius-4xl` の計算式参照を `var(--radius)` から `var(--radius-shadcn)` に書き換え、新 `--radius: 6px` への巻き込み事故を防止（TC-05対応） 🟡
6. **media_type別アクセントカラー8色は変更なし** 🔵

## 2. テスト結果

### 新規テスト
```
yarn test design-tokens.test.ts
```
結果: `Test Files 1 passed (1)` / `Tests 10 passed (10)` — 全件成功

### 既存テスト（全体回帰）
```
yarn test
```
結果: `Test Files 22 passed (22)` / `Tests 192 passed (192)` — 全件成功（既存コンポーネントテストへの回帰なし）

### ビルド確認
```
yarn build
```
結果: 成功（`tsc -b && vite build` エラーなし完了）。CSSバンドル時に `@import`順序に関する警告が1件出力されるが、ビルド自体は正常終了しており機能上の問題はない。

## 3. スコープ外の追随修正（ビルド前提のため実施）

- `frontend/tsconfig.app.json` の `compilerOptions.types` に `"node"` を追加。
  - 理由: Redフェーズで作成された `design-tokens.test.ts` が `node:fs` / `node:path` / `__dirname` を使用しており、`tsc -b`（`yarn build`に含まれる型チェック）でこれらの型が解決できずビルドが失敗していたための最小限の追随修正（`@types/node`は既にインストール済みで、tsconfigの`types`指定への追加のみ）。

## 3.1 差し戻し修正（完了条件8未達への対応）

`tdd-verify-complete` にて完了条件8「`yarn dev`でHomePage等の背景色が目視で暗いグレー（`#1e1e1e`系）になっている」が未達成と判定されたため、以下の最小修正を追加実施した。

- **問題**: `:root` に `--bg-app: #1e1e1e` 等の新トークンは定義済みだったが、実際の `body` は `@apply bg-background text-foreground;`（shadcnのTailwindユーティリティ、`--background` oklch値参照）のみで配線されており、`--bg-app` が実際の画面表示に反映されていなかった。
- **修正**: `frontend/src/index.css` の `@layer base` 内 `body` ルールに `background: var(--bg-app);` を追記し、Tailwindユーティリティ由来の `background-color` を直後の `background` ショートハンドで上書きする形で新トークンを配線した（`bg-background` クラス自体・`@theme inline`のマッピングは変更せず共存。Tailwindユーティリティ体系の本格連携はTASK-0002スコープのため対象外）。
- **確認**:
  - `yarn build`: 成功。生成CSSを確認したところ `body{background-color:var(--background);color:var(--foreground);background:var(--bg-app)}` となっており、`background` ショートハンドが後勝ちで `--bg-app`（`#1e1e1e`）が実際の背景色として適用されることを確認。
  - `yarn test`: `Test Files 22 passed (22)` / `Tests 192 passed (192)` — 既存テストへの回帰なし。

## 4. 課題・改善点（Refactorフェーズ候補）

- CSSバンドル時の `@import` 順序警告（Tailwindの内部バンドル順序起因、機能影響なし）は次フェーズで解消可否を検討する。→ **Refactorフェーズで解消済み**（design-tokens-refactor-phase.md参照）。
- `.dark` クラスおよび `@media (prefers-color-scheme: dark)` ブロック内に残る旧shadcn由来の `--border`, `--accent` oklch値の扱いはTASK-0002のスコープで整理する。
- `@theme inline` 内のマッピング名（`--color-bg-base`等）自体の妥当性・命名整理はTASK-0002で本格対応する（本タスクでは未定義参照回避の最小限修正のみ）。
