# TASK-0001: デザイントークンの再定義（index.css）- TDD要件定義書

## 0. メタ情報

- **機能名**: design-tokens（デザイントークン再定義）
- **タスクID**: TASK-0001
- **要件名**: frontend-ui-compliance
- **対象タスクファイル**: `docs/tasks/frontend-ui-compliance/TASK-0001.md`
- **参照ノート**: `docs/implements/frontend-ui-compliance/TASK-0001/note.md`

---

## 1. 機能の概要（EARS要件定義書・設計文書ベース）

- 🔵 `frontend/src/index.css` の `:root` に定義されている旧デザイントークン（`--bg-base`, `--bg-surface`, `--bg-elevated`, `--text-primary`, `--text-secondary`, `--border-default` 等）を、モックアップ共通スタイル `docs/frontend/ui/_shared.css` に定義済みの値へ**直接置換**する機能（`_shared.css`は要件定義書REQ-001, architecture.md「デザイントークン層」より）
- 🔵 解決する問題: 現行UIのダークテーマの配色・タイポグラフィがモックアップ（Obsidianライクな3ペインダークUI）と乖離しているため、デザイントークンの値をモックアップ準拠に揃え、以後の全コンポーネント/画面実装（Phase 1以降）が一貫した見た目になる土台を作る
- 🔵 想定ユーザー: MediaVaultのエンドユーザー（アプリを操作して暗い背景・単一アクセント色のUIを見る利用者）。直接の作業対象は本タスクを実施する開発者
- 🔵 システム内での位置づけ: Phase 1「デザイントークン基盤」の最初のタスクであり、後続タスク（TASK-0002以降、全コンポーネント/画面実装タスク）すべての前提となる基盤変更（依存タスクなし、後続タスクTASK-0002等が本タスクに依存）
- **参照したEARS要件**: REQ-001（デザイントークン再定義）, REQ-002（media_type別アクセントカラー維持）, REQ-402（shadcn由来oklchトークンの上書き方針）
- **参照した設計文書**: `docs/design/frontend-ui-compliance/architecture.md`「デザイントークン層」節

---

## 2. 入力・出力の仕様（EARS機能要件・TypeScript型定義ベース）

本タスクはCSS変数定義の置換であり、実行時の関数的な入出力（引数・戻り値）は存在しない。ここでは「入力＝変更前のCSSファイル状態」「出力＝変更後のCSSファイル状態」として整理する。

### 入力（変更前の状態）🔵
*出典: `frontend/src/index.css`実測（現状ファイル冒頭に記載）*

- `:root` ブロック内の旧トークン（置換対象）:
  - `--bg-base: #0f1115`
  - `--bg-surface: #1a1d23`
  - `--bg-elevated: #232730`
  - `--text-primary: #f5f5f5`
  - `--text-secondary: #9ca3af`
  - `--border-default: #2d313a`
- 既存の`--radius: 0.625rem`（shadcn用、`@theme inline`の`--radius-sm`〜`--radius-4xl`計算に使用中、行99, 248-254）
- media_type別アクセントカラー8色（`--accent-anime`等、行18-25、**変更対象外**）
- フォント: `@fontsource-variable/geist`をnpmパッケージ経由でimport（行4）、`@theme inline`内で`--font-sans: 'Geist Variable', sans-serif`（行216）

### 出力（変更後の状態）🔵
*出典: `docs/tasks/frontend-ui-compliance/TASK-0001.md`完了条件、`docs/frontend/ui/_shared.css`実測値*

- `:root` に以下のトークンが追加・置換されている:
  - 背景系: `--bg-app: #1e1e1e`, `--bg-sidebar: #161616`, `--bg-surface: #262626`, `--bg-surface-hover: #2c2c2c`, `--bg-input: #1c1c1c`
  - 境界線系: `--border: #383838`, `--border-soft: #2e2e2e`
  - 文字色系: `--text-primary: #dcddde`, `--text-muted: #8a8a8d`, `--text-faint: #5c5c5f`
  - 単一アクセント色: `--accent: #8b6cf6`, `--accent-strong: #a48bf8`, `--accent-soft: rgba(139,108,246,0.15)`
  - ステータス色: `--favorite: #e0a85a`, `--status-progress: #5aa9e0`, `--status-done: #5ac98a`, `--status-none: #6b6b6e`, `--danger: #e0615a`
  - フォント: `--font-ui: 'Inter', -apple-system, sans-serif`, `--font-display: 'Source Serif 4', Georgia, serif`, `--font-mono: 'JetBrains Mono', monospace`
  - レイアウト・角丸: `--sidebar-w: 232px`, `--properties-w: 300px`, `--radius: 6px`
- media_type別アクセントカラー8色は**変更されず維持**（値・変数名とも同一）
- 既存の`--border: oklch(0.922 0 0)`（shadcn用、行30）と新規`--border: #383838`（`_shared.css`由来）は**変数名が衝突**するため、共存方法の決定が必要（詳細は3節参照）
- 既存の`--radius: 0.625rem`（shadcn用）と新規`--radius: 6px`（`_shared.css`由来）も**変数名が衝突**するため、別名（例: `--radius-shadcn: 0.625rem`）へのリネームが必要 🟡（note.md/タスク定義に明記、資料に厳密な命名指定なし）
- 新規Google Fonts CDN `@import`文が`index.css`冒頭付近に追加される

### データフロー 🟡
- ビルド時: `index.css`が Vite の CSS 処理パイプラインでバンドルされ、`:root`のCSS変数が全コンポーネントに配信される
- 実行時: 各コンポーネント（Tailwindユーティリティ経由、または直接`var(--xxx)`参照）がブラウザのCSSカスケードでトークン値を解決する
- 🟡 本タスクでは`@theme inline`のマッピング名自体は変更しないため、Tailwindユーティリティクラス経由の値反映はTASK-0002まで発生しない可能性がある（資料に明記なし、`@theme inline`未変更のため妥当な推測）

- **参照したEARS要件**: REQ-001（トークン値の置換仕様）, REQ-002（media_type色の非変更）
- **参照した設計文書**: `docs/tasks/frontend-ui-compliance/TASK-0001.md`実装詳細節（旧トークン置換、フォント導入、レイアウト・角丸トークン追加の各コードブロック）

---

## 3. 制約条件（EARS非機能要件・アーキテクチャ設計ベース）

- 🔵 **アーキテクチャ制約**: `_shared.css`を独立ファイルとして残し二重管理しない。`:root`の値を直接`index.css`内に定義する（`docs/design/frontend-ui-compliance/architecture.md`「デザイントークン層」）
- 🔵 **スコープ制約**: 本タスクは`:root`の値定義のみを対象とし、`@theme inline`ブロックのマッピング名変更（Tailwindユーティリティ・shadcnトークン連携）はTASK-0002で扱う（TASK-0001.mdに明記）
- 🔵 **互換性制約**: media_type別アクセントカラー8色（`--accent-anime`等）はREQ-002により絶対に変更しない
- 🟡 **名前衝突回避**: 既存の`--radius: 0.625rem`（shadcn用）と新規`--radius: 6px`が衝突するため、既存側を別名（例: `--radius-shadcn`）にリネームして共存させる（資料に厳密指定なし、衝突回避の一般的手法からの妥当な推測）。同様に既存`--border: oklch(...)`（shadcn/`@theme inline`が参照、行232 `--color-border: var(--border)`）と新規`--border: #383838`も衝突するため、同じ方針の検討が必要（🔴 資料に`--border`衝突への言及なし、`--radius`と同様の対応が必要と判断した推測。実装時にどちらを優先するか、または別名化するかを決定する必要がある）
- 🟡 **導入手段制約**: フォント導入はGoogle Fonts CDN `@import`方式を採用（既存の`@fontsource-variable/geist`はnpムパッケージ経由だが、資料に明示の指定がないため`_shared.css`との整合を優先した妥当な推測）
- 🔵 **技術スタック制約**: React/TypeScript/Vite/Tailwind CSS v4 + shadcn/uiの既存構成を維持し、追加ライブラリは導入しない（architecture.md互換性制約）
- 🔵 **アクセシビリティ要件**: 色変更後もコントラスト比がWCAG 2.1 AA基準を満たすこと（tech-stack.md品質基準、architecture.md互換性制約）
- 🔵 **ビルド成功要件**: `yarn build`（`tsc -b && vite build`、型チェック含む）がエラーなく完了すること（TASK-0001.md完了条件）
- 🟡 **単体テスト方針制約**: CSS変数自体は既存テストフレームワーク（Vitest + Testing Library）で直接アサーションすることが困難なため、本タスクでは新規の単体テストコードを作成しない方針（資料に単体テスト手法の指定なし、CSS変数特性からの妥当な判断）。既存コンポーネントテストが参照するTailwindユーティリティクラス名が変更後も同一であることの回帰確認のみ実施する
- **参照したEARS要件**: NFR系（アクセシビリティ・互換性、requirements.md該当節）, REQ-002
- **参照した設計文書**: `docs/design/frontend-ui-compliance/architecture.md`「デザイントークン層」「互換性制約」節、`docs/tasks/frontend-ui-compliance/TASK-0001.md`完了条件・実装詳細節

---

## 4. 想定される使用例（EARSEdgeケース・データフローベース）

### 基本的な使用パターン 🔵
- 開発者が`frontend/src/index.css`の`:root`ブロックを編集し、旧トークンを`_shared.css`準拠の値に置換する
- 編集後`yarn build`を実行し、型チェック・ビルドがエラーなく完了することを確認する
- `yarn dev`で開発サーバーを起動し、HomePage・ItemDetailPage・SettingsPage等の背景色が`#1e1e1e`系の暗いグレーになっていることを目視確認する
- 各画面のMediaCardバッジ等でmedia_type別アクセントカラー8色が変更前と同じ色で表示されていることを目視確認する

### エッジケース 🟡
- **変数名衝突**: 新規`--radius`, `--border`と既存shadcn用トークンの重複定義（後勝ちでCSSが上書きされ、意図しない値になるリスク）→ リネームで回避する必要がある（🟡推測、資料に明記なし）
- **`@theme inline`との不整合**: `--color-bg-base`等（`@theme inline`内）は本タスクでは`--bg-base`等の旧トークンを引き続き参照するが、`:root`側で`--bg-base`自体を削除・置換すると、`@theme inline`のマッピングが未定義変数を参照し、Tailwindユーティリティ（`bg-bg-base`等）が壊れる可能性がある 🔴（資料に明記なし。TASK-0001.mdは「`@theme inline`ブロックの調整はTASK-0002」としているが、`:root`側の`--bg-base`等を削除した場合に本タスク時点で一時的に未定義参照が発生するかどうかは要確認・要判断）
- **フォント読み込み失敗**: Google Fonts CDNへのネットワークアクセスが失敗した場合のフォールバック（`--font-ui`のフォールバックは`-apple-system, sans-serif`等で担保）🟡

### エラーケース 🟡
- CSS変数の構文誤り（重複プロパティ名、閉じ括弧不足等）によるビルド失敗 → `yarn build`のエラー出力で検知
- `yarn build`は通るがTailwindユーティリティのクラス名解決に失敗し実行時に意図しないスタイルが適用される場合 → `yarn dev`の目視確認、既存コンポーネントテストの回帰確認で検知

- **参照したEARS要件**: REQ-001受け入れ基準（全体一覧・詳細・設定の各画面でダーク背景・単一アクセント色が表示される）
- **参照した設計文書**: `docs/tasks/frontend-ui-compliance/TASK-0001.md`統合テスト要件節、`docs/design/frontend-ui-compliance/dataflow.md`（該当する場合）

---

## 5. EARS要件・設計文書との対応関係

- **参照したユーザストーリー**: MediaVaultユーザーとして、モックアップ準拠のダークUI（暗い背景・単一アクセント色）でアプリを閲覧したい、というストーリー（requirements.md該当ユーザーストーリー節、要確認）
- **参照した機能要件**: REQ-001（デザイントークンの`_shared.css`準拠再定義）, REQ-002（media_type別アクセントカラーの維持）, REQ-402（shadcn由来oklchトークンの上書き方針、値定義部分のみ本タスク対象）
- **参照した非機能要件**: アクセシビリティ（WCAG 2.1 AA コントラスト比）, 互換性（既存技術スタック維持）
- **参照したEdgeケース**: 変数名衝突（`--radius`, `--border`）、`@theme inline`との未定義参照リスク（🔴要確認事項として明記）
- **参照した受け入れ基準**: `docs/tasks/frontend-ui-compliance/TASK-0001.md`「完了条件」全8項目（トークン値の置換完了、アクセント色追加、ステータス色追加、media_type色維持、フォント導入、レイアウト/角丸トークン追加、`yarn build`成功、目視でのダーク背景確認）
- **参照した設計文書**:
  - **アーキテクチャ**: `docs/design/frontend-ui-compliance/architecture.md`「デザイントークン層」節
  - **データフロー**: `docs/design/frontend-ui-compliance/dataflow.md`（該当する場合、CSS変数配信フロー）
  - **型定義**: 該当なし（本タスクはCSS変数のみでTypeScript型定義の変更を伴わない）
  - **データベース**: 該当なし
  - **API仕様**: 該当なし
  - **値の出典**: `docs/frontend/ui/_shared.css`（`:root`定義全体、行8-38）

---

## 品質判定

- **要件の曖昧さ**: 一部あり（🟡🔴項目：フォント導入手段、`--radius`/`--border`の名前衝突対応、`@theme inline`との未定義参照リスクは資料に明記がなく実装時の判断が必要）
- **入出力定義**: 完全（置換前後のトークン名・値は`_shared.css`実測値およびTASK-0001.mdの完了条件・コード例から一意に確定できる）
- **制約条件**: 概ね明確（スコープ制約・互換性制約は🔵で明確。名前衝突対応の具体的手法のみ🟡🔴）
- **実装可能性**: 確実（CSS変数の追加・置換という単純な変更であり、既存の技術スタックで実装可能）
- **信頼性レベル分布**: 🔵が主体（値・スコープ・維持方針は資料に明記）。🟡🔴はフォント導入手段・変数名衝突対応・`@theme inline`との整合性確認という実装判断が必要な3点に集中

**総合評価**: ⚠️ 要改善（実装可能ではあるが、`--border`変数名衝突と`@theme inline`未定義参照リスクの2点は次工程（テストケース洗い出し・実装）で具体的な対応方針を確定する必要がある）

---

## 次のお勧めステップ

`/tsumiki:tdd-testcases frontend-ui-compliance TASK-0001` でテストケースの洗い出しを行います。
</content>
</invoke>
