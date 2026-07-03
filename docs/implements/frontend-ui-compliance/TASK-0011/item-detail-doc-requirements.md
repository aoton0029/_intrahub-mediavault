# TASK-0011: ItemDetailPageのパンくず・タイトルバー・ドキュメント本文 要件定義

## 1. 機能の概要

- 🔵 `frontend/src/pages/ItemDetailPage.tsx` にモックアップ（`docs/frontend/ui/02_item_detail.html`）準拠のパンくずリスト（`.breadcrumb`）、タイトルバー右の「編集」「削除」ボタン（`.btn`/`.btn-danger`）、ドキュメント本文（`.doc-cover`/`.doc-title`/`.doc-original`/`.doc-section`）を実装する。
- 🔵 現状の `ItemDetailPage.tsx` は Tailwind ユーティリティクラスのみで構成されており、モックアップのクラス名（`.breadcrumb` 等）が未実装。
- 🔵 想定ユーザー: MediaVaultでアイテム詳細を閲覧・編集・削除するユーザー。
- 🔵 システム内での位置づけ: `RootLayout`（TASK-0009で`.app-shell.has-properties`実装済み）配下の`main`にレンダリングされる画面コンポーネント。
- **参照したEARS要件**: REQ-007
- **参照した設計文書**: architecture.md「画面別スタイル適用」表、TASK-0009.md（`.app-shell.has-properties`）

## 2. 入力・出力の仕様

- 🔵 入力: URLパラメータ`id`（`useParams`）。`useItemQuery(id)`で取得する`Item`型データ（`frontend/src/types/index.ts`）。
  - `item.title: string`, `item.originalTitle?: string`, `item.description?: string`, `item.coverImageUrl?: string`
  - 🟡 `item`にはカテゴリ専用フィールドが存在しないため、パンくずの「カテゴリ」相当ラベルは`item.mediaType`（例: `anime`）を日本語ラベルに変換して使用する（既存の`Sidebar`ナビゲーション構造がmediaType単位のカテゴリ分けであることに基づく妥当な推測）。
- 🔵 出力: パンくず（ホーム › mediaTypeラベル › タイトル）、タイトルバー（編集/削除ボタン）、ドキュメント本文（カバー・タイトル・原題・概要）のJSX。
- 🔵 データフロー: `useItemQuery` → `item` → 各表示要素へマッピング。削除は既存`useDeleteItemMutation`（`frontend/src/api/items.ts:147`）を使用。
- **参照したEARS要件**: REQ-007
- **参照した設計文書**: `frontend/src/types/index.ts`（`ItemBase`/`Item`）, `frontend/src/api/items.ts`（`useItemQuery`, `useDeleteItemMutation`）

## 3. 制約条件

- 🔵 既存の削除確認ダイアログ`useConfirmDialog`＋`ConfirmDialog`（`frontend/src/hooks/useConfirmDialog.ts`, `frontend/src/components/common/ConfirmDialog.tsx`）をそのまま利用し、削除ボタン押下時にダイアログを開き、確定時に`useDeleteItemMutation`を呼び出す。
- 🔵 Propertiesパネルの中身は対象外。TASK-0009で実装済みの`RootLayout`の`.properties`（空div）をそのまま利用し、本タスクでは変更しない。
- 🔵 `.btn`/`.btn-danger`/`.breadcrumb`/`.doc-cover`/`.doc-title`/`.doc-original`/`.doc-section`/`.titlebar`は`frontend/src/index.css`に既存定義があるものを再利用する（`.titlebar`は既にTASK-0010で定義済み、`.btn`系・`.breadcrumb`・`.doc-*`は本タスクで未定義のため`_shared.css`を参照し追加が必要）。
- 🟡 編集ボタンは既存の`/items/:id/edit`遷移（`Link to`）をそのまま踏襲する。
- **参照したEARS要件**: REQ-007, REQ-006（3ペインApp Shell）
- **参照した設計文書**: `docs/frontend/ui/_shared.css`（158, 371-392, 518-547行目）, TASK-0009.md

## 4. 想定される使用例

- 🔵 基本パターン: 一覧画面からアイテムをクリック → 詳細画面遷移 → パンくず・タイトルバー・本文が表示される。
- 🔵 編集ボタン押下 → `/items/:id/edit`へ遷移。
- 🔵 削除ボタン押下 → `ConfirmDialog`が開く → 確定 → `useDeleteItemMutation`実行 → 一覧へ遷移。
- 🟡 `item.originalTitle`未設定時は`.doc-original`を表示しない（既存実装の条件付きレンダリングパターンを踏襲）。
- 🟡 `item.coverImageUrl`未設定時は`.doc-cover`をプレースホルダ表示のまま（背景画像なし）とする。
- **参照したEARS要件**: REQ-007
- **参照した設計文書**: 02_item_detail.html

## 5. EARS要件・設計文書との対応関係

- **参照した機能要件**: REQ-007
- **参照した設計文書**:
  - **アーキテクチャ**: architecture.md「画面別スタイル適用」
  - **型定義**: `frontend/src/types/index.ts` `Item`/`ItemBase`
  - **既存実装**: `frontend/src/pages/ItemDetailPage.tsx`, `frontend/src/api/items.ts`, `frontend/src/hooks/useConfirmDialog.ts`, `frontend/src/components/common/ConfirmDialog.tsx`
  - **モックアップ**: `docs/frontend/ui/02_item_detail.html`, `docs/frontend/ui/_shared.css`

## 品質判定

- 要件の曖昧さ: カテゴリラベルの導出（mediaType使用）と削除フローの結線は🟡だが、既存コードとの整合性が取れており実装可能。
- 入出力定義: 🔵 既存の`Item`型・APIフックを流用するため完全。
- 制約条件: 🔵 明確（既存ConfirmDialog/RootLayoutを変更しない）。
- 実装可能性: 確実。
- 信頼性レベル分布: 🔵多数、🟡少数。**総合: 高品質**
