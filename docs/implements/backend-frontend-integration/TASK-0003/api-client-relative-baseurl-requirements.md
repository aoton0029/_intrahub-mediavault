# TASK-0003 要件定義: apiClient のデフォルトBASE_URL相対パス化

## 1. 機能の概要

- 🔵 `frontend/src/api/client.ts` の `BASE_URL` デフォルト値を絶対URL `http://localhost:8080/api/v1` から相対パス `/api/v1` に変更する。
- 🔵 統合環境（nginxリバースプロキシ経由）でフロントエンドとバックエンドを同一オリジンで疎通させるための変更。
- 🔵 `VITE_API_BASE_URL` 環境変数が設定されている場合はそちらを優先する既存の挙動は維持する。
- **参照したEARS要件**: REQ-009
- **参照した設計文書**: docs/design/backend-frontend-integration/architecture.md

## 2. 入力・出力の仕様

- 🔵 入力: `import.meta.env.VITE_API_BASE_URL`（未設定の場合は `undefined`）
- 🔵 出力: `BASE_URL` 定数。環境変数未設定時は `/api/v1`、設定時はその値。
- 🔵 `apiClient(path)` はこの `BASE_URL` と `path` を連結してfetchする（`frontend/src/api/client.ts:25`）。
- **参照したEARS要件**: REQ-009
- **参照した設計文書**: frontend/src/api/client.ts（既存実装）

## 3. 制約条件

- 🔵 既存の `frontend/src/api/*.test.ts`（groups.test.ts, relations.test.ts, search.test.ts等）が全てパスすること
- 🟡 テスト内でリクエストURLをアサーションしている箇所は相対パス基準に修正が必要な場合がある
- 🔵 `??` 演算子によるフォールバック挙動は変更しない
- **参照したEARS要件**: NFR-102（機密情報管理は本タスク対象外）, REQ-009
- **参照した設計文書**: docs/design/backend-frontend-integration/architecture.md

## 4. 想定される使用例

- 🔵 通常使用: 統合環境でnginx経由リクエストする際、`/api/v1/items` のような相対パスでリクエストされ、同一オリジンとして扱われる
- 🟡 開発者が `yarn dev` 単体起動時（nginxを経由しない場合）、`VITE_API_BASE_URL` 未設定だと `http://localhost:5173/api/v1/...` にリクエストが飛び、疎通しない可能性がある（Vite devサーバープロキシはスコープ外）
- **参照したEARS要件**: REQ-009, 受け入れ基準 TC-003-01
- **参照した設計文書**: docs/design/backend-frontend-integration/dataflow.md

## 5. EARS要件・設計文書との対応関係

- **参照した機能要件**: REQ-009
- **参照した受け入れ基準**: TC-003-01
- **参照した設計文書**: architecture.md, dataflow.md

## 品質判定

✅ 高品質: 要件の曖昧さなし、入出力定義完全、制約条件明確、実装可能性確実。信号は🔵が中心で🟡は既存テスト影響範囲の推測部分のみ。
