# frontend-collection-ui コンテキストノート

## 技術スタック
- React 18.3+ / TypeScript 5.7+ / Vite 6
- TanStack Query 5（サーバー状態） + React内蔵state/useContext（UI状態）
- React Router v7（URLクエリパラメータで一覧フィルタ状態を保持）
- Tailwind CSS 4 + shadcn/ui
- pnpm / Vitest + Testing Library / Playwright / ESLint + Prettier

参照: [docs/frontend/tech-stack.md](../../frontend/tech-stack.md)

## 実装状況
- `frontend/` は `yarn create vite . --template react-ts` 直後のスケルトンのみで、画面・コンポーネントは未実装（2026-06-22時点）。

## 既存デザインシステム
- `docs/frontend/ui/01_components.html` にダークテーマのカラートークン・コンポーネントカタログが存在（`--bg-base`, `--accent`等のCSS変数、media_type別アクセントカラー: anime/movie/manga/novel/game/drama/book/paper）。
- このトークン・コンポーネント構成をTailwind 4 + shadcn/uiで再現する前提とする。

## 連携バックエンドAPI仕様（既存・確定済み）
- [docs/design/mediavault-backend/api-endpoints.md](../../design/mediavault-backend/api-endpoints.md)
- [docs/design/mediavault-backend/types.rs](../../design/mediavault-backend/types.rs)
- [docs/design/mediavault-backend/database-schema.sql](../../design/mediavault-backend/database-schema.sql)
- [docs/spec/mediavault-backend/requirements.md](../mediavault-backend/requirements.md)
- ベースURL: `http://localhost:8080/api/v1`（🟡推測）。内部API（`/internal/*`）はフロントエンドからは使用しない（巡回バッチ・ファイルサーバー監視プロセス専用）。
- 主要エンドポイント: `GET/POST/PATCH/DELETE /items`, `GET /items/search`（外部API検索）, `POST /items/import`, `POST/GET /items/:id/groups`, `POST/GET /groups/:group_id/episodes`, `POST/DELETE /tags`, `/categories`, `/mylists`, `/item-relations`, `/staff`, `/items/:id/staff`, `/items/:id/links`, `/items/:id/files`, `/items/:id/files/upload`, `/items/:id/trailers`, `/import/booklog`, `/import/steam`, `PUT /settings/api-keys/:provider`。

## 注意事項・制約
- 単一ユーザー前提。認証・ログインUIは持たない（REQ-401相当）。
- エクスポート機能（Obsidian/Notion）はバックエンド側で次回フェーズ対象外と確認済み。フロントエンドも今回は未実装ボタンのみ表示。
- インポート（ブクログCSV・Steam）・APIキー管理は今回スコープに含める。
- ファイルアップロードはドラッグ&ドロップ/ファイル選択（`POST /items/:id/files/upload`）を主方式とする。パス直接指定（`POST /items/:id/files`）は内部API/バッチ向けのためフロントエンドUIでは扱わない。
