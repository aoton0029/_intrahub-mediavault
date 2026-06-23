# TASK-0002 設定作業実行

## 作業概要

- **タスクID**: TASK-0002
- **作業内容**: docker-compose.yml作成（Postgresコンテナ＋アプリコンテナ）
- **実行日時**: 2026-06-23
- **実行者**: Claude Code（kairo-implement）

## 設計文書参照

- **参照文書**: docs/tasks/mediavault-backend/TASK-0002.md, docs/design/mediavault-backend/architecture.md, docs/backend/tech-stack.md
- **関連要件**: REQ-401, REQ-402, REQ-403, REQ-404

## 実行した作業

### 1. docker-compose.yml の作成

`backend/docker-compose.yml` を新規作成。

- `db` サービス: `postgres:16` イメージ、`.env` の `POSTGRES_USER`/`POSTGRES_PASSWORD`/`POSTGRES_DB` を注入、名前付きボリューム `mediavault_pgdata` でデータ永続化、`pg_isready` によるヘルスチェック付き。
- `app` サービス: `backend/Dockerfile` をビルド、`db` のヘルスチェック完了を待って起動（`depends_on: condition: service_healthy`）、`.env` を `env_file` で読み込み。

### 2. Dockerfile の作成

`backend/Dockerfile` を新規作成（マルチステージビルド）。

```dockerfile
FROM rust:1-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p mediavault-api

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libpq5 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/mediavault-api /app/mediavault-api
EXPOSE 8080
ENTRYPOINT ["/app/mediavault-api"]
```

builderステージでworkspace全体をコピーして `mediavault-api` をビルドし、runtimeステージにはバイナリのみ配置。

### 3. 環境変数の外部化

`backend/.env.example` を新規作成し、必要な環境変数のテンプレートを記載：

```
POSTGRES_USER=mediavault
POSTGRES_PASSWORD=changeme
POSTGRES_DB=mediavault
DATABASE_URL=postgresql://mediavault:changeme@db:5432/mediavault
INTERNAL_API_KEY=changeme
```

`backend/.gitignore` に `.env` を追加し、実際の `.env` ファイルがリポジトリにコミットされないようにした。

## 作業結果

- [x] `backend/docker-compose.yml` を作成した
- [x] `db` サービスがPostgreSQLイメージ＋名前付きボリュームで永続化される設定になっている
- [x] `.env` から環境変数を読み込む設定（`environment` + `${VAR}` / `env_file`）になっている
- [x] `docker compose config` でcompose定義の構文・変数展開が正しいことを確認済み
- [ ] `docker compose up -d db` の実機起動確認（**未実施**: 本環境にDocker Desktopのデーモンが起動していないため `docker compose up -d db` が `failed to connect to the docker API` で失敗。compose定義自体は `docker compose config` で検証済み）

## 遭遇した問題と解決方法

### 問題1: Docker daemonが起動していない

- **発生状況**: `docker compose up -d db` 実行時
- **エラーメッセージ**: `failed to connect to the docker API at npipe:////./pipe/dockerDesktopLinuxEngine: ... The system cannot find the file specified.`
- **解決方法**: 本作業環境ではDocker Desktopが起動していないため実機起動確認は未実施。`docker compose config` でcompose定義のYAML構文・変数展開・サービス定義が正しいことのみ確認した。Docker Desktop起動後にユーザー側で `docker compose up -d db` を実行し、`docker compose ps` でhealthy確認することを推奨。

## 次のステップ

- ユーザー環境でDocker Desktopを起動した上で `docker compose up -d db` を実行し、`docker compose ps` でhealthy状態を確認する
- `/tsumiki:direct-verify` を実行して設定を確認
