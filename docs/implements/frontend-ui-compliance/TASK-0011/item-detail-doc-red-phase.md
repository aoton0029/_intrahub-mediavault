# TASK-0011: Redフェーズ記録

## 対象テストファイル

`frontend/src/pages/ItemDetailPage.test.tsx`

## 実行結果

`yarn vitest run src/pages/ItemDetailPage.test.tsx`

- 12件中 9件失敗、3件成功
- 失敗（Greenフェーズで実装すべき内容）:
  - TC-IDP-N-01: `.breadcrumb`要素が存在しない
  - TC-IDP-N-02: ホームへのLinkがない
  - TC-IDP-N-03: 編集ボタンが`.btn`クラスでない（現状Tailwindのみ）
  - TC-IDP-N-04: 削除ボタン自体が存在しない
  - TC-IDP-N-05: 削除ボタンクリックでconfirmが呼ばれない（削除ボタン未実装）
  - TC-IDP-N-06: 削除確定でmutateが呼ばれない（削除フロー未実装）
  - TC-IDP-N-07: `.doc-title`クラスが付与されていない
  - TC-IDP-N-08: `.doc-original`クラスが付与されていない
  - TC-IDP-N-09: `.doc-section`要素が存在しない
  - TC-IDP-N-10: `.doc-cover`要素が存在しない
- 成功（既存実装が偶然満たしていたもの）:
  - TC-IDP-B-01: originalTitle未設定時は非表示（既存の条件付きレンダリングが機能）
  - TC-IDP-E-01: ITEM_NOT_FOUND時のリダイレクト（既存useEffectロジックが機能）

## Greenフェーズで実装すべき内容

1. `.breadcrumb`ナビゲーション（ホームLink・mediaTypeラベル・タイトル）の追加
2. タイトルバーを`.titlebar`構造に変更し、`.btn`（編集）・`.btn-danger`（削除）ボタンを配置
3. `useConfirmDialog`・`useDeleteItemMutation`・`ConfirmDialog`を組み込み、削除フローを実装
4. `.doc-cover`/`.doc-title`/`.doc-original`/`.doc-section`のマークアップに変更
5. `frontend/src/index.css`に`.breadcrumb`/`.btn`/`.btn-danger`/`.doc-cover`/`.doc-title`/`.doc-original`/`.doc-section`のスタイルを追加（`_shared.css`準拠）
