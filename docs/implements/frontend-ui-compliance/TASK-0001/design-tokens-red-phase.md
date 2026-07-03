# TASK-0001: デザイントークンの再定義（index.css）- TDD Redフェーズ

## 0. 方針

design-tokens-testcases.md 0.5節の確定方針どおり、CSS変数値そのものはjsdomで実カスケード計算できず
直接アサーション困難なため、**index.cssのファイル内容（テキスト）を直接読み込みgrep相当の正規表現で
検証するVitestテスト**をRedフェーズの成果物とした（TC-04, TC-05に対応）。

## 1. 作成したテストファイル

- `frontend/src/design-tokens.test.ts`
- テストケース数: 10（TC-04-1〜TC-04-9, TC-05）

## 2. テストケース一覧

| No | 内容 | 対応TC | 信頼性 |
|----|------|--------|--------|
| TC-04-1 | 背景色トークン5種の存在確認 | TC-04 | 🔵 |
| TC-04-2 | 境界線トークン2種の存在確認 | TC-04 | 🔵 |
| TC-04-3 | 文字色トークン3種の存在確認 | TC-04 | 🔵 |
| TC-04-4 | 単一アクセント色3種の存在確認 | TC-04 | 🔵 |
| TC-04-5 | ステータス色5種の存在確認 | TC-04 | 🔵 |
| TC-04-6 | フォント変数3種の存在確認 | TC-04 | 🔵 |
| TC-04-7 | レイアウト・角丸トークン4種の存在確認（--radius-shadcn含む） | TC-04 | 🔵🟡 |
| TC-04-8 | media_type別アクセントカラー8色の非変更確認 | TC-08 | 🔵 |
| TC-04-9 | 旧トークン（--bg-base等）の削除確認 | TC-04 | 🔵 |
| TC-05 | @theme inline内--radius参照の--radius-shadcnへの追随修正確認 | TC-05 | 🟡 |

## 3. 実行コマンドと結果

```
yarn test design-tokens.test.ts
```

結果: `Test Files 1 failed (1)` / `Tests 9 failed | 1 passed (10)`

- 失敗した9件: 新トークン未定義・旧トークン未削除・--radius-shadcn未対応のため、いずれも意図通り失敗
- 成功した1件（TC-04-8）: media_type別アクセントカラー8色は本タスクで変更対象外のため、現状のindex.cssの値のまま一致し成功（回帰確認としては正しい状態）

## 4. Greenフェーズで実装すべき内容

`frontend/src/index.css` の `:root` ブロックを以下のとおり編集する:

1. 旧トークン（`--bg-base`, `--bg-surface`, `--bg-elevated`, `--text-primary`, `--text-secondary`, `--border-default`）を削除
2. 新トークンを追加:
   - 背景系: `--bg-app: #1e1e1e`, `--bg-sidebar: #161616`, `--bg-surface: #262626`, `--bg-surface-hover: #2c2c2c`, `--bg-input: #1c1c1c`
   - 境界線系: `--border: #383838`（既存oklch値を置換）, `--border-soft: #2e2e2e`
   - 文字色系: `--text-primary: #dcddde`（置換）, `--text-muted: #8a8a8d`, `--text-faint: #5c5c5f`
   - 単一アクセント色: `--accent: #8b6cf6`（置換）, `--accent-strong: #a48bf8`, `--accent-soft: rgba(139,108,246,0.15)`
   - ステータス色: `--favorite: #e0a85a`, `--status-progress: #5aa9e0`, `--status-done: #5ac98a`, `--status-none: #6b6b6e`, `--danger: #e0615a`
   - フォント: `--font-ui: 'Inter', -apple-system, sans-serif`, `--font-display: 'Source Serif 4', Georgia, serif`, `--font-mono: 'JetBrains Mono', monospace`
   - レイアウト・角丸: `--sidebar-w: 232px`, `--properties-w: 300px`, `--radius: 6px`（置換）, `--radius-shadcn: 0.625rem`（新規、旧--radiusの値を退避）
3. `@theme inline` ブロック内 `--radius-sm`〜`--radius-4xl` の計算式内 `var(--radius)` を `var(--radius-shadcn)` に書き換える
4. media_type別アクセントカラー8色は変更しない
5. Google Fonts CDN `@import` を追加（Inter, Source Serif 4, JetBrains Mono）

実装後、`yarn test design-tokens.test.ts` を再実行し全件成功（10 passed）することを確認する。
併せて `yarn test`（全体回帰）と `yarn build` の成功も確認する。
