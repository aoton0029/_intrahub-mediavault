# 01. フェーズ0: 基盤構築

依存: なし（最初に着手）
参照: [design/00_common.md](../design/00_common.md) §1, §2, §5-1

## タスク一覧

- [ ] **トークン定義**: `frontend/src/index.css`（または新規グローバルCSS）に共通設計 §2 の `@theme` ブロックと `:root[data-theme="light"]` オーバーライドを追加する
  - 参照: [00_common.md §2](../design/00_common.md#2-tailwind-v4-theme-トークン対応表)
  - 完了条件: `bg-bg-app` / `text-text-primary` / `border-border-soft` 等のTailwindユーティリティが生成され、コンポーネントから利用できる
  - 【要確認】ライトモードは `prefers-color-scheme` ではなく `data-theme` 属性の明示的トグルで実現する方針（`dark:`バリアントは使わない）。詳細は [05_open_questions.md](05_open_questions.md) 参照

- [ ] **AppShellの実装**: `.app-shell`（grid: sidebar + main）に対応するコンポーネントを作成
  - 構成: `<AppShell><Sidebar/><main className="main"><Titlebar/><div className="content">{children}</div></main></AppShell>`
  - 参照: [00_common.md §1](../design/00_common.md#1-アプリシェル構成)
  - `AppShell` は React Router v7 の共通レイアウトルートとして実装（`<Outlet>` を content 内に配置）

- [ ] **Sidebarの実装**: `<Brand/>`（`.dot` + アプリ名）、`<NavSection label>` + `<NavItem active? count? indent?>`、`<ThemeToggle/>` を実装
  - ナビ項目・件数（`.count`）は全画面共通のため、ここで確定させる

- [ ] **Titlebarの実装**: `<Breadcrumb/>` + `<h1/>` + アクション領域（例:「編集する」`btn-accent`）を実装
  - `.titlebar` は sticky

- [ ] **useTheme() フックの実装**
  - `useState` 初期値を `localStorage['mediavault-theme']` から読込
  - `useEffect` で `<html data-theme>` に反映
  - 参照: [00_common.md §5-1](../design/00_common.md#5-インタラクション-react状態への変換方針)

- [ ] **ThemeToggle コンポーネントの実装**: `useTheme()` の `toggle()` を呼ぶのみの薄いコンポーネントにする

- [ ] **ルーティング骨格の実装**: React Router v7 で `AppShell` をレイアウトルートとし、18画面ぶんの空ルート（プレースホルダ）を用意する
  - 各ルートのpathは対応する画面設計書のURL想定に合わせる（各画面設計書の「ルーティング」記載を参照。未記載の場合は画面実装時に確定し、本タスクへ差し戻す）

## 完了条件

- `npm run dev` で `AppShell` が表示され、サイドバーのテーマ切替ボタンでライト/ダークが切り替わり `localStorage` に永続化される
- 空のルートに対してブレッドクラムとタイトルが表示される
