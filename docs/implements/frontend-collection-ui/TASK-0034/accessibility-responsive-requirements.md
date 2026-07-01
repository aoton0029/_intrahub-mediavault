# TDD要件定義書: アクセシビリティ・レスポンシブ対応 (TASK-0034)

**機能名**: accessibility-responsive（アクセシビリティ・レスポンシブ対応）
**タスクID**: TASK-0034
**要件名**: frontend-collection-ui
**タスクタイプ**: TDD
**フェーズ**: Phase 6 - 統合・品質保証

## 信頼性レベル指示

- 🔵 **青信号**: EARS要件定義書・設計文書を参考にしてほぼ推測していない
- 🟡 **黄信号**: EARS要件定義書・設計文書から妥当な推測
- 🔴 **赤信号**: EARS要件定義書・設計文書にない推測

> 注記: 本タスク専用のタスクノート（`note.md`）は未生成のため、`docs/tasks/frontend-collection-ui/TASK-0034.md`、`docs/spec/frontend-collection-ui/requirements.md`（NFR-202）、および対象ソースの実地調査結果を一次情報として本書を作成した。

---

## 1. 機能の概要（EARS要件定義書・設計文書ベース）

- 🔵 **何をする機能か**: Phase2〜5で実装済みの全13画面を対象に、基本的な **WCAG 2.1 AA準拠**（フォームのラベル付け・キーボード操作・コントラスト比）を確認・修正し、加えてモバイル幅（375px〜768px想定）でのレイアウト崩れを確認・修正する。*出典: TASK-0034.md「タスク概要」、NFR-202*
- 🔵 **解決する問題**: 支援技術（スクリーンリーダー）利用者・キーボードのみ操作利用者・低視力利用者・モバイル利用者が、コレクション管理UIを問題なく操作できない状態を解消する。*出典: NFR-202*
- 🟡 **想定ユーザー**: 単一ユーザー前提（REQ-401）のもと、当該ユーザーが多様な入力手段・画面幅・視環境で利用するケースを想定。*妥当な推測*
- 🔵 **システム内での位置づけ**: 個別画面実装（Phase2〜5）の完了後に横断的に品質を担保する統合・品質保証タスク。E2E（TASK-0035）の前提となる。*出典: TASK-0034.md 依存タスク*
- **参照したEARS要件**: NFR-202（主）、NFR-201、REQ-403、REQ-401
- **参照した設計文書**: `docs/design/frontend-collection-ui/architecture.md`, `interfaces.ts`

---

## 2. 入力・出力の仕様

本タスクはUI品質担保タスクであり、明確なAPI入出力を持たない。ここでは「検証対象（入力）」と「期待される確認可能な属性・挙動（出力）」を定義する。

### 対象コンポーネント（入力） 🔵

| # | 対象 | 実ファイルパス | 備考 |
|---|---|---|---|
| 1 | 手動追加・編集フォーム | `frontend/src/pages/ItemFormPage.tsx` | shadcn/ui `Form`(`FormLabel`/`FormControl`)使用済み |
| 2 | APIキー登録・インポートタブ | `frontend/src/pages/SettingsPage.tsx` | REQ-403関連 |
| 3 | 検索フォーム | `frontend/src/pages/SearchAddPage.tsx` | — |
| 4 | サイドバーナビ | `frontend/src/components/common/Sidebar.tsx` | ⚠️ タスク記載の`components/layout/Sidebar.tsx`は誤り。実体は`common/`配下 |
| 5 | フィルタUI | `frontend/src/components/common/FilterBar.tsx` | 既に`label htmlFor`/`id`紐付け済み |
| 6 | 各一覧画面のカード | `frontend/src/pages/{General,Academic,Paper}ListPage.tsx` 等 | カードリンクのフォーカス可能性 |

### 期待される確認可能属性・挙動（出力） 🔵🟡

- 🔵 各入力フィールドが `getByLabelText` 相当で取得可能（`label htmlFor`/`id`、`aria-labelledby`、または `aria-label` が機能）
- 🟡 バリデーションエラー時、フィールドに `aria-describedby` がエラーメッセージ要素の `id` を指す（shadcn/ui `Form`標準機能に依存）
- 🔵 ナビゲーション（サイドバー・ページネーション・タブ・カードリンク）の全インタラクティブ要素が `getByRole('link'|'button')` 相当で取得でき、Tabでフォーカス可能
- 🟡 削除確認 `Dialog` が Escキーで閉じる／フォーカストラップされる（Radix UI標準）
- 🔵 フィルタ各コントロールに `aria-label` またはラベル関連付けが存在
- 🟡 モバイル幅375pxでレイアウトが崩れず、意図しない横スクロールが発生しない

- **参照したEARS要件**: NFR-202, NFR-201
- **参照した設計文書**: `interfaces.ts`（`ItemListFilters`, `MediaType`, `ItemStatus`）

---

## 3. 制約条件（EARS非機能要件・アーキテクチャ設計ベース）

- 🔵 **アクセシビリティ基準**: 基本的なWCAG 2.1 AA準拠（通常テキスト コントラスト比 4.5:1、大テキスト 3:1）。*NFR-202*
- 🔵 **範囲の限定**: 「基本的な」準拠であり、axe-core等によるフル自動スキャン・全項目準拠の保証は対象外。重大な欠落（ラベル未設定・キーボード操作不能）の検出・修正を優先。*TASK-0034.md 注意事項*
- 🟡 **テーマ変更の抑制**: コントラスト調整でCSS変数を変える場合、TASK-0002確定のダークテーマの見た目を大きく変えない最小限の調整に留める。対象は `frontend/src/index.css`。*TASK-0034.md 注意事項*
- 🟡 **既存テスト非破壊**: レスポンシブ修正でDOM構造依存の既存単体テストが壊れないよう、修正後に再実行・確認する。*TASK-0034.md 注意事項*
- 🔵 **技術スタック制約**: React 19 / TypeScript / Tailwind CSS 4 / shadcn/ui（Radix UI基盤）/ Vitest + Testing Library。フォーカストラップ・Escクローズはshadcn/ui `Dialog`標準機能をそのまま利用。*frontend/CLAUDE.md, TASK-0034.md*
- 🔵 **実装方針**: `div`+`onClick`のみのインタラクティブ要素は禁止。ネイティブ `<a>`/`<button>` または `role`+`tabIndex`+`onKeyDown`(Enter/Space) を使用。*TASK-0034.md 実装詳細2*
- 🟡 **モバイルサイドバー**: モバイル幅では shadcn/ui `Sheet`（ドロワー）形式とし、ハンバーガーアイコンに `aria-label="メニューを開く"` を付与。現状 `Sidebar.tsx` にモバイル切替は未実装のため新規追加が必要。*TASK-0034.md UI/UX要件・実地調査*
- 🟡 **ラベルの視覚表示**: プレースホルダのみでラベルを代替せず、ラベルを常時表示。*TASK-0034.md UI/UX要件, NFR-202*

- **参照したEARS要件**: NFR-202, NFR-201, REQ-403, REQ-401
- **参照した設計文書**: `architecture.md`, `docs/design/frontend-collection-ui/dataflow.md`

---

## 4. 想定される使用例（Edgeケース・データフローベース）

### 基本パターン 🔵
1. スクリーンリーダー利用者が手動追加フォームのタイトル入力にフォーカス→ラベルテキストが読み上げられる。
2. キーボード利用者がTab/Shift+Tabでサイドバー→フィルタ→カードリンク→ページネーションを順に移動でき、視覚的順序と一致する。
3. モバイル利用者が375px幅でハンバーガーメニューからサイドバーを開閉し、全13画面を横スクロールなしで閲覧する。

### エッジ・エラーケース 🟡
- バリデーションエラー時、エラーメッセージが `aria-describedby` で該当フィールドに関連付き読み上げられる。
- 削除確認 `Dialog` 表示中、フォーカスがモーダル内にトラップされ、Escで閉じる（背後要素にフォーカスが漏れない）。
- ネイティブ `<input type="file">` 等 shadcn/ui `Form` 非経由の入力に明示的 `<label>`/`aria-label` がない場合の欠落検出。
- タブUI・ページネーションで Tabキーがフォーカス不能な要素（`div`+`onClick`）に到達できないケースの検出。

- **参照したEARS要件**: NFR-202, NFR-201
- **参照した設計文書**: `dataflow.md`

---

## 5. テストケース対応（TASK-0034 単体テスト要件）

| TC | 概要 | 対象ファイル | 信頼性 |
|---|---|---|---|
| TC-1 | 手動追加フォーム入力にラベルが関連付く（`getByLabelText`） | `ItemFormPage.tsx` | 🔵 |
| TC-2 | APIキー登録フォーム入力にラベルが関連付く | `SettingsPage.tsx` | 🔵 |
| TC-3 | サイドバー各リンクがTabフォーカス可能（`getAllByRole('link')`） | `Sidebar.tsx` | 🔵 |
| TC-4 | 削除確認モーダルがEscで閉じる | `Dialog`使用箇所 | 🟡 |
| TC-5 | フィルタUI各コントロールにaria属性／ラベル関連付け | `FilterBar.tsx` | 🔵 |
| TC-6 | エラーメッセージがフィールドに`aria-describedby`で関連付く | `ItemFormPage.tsx` | 🟡 |

---

## 6. EARS要件・設計文書との対応関係

- **参照した機能要件**: REQ-403（APIキーは設定画面UI経由）、REQ-401（単一ユーザー）
- **参照した非機能要件**: **NFR-202（主: WCAG 2.1 AA / ラベル・キーボード・コントラスト）**、NFR-201（フィールド近傍エラー表示）
- **参照した受け入れ基準**: TASK-0034.md「完了条件」8項目、「単体テスト要件」TC-1〜6
- **参照した設計文書**:
  - アーキテクチャ: `architecture.md`
  - データフロー: `dataflow.md`
  - 型定義: `interfaces.ts`（`ItemListFilters` 他）

---

## 品質判定

**判定: ⚠️ 要改善（一部要確認）**

- 要件の曖昧さ: 一部あり。コントラスト比の具体的検証手法・数値基準（🟡）、モバイル実装パターン（🟡）は設計文書に明記なく一般プラクティスからの推測。
- 入出力定義: 本タスクは属性・挙動確認が中心のため、確認可能な属性リストとして概ね完全。
- 制約条件: 明確（テーマ変更抑制・既存テスト非破壊・実装方針が明記）。
- 実装可能性: 確実。
- 信頼性分布: 実装詳細 🔵2/🟡2、テスト 🔵4/🟡2。

**要確認事項**:
1. 対象パスの齟齬: タスク記載の `components/layout/Sidebar.tsx` / `FilterBar.tsx` は実体が `components/common/` 配下。本書では実パスを採用。
2. `Sidebar.tsx` にモバイル用 `Sheet`/ハンバーガーは未実装。TASK-0034で新規追加する前提で良いか要確認。
3. コントラスト比の検証手段（DevTools目視 / 自動ツール）とAA未達時のトークン調整可否の最終方針。
