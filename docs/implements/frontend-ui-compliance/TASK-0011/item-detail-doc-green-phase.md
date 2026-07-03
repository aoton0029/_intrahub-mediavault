# TASK-0011: Greenフェーズ記録

## 実装ファイル

- `frontend/src/pages/ItemDetailPage.tsx`
- `frontend/src/index.css`（`.breadcrumb`/`.btn`/`.btn-danger`/`.doc-cover`/`.doc-title`/`.doc-original`/`.doc-section`/`.titlebar .actions`を追加）

## 実装方針

- `.breadcrumb`にホームLink・`item.mediaType`の日本語ラベル（`MediaTypeBadge.tsx`のマッピングを踏襲）・タイトルを表示。
- `.titlebar`に編集(`.btn`)・削除(`.btn-danger`)ボタンを配置。編集は既存の`/items/:id/edit`遷移をそのまま維持。
- 削除は`useConfirmDialog`＋`ConfirmDialog`（既存実装）と`useDeleteItemMutation`（既存API）を組み合わせて実装。
- `.doc`配下に`.doc-cover`（背景画像はcoverImageUrl設定時のみ）・`.doc-title`・`.doc-original`（条件付き）・`.doc-section`（概要）を実装。
- RootLayoutの`.properties`（TASK-0009実装済み）は変更せず、右カラム空間確保のみ前提とする。

## テスト結果

`yarn vitest run src/pages/ItemDetailPage.test.tsx` → 12件全て成功
`yarn tsc --noEmit` → エラーなし

## 課題・改善点（Refactorフェーズで対応）

- パンくずの`<div />`プレースホルダ（タイトルバー左側）が空要素のまま。モックアップでは左側にbreadcrumb+h1を縦積みする構造だが、既存実装踏襲のため`.breadcrumb`をタイトルバー外に配置している。構造の整理を検討。
- `Button`コンポーネントへの`className="btn"`/`className="btn btn-danger"`のべた書きが重複気味。
