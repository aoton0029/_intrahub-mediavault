# backend-frontend-integration 要件定義書

## 概要

バックエンド（Rust/Axum、port 8080）とフロントエンド（React/Vite）を1つのdocker-composeで結合し、結合テストやWebアプリとしての動作確認をローカル環境で容易に行えるようにする。ブラウザからのアクセス窓口はfrontendのnginxのみとし、`/api/` 宛のリクエストはnginxがbackendへリバースプロキシすることで、CORS設定不要・同一オリジン通信を実現する。

## 関連文書

- **ヒアリング記録**: [💬 interview-record.md](interview-record.md)
- **ユーザストーリー**: [📖 user-stories.md](user-stories.md)
- **受け入れ基準**: [✅ acceptance-criteria.md](acceptance-criteria.md)
- **コンテキストノート**: [📝 note.md](note.md)
- **PRD**: [docs/PRD-integration.md](../../PRD-integration.md)

## 機能要件（EARS記法）

**【信頼性レベル凡例】**:
- 🔵 **青信号**: PRD・設計文書・ユーザヒアリングを参考にした確実な要件
- 🟡 **黄信号**: PRD・設計文書・ユーザヒアリングから妥当な推測による要件
- 🔴 **赤信号**: PRD・設計文書・ユーザヒアリングにない推測による要件

### 通常要件

- REQ-001: システムはルート直下の統合用`docker-compose.yml`で `frontend` / `backend` / `db` の3コンテナを起動しなければならない 🔵 *PRD 提案アーキテクチャより*
- REQ-002: `backend`コンテナはシステムは既存の `backend/Dockerfile`（マルチステージビルド）をそのまま利用しなければならない 🔵 *PRD ディレクトリ・ファイル構成案より*
- REQ-003: `db`コンテナはシステムは `backend/docker-compose.yml` のPostgres定義（ヘルスチェック付き）を流用しなければならない 🔵 *PRD 提案アーキテクチャより*
- REQ-004: `frontend`コンテナはシステムはViteのビルド成果物をnginxで静的配信しなければならない 🔵 *PRD 提案アーキテクチャ・ヒアリングより（Vite devサーバーのコンテナ化は不要）*
- REQ-005: システムは新規に `frontend/Dockerfile` を作成しなければならない 🔵 *PRD ディレクトリ・ファイル構成案より*
- REQ-006: システムは新規に `frontend/nginx.conf`（または同等の配信設定）を作成しなければならない 🔵 *PRD ディレクトリ・ファイル構成案より*
- REQ-007: nginxはシステムは `/api/` 宛のリクエストを `http://backend:8080/api/` へリバースプロキシしなければならない 🔵 *PRD nginx設定例より*
- REQ-008: nginxはシステムは `/api/` 以外のリクエストに対し `try_files $uri /index.html` によるSPAフォールバックを行わなければならない 🔵 *PRD nginx設定例より*
- REQ-009: システムはフロントエンドのAPI呼び出し（`frontend/src/api/client.ts` の `apiClient`）のデフォルトURLを相対パス `/api/v1` に変更しなければならない 🔵 *ヒアリングにて含めることを確認済み*

### 条件付き要件

- REQ-101: `docker compose up` が実行された場合、システムは `db` のヘルスチェック成功後に `backend` を起動しなければならない 🔵 *既存 backend/docker-compose.yml の `depends_on: condition: service_healthy` より*
- REQ-102: ブラウザから統合環境のfrontendへアクセスした場合、システムは同一オリジン（frontendのnginx）経由でbackend APIと通信しなければならない 🔵 *PRD API疎通方式より*

### 状態要件

- REQ-201: `backend` と `db` が内部networkに所属する状態にある場合、システムはホストへポートを公開してはならない（`ports:` を設定しない） 🔵 *PRD ポート方針・ヒアリングより*
- REQ-202: `frontend` が起動状態にある場合、システムはホストにのみポート公開（例: `80:80`）しなければならない 🔵 *PRD ポート方針より*

### オプション要件

- REQ-301: システムは統合用compose定義を `docker-compose.yml` または `docker-compose.integration.yml` のいずれかのファイル名で配置してもよい 🟡 *PRD「もしくは」表記から妥当な推測*

### 制約要件

- REQ-401: システムは統合用docker-compose内で `backend`/`db` にホストポートを公開してはならない（内部networkのみで到達可能にする） 🔵 *ヒアリングにて非公開方針を確認済み*
- REQ-402: 既存の `backend/docker-compose.yml`（backend単体起動用）はシステムは変更しない 🔵 *ヒアリングにて統合用composeのみ非公開化する方針を確認済み*
- REQ-403: CI（GitHub Actions）の再構築・移行はシステムのスコープに含めない 🔵 *ヒアリングにて対象外と確認済み*
- REQ-404: selfhosted環境固有の設定（外部ファイルサーバーのバインドマウント、ドメイン/HTTPS終端）はシステムのスコープに含めない 🔵 *ヒアリングにて対象外と確認済み*
- REQ-405: Vite devサーバー自体のコンテナ化・docker-compose組み込みはシステムのスコープに含めない 🔵 *ヒアリングにて対象外と確認済み*

## 非機能要件

### パフォーマンス

- NFR-001: `docker compose up` によるフルスタック起動は、開発者のローカルマシンで数分以内に完了することが望ましい 🟡 *一般的な開発体験からの妥当な推測（PRD・設計文書に具体的数値なし）*

### セキュリティ

- NFR-101: `backend` と `db` はホストから直接アクセスできず、`frontend` 経由でのみAPIアクセス可能でなければならない 🔵 *PRD API疎通方式・セキュリティ意図より*
- NFR-102: 環境変数（`INTERNAL_API_KEY` 等の機密情報）は `.env` ファイルで管理し、リポジトリにコミットしてはならない 🔵 *backend/tech-stack.md セキュリティ節より*

### ユーザビリティ

- NFR-201: 開発者は `docker compose up` の単一コマンドでフロントエンド・バックエンド・DBの結合環境を起動できなければならない 🔵 *PRD 検証方法より*

## Edgeケース

### エラー処理

- EDGE-001: `db` のヘルスチェックが失敗し続ける場合、`backend` コンテナは起動を待機し続けなければならない（既存の `depends_on: condition: service_healthy` の挙動を踏襲） 🟡 *既存docker-compose設定から妥当な推測*
- EDGE-002: `backend` が起動前・起動失敗中にブラウザから `/api/` へアクセスした場合、nginxは502等のエラーをブラウザへ返す 🟡 *nginxリバースプロキシの一般的挙動からの妥当な推測*

### 境界値

- EDGE-101: `frontend` のホスト公開ポート（例: 80番）が既に使用中の場合、`docker compose up` は失敗する（compose標準挙動、本要件では特別なハンドリングを行わない） 🟡 *Docker Composeの標準挙動からの妥当な推測*
