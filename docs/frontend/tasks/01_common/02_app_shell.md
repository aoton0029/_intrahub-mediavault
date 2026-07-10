# 02. App Shell（Sidebar / Titlebar / ThemeToggle / ルーティング骨格）

対応: 設計書 §1, §5-1

依存: [01_foundation.md](01_foundation.md) 完了後に着手。

## 前提ファイル

- 参照: `docs/frontend/ui/_shared.css`, `docs/frontend/ui/_shared.js`, `docs/frontend/ui/01_home.html`
- 出力: `frontend/src/components/layout/AppShell.tsx`, `Sidebar.tsx`, `Titlebar.tsx`, `ThemeToggle.tsx`, `frontend/src/hooks/useTheme.ts`, `frontend/src/config/navigation.tsx`, `frontend/src/routes.tsx`
- 参照範囲はこの節に列挙したファイルとそこから直接importされるファイルに限る。それ以外のファイルを探すための横断的な探索は行わない。

## タスク一覧

- [x] `useTheme()` フックを実装する（`useState`初期値を`localStorage['mediavault-theme']`から読込 → `useEffect`で`<html data-theme>`に反映、`toggle()`を公開）
- [x] `ThemeToggle` を実装し `useTheme().toggle` を呼ぶだけの薄いコンポーネントにする（`.theme-toggle` 相当）
- [x] `Brand` サブコンポーネント（`.brand` = `.dot` + アプリ名）を実装する
- [x] `NavSection` / `NavItem` を実装する（`active` / `count` / `indent` props、`.count`バッジ対応）
- [x] `frontend/src/config/navigation.tsx` にナビ項目定義（ラベル・アイコン・ルートパス・件数取得元）を集約する
- [x] `Sidebar` を実装し `Brand` + `NavSection[]` + `ThemeToggle` を組み立てる
- [x] `Titlebar` を実装する（`Breadcrumb` + `h1` + `actions` スロット、sticky指定）
- [x] `AppShell` を実装する（`.app-shell` grid、`Sidebar` + `main > Titlebar + .content`、`react-router-dom` v7 の `<Outlet>` を `.content` 内に配置）
- [x] `frontend/src/routes.tsx` に `AppShell` をレイアウトルートとするルーティング骨格を定義する（個別画面ルートは各画面タスクで追加する前提のプレースホルダで可）
- [x] `frontend/src/main.tsx` を `RouterProvider` + `QueryClientProvider` + `Toaster` を用いて再構成する（`App.tsx`が参照する`./routes`を実体化する）

## テストリスト

- [x] `useTheme.test.tsx`: 初期値がlocalStorage未設定時にダーク（デフォルト）になること
- [x] `useTheme.test.tsx`: `toggle()`呼び出しでlocalStorageと`data-theme`属性が更新されること
- [x] `AppShell.test.tsx`: `Sidebar` / `Titlebar` / `.content`（Outlet）がそれぞれレンダリングされること
- [x] `AppShell.test.tsx`: アクティブな`NavItem`に`active`スタイル相当のクラス/aria属性が付与されること
- [x] Titlebarの`actions`スロットに渡した要素が描画されること

> Codexメモ: `AppShell` は route `handle` から title/breadcrumbs/actions を読める形にしつつ、直接 props でも上書きできるようにした。
