# backend-frontend-integration コンテキストノート

**作成日**: 2026-07-02

## 技術スタック

### バックエンド（[docs/backend/tech-stack.md](../../backend/tech-stack.md)）
- Rust（最新stable）/ Axum
- sqlx + PostgreSQL（Dockerコンテナ）
- 単一ユーザー・APIキー簡易認証
- 既存: `backend/Dockerfile`（マルチステージビルド、8080番ポートでEXPOSE）
- 既存: `backend/docker-compose.yml`（db + app、両方ともホストにポート公開中）
- 既存: `backend/.env.example`（POSTGRES_USER/PASSWORD/DB, DATABASE_URL, INTERNAL_API_KEY）

### フロントエンド（[docs/frontend/tech-stack.md](../../frontend/tech-stack.md)）
- React 18.3+ / TypeScript 5.7+ / Vite 6
- TanStack Query 5 / React Router v7 / Tailwind CSS 4 + shadcn/ui
- パッケージマネージャー: yarn（`frontend/package.json`）
- 既存: Dockerfile・nginx.conf は未整備（新規作成が必要）
- 既存: `frontend/src/api/client.ts` の `BASE_URL` は `VITE_API_BASE_URL` 環境変数 or `http://localhost:8080/api/v1` のデフォルト値
- 既存: `frontend/vite.config.ts` にdevサーバーのプロキシ設定なし

## 関連実装ファイル

- `backend/Dockerfile`
- `backend/docker-compose.yml`
- `backend/.env.example`
- `frontend/src/api/client.ts`
- `frontend/vite.config.ts`
- `frontend/package.json`

## 既存設計文書

- `docs/design/backend-frontend-integration/`（ディレクトリのみ存在、中身は未作成）
- `docs/PRD-integration.md`（本要件のPRD）

## 注意事項

- `.github/workflows/backend-ci.yml` が現在の git status で削除(D)扱いだが、CI再構築は本要件のスコープ外（ユーザーヒアリングにて確認済み）
- selfhosted環境固有の設定（外部ファイルサーバーのバインドマウント、ドメイン/HTTPS終端）は本要件のスコープ外
- Vite devサーバーのdocker-compose内コンテナ化は不要。nginxによるビルド成果物配信のみで結合テスト・動作確認の目的を満たす
