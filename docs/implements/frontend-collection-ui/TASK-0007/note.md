# TASK-0007: グローバルナビゲーション実装 開発コンテキストノート

## 1. 技術スタック

- React 19.2.6 / TypeScript ~6.0.2 / Vite
- react-router-dom v7（`NavLink`の`className`関数で`isActive`に応じたスタイル切り替えが可能）
- Tailwind CSS v4.3.2（`@tailwindcss/vite`、専用design tokenファイルは未検出。v4既定値＋shadcn/uiのCSS変数ベースクラスを使用）
- UIライブラリ: radix-ui 1.6.0 ベースの shadcn/ui（事前インストール済み）
- スタイリングユーティリティ: clsx 2.1.1 / class-variance-authority 0.7.1 / tailwind-merge 3.6.0（`@/lib/utils`の`cn()`を使用）
- アイコン: lucide-react 1.22.0
- テスト: Vitest 4.1.9 / @testing-library/react 16.3.2 / Playwright 1.61.1
- 参照元: frontend/package.json, frontend/CLAUDE.md

## 2. 開発ルール

- テストコマンド: `yarn test`（ユニット, Vitest）/ `yarn test:watch` / `yarn test:e2e`（Playwright）
- ビルド: `yarn build`（型チェック含む）/ Lint: `yarn lint`
- shadcn/uiコンポーネント追加: `npx shadcn@latest add <component-name>`
- 参照元: frontend/CLAUDE.md

## 3. 関連実装（既存コンポーネントパターン）

- ディレクトリ: frontend/src/components/common/（MediaCard, MediaTypeBadge, FilterBar, EmptyState, ConfirmDialogが`.tsx`+`.test.tsx`ペアで存在）、frontend/src/components/ui/（button, badge, dialog等のshadcn/uiコンポーネント）
- 命名・実装パターン:
  - **named export**（default exportではない）
  - Propsインターフェースをコンポーネント直上に定義
  - `cn()`ユーティリティ（frontend/src/lib/utils）でTailwindクラスを合成
  - `data-testid`属性をテスト用に付与
  - JSDocコメント形式: `【機能概要】`, `【実装方針】`, `【テスト対応】`
- 既存のSidebar/Navigationコンポーネントは未実装（本タスクはグリーンフィールド）
- 参照元: frontend/src/components/common/EmptyState.tsx, frontend/src/components/common/*.tsx

## 4. 設計文書

- ルーティング実体: frontend/src/routes.tsx（TASK-0003実装済み）
  - `/`配下に`RootLayout`がネストされ、子ルートとして `/`(HomePage), `/collections/general`, `/collections/academic`, `/collections/paper`, `/items/:id`, `/items/:id/edit`, `/items/new/{general|academic|paper}`, `/search/{general|academic|paper}`, `/mylists`, `/tags-categories`, `/staff`, `/settings` が定義済み
- レイアウト統合先: frontend/src/pages/RootLayout.tsx（現状`<Outlet />`のみのプレースホルダー。本タスクでSidebarと組み合わせる）
- ナビゲーション項目とパス対応表（確定済み、docs/implements/frontend-collection-ui/TASK-0007/global-navigation-requirements.md より）:

| label | to |
|---|---|
| 全体一覧 | `/` |
| 一般メディア | `/collections/general` |
| 学術書・専門書 | `/collections/academic` |
| 論文・文献 | `/collections/paper` |
| マイリスト | `/mylists` |
| タグ/カテゴリ | `/tags-categories` |
| スタッフ | `/staff` |
| 設定 | `/settings` |

- 配置方針: `src/components/common/Sidebar.tsx`（🟡 architecture.mdの「common/＝複数画面で再利用する独自コンポーネント」方針からの推測）
- 完了条件: `NavLink`使用、`isActive`に基づくアクティブ状態の視覚的識別、単体テスト全パス
- 参照元: docs/tasks/frontend-collection-ui/TASK-0007.md, docs/implements/frontend-collection-ui/TASK-0007/global-navigation-requirements.md, frontend/src/routes.tsx, frontend/src/pages/RootLayout.tsx

## 5. テスト関連情報

- テストファイル配置: 実装ファイルと同階層に `ComponentName.test.tsx`（例: frontend/src/components/common/EmptyState.test.tsx）
- セットアップ: frontend/src/test/setup.ts（`@testing-library/jest-dom/vitest`をimport）
- Vitest設定: frontend/vitest.config.ts（environment: jsdom, path alias `@` → `./src`, setupFiles: `./src/test/setup.ts`, `globals: true`）
- テスト構造パターン（EmptyState.test.tsxより）:
  - `describe('ComponentName')`ブロック
  - 各テストにJSDocコメント: `【テスト目的】`, `【テスト内容】`, `【期待される動作】`, `【テストデータ準備】`
  - `render(<Component />)`, `screen.getByText()`, `screen.getByRole()`, `fireEvent.click()` / `userEvent`
  - `vi.fn()`でモック
  - 正常系・異常系・境界値で整理
- 本タスク向け想定: `MemoryRouter`（react-router-dom v7）でラップしてレンダリング・遷移・アクティブ状態をテストする（TASK-0007.md単体テスト要件のテストケース1〜3）
- 参照元: frontend/vitest.config.ts, frontend/src/test/setup.ts, frontend/src/components/common/EmptyState.test.tsx

## 6. 注意事項

- Tailwind v4には専用design tokenファイルが見つからず、既存コンポーネントは`text-muted-foreground`, `border-border`, `bg-background`等のshadcn/ui CSS変数ベースクラスを使用している。アクティブ状態の視覚仕様は設計文書に明記がないため、これらの既存トークンに準拠した実装とする（🟡推測）。
- `/items/:id`等の動的パラメータルートはナビゲーション対象外（navItemsに含めない）。
- 依存タスクTASK-0003（ディレクトリ構造とルーティング基盤）は完了済み。RootLayoutとroutes.tsxはそのまま利用可能。
- 参照元: docs/implements/frontend-collection-ui/TASK-0007/global-navigation-requirements.md
