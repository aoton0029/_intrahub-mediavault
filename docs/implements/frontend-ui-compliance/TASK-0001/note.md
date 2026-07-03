# TASK-0001 開発コンテキストノート: デザイントークンの再定義（index.css）

## 1. 技術スタック

- React 18.3+ / TypeScript / Vite 6 / Tailwind CSS v4 + shadcn/ui / TanStack Query v5 / react-router-dom v7
- CSSはTailwind v4の`@import "tailwindcss"`方式（`@theme inline`でCSS変数をユーティリティにマッピング）
- 既存フォントは`@fontsource-variable/geist`（npmパッケージ経由）
- 参照元: frontend/CLAUDE.md, frontend/src/index.css, frontend/package.json

## 2. 開発ルール

- テストコマンド: `yarn test`（Vitest run）, `yarn test:watch`, `yarn test:e2e`（Playwright）
- ビルド: `yarn build`（`tsc -b && vite build`、型チェック含む）
- Lint: `yarn lint`
- Dockerビルド確認手順あり（TASK-0001/0002向け、nginx配信）: `docker build -f frontend/Dockerfile -t mediavault-frontend-test frontend` 等
- `package.json`に`resolutions.vite`設定あり。vite関連のバージョン変更時は`yarn install`でロックファイル再生成し`docker build`で確認要（本タスクはCSSのみのため影響小）
- AGENTS.md、`docs/rule/`ディレクトリは本プロジェクトに存在しない（追加ルールなし）
- 参照元: frontend/CLAUDE.md

## 3. 関連実装

- 変更対象: `frontend/src/index.css`の`:root`ブロック（旧トークン定義部分）
- 値の出典（モックアップ共通スタイル）: `docs/frontend/ui/_shared.css`
  - 新トークン一式（色・フォント・レイアウト）がこのファイルの`:root`にすべて定義済み
  - Google Fonts CDN `@import`でInter / Source Serif 4 / JetBrains Monoを導入している実例あり
- 既存の`@theme inline`ブロック（`--color-bg-base`等の旧トークン名マッピング）は本タスクでは変更せず、TASK-0002で対応予定（タスク定義に明記）
- 参照元: frontend/src/index.css, docs/frontend/ui/_shared.css, docs/tasks/frontend-ui-compliance/TASK-0001.md

## 4. 設計文書

- 要件定義: `docs/spec/frontend-ui-compliance/requirements.md`
  - REQ-001: `index.css`のデザイントークンを`_shared.css`準拠に再定義
  - REQ-002: media_type別アクセントカラー8色は変更せず維持
  - REQ-402: shadcn由来の`oklch`系トークンは`_shared.css`の対応色に上書き（ただし本タスクでは`:root`の値定義のみに留め、`@theme inline`との連携調整はTASK-0002）
- アーキテクチャ設計: `docs/design/frontend-ui-compliance/architecture.md`
  - 「デザイントークン層」節: `:root`の旧トークンを`_shared.css`の値に直接置換する方針（`_shared.css`を別ファイルとして二重管理しない）
  - Tailwind連携（`@theme`経由のマッピング）は別レイヤーとして記載されるがTASK-0002以降の範囲
- タスク定義: `docs/tasks/frontend-ui-compliance/TASK-0001.md`
  - 完了条件・置換前後のトークン値・フォント導入方法・レイアウト/角丸トークンの詳細実装例が記載済み
  - 既存`--radius: 0.625rem`（shadcn用）は名前衝突回避のため別名（例: `--radius-shadcn`）へのリネームが必要（🟡推測）
- 参照元: docs/spec/frontend-ui-compliance/requirements.md, docs/design/frontend-ui-compliance/architecture.md, docs/tasks/frontend-ui-compliance/TASK-0001.md

## 5. テスト関連情報

- テストフレームワーク設定: `frontend/vitest.config.ts`（ユニット/コンポーネントテスト）、`frontend/playwright.config.ts`（E2E）
- テストセットアップ: `frontend/src/test/setup.ts`
- 既存テストファイルの配置パターン: 対象コンポーネント/モジュールと同ディレクトリに`*.test.ts(x)`として配置
  - 例: `frontend/src/components/common/MediaCard.test.tsx`, `frontend/src/components/common/Sidebar.test.tsx`, `frontend/src/components/common/FilterBar.test.tsx`
  - a11yテストは`*.test.a11y.tsx`という命名で分離（例: `frontend/src/components/common/FilterBar.test.a11y.tsx`, `frontend/src/components/common/Sidebar.test.a11y.tsx`）
  - ページ単位のテストは`frontend/src/pages/*.test.tsx`（例: `HomePage.test.tsx`, `GeneralListPage.test.tsx`）
  - APIフックのテストは`frontend/src/api/*.test.ts`
- タスク定義（TASK-0001.md）記載の単体テスト方針: デザイントークン（CSS変数）自体は既存テストフレームワークで直接アサーションすることが困難なため、**本タスクでは新規の単体テストコードは作成しない方針**。既存コンポーネントテストが参照するTailwindユーティリティクラス名が変更後も同一であることの回帰確認のみ行う
- 統合テスト要件: `yarn build`がエラーなく完了すること、`yarn dev`でHomePage等の背景色が目視で`#1e1e1e`系になっていること、media_type別アクセントカラーに巻き込み事故がないことの目視確認
- 参照元: frontend/vitest.config.ts, frontend/playwright.config.ts, frontend/src/test/setup.ts, docs/tasks/frontend-ui-compliance/TASK-0001.md

## 6. 注意事項

- **技術的制約**: React/TypeScript/Vite/Tailwind CSS 4 + shadcn/uiの既存構成を維持し、追加ライブラリは導入しない（互換性制約、architecture.md）
- **スコープ制約**: 本タスクでは`:root`の値定義のみを対象とし、`@theme inline`ブロックのマッピング名変更（Tailwindユーティリティ・shadcnトークン連携）はTASK-0002で扱う（TASK-0001.mdに明記）
- **既存`--radius`の共存**: shadcn用途の既存`--radius: 0.625rem`は新規`--radius: 6px`と名前が衝突するため、別名（例: `--radius-shadcn`）へのリネームが必要（🟡推測、資料に明記なし）
- **media_type別アクセントカラー**（`--accent-anime`等8色）は絶対に変更しない（REQ-002、既存実装のまま維持）
- **アクセシビリティ**: 色変更後もコントラスト比がWCAG 2.1 AA基準を満たすこと（tech-stack.md品質基準、architecture.md互換性制約）
- **フォント導入手段**: 既存は`@fontsource-variable/geist`（npmパッケージ経由）だが、本タスクでは`_shared.css`と同様にGoogle Fonts CDN `@import`方式を採用する方針（🟡推測、既存方式との不整合は許容と判断）
- **単体テストを新規作成しない方針**であるため、実装完了確認は既存`vitest run`の全パス確認とビルド・目視確認が中心になる
- 参照元: docs/design/frontend-ui-compliance/architecture.md, docs/tasks/frontend-ui-compliance/TASK-0001.md, frontend/CLAUDE.md
