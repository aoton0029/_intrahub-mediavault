# MediaVault

## docker-compose ファイルの使い分け

| ファイル | 用途 | 起動コマンド |
|---|---|---|
| `docker-compose.test.yml` | ローカルでbackend・frontend・dbをまとめてbuildし結合確認するための統合環境（旧 `docker-compose.yml`） | `docker compose -f docker-compose.test.yml --env-file .env.test up -d --build` |
| `docker-compose.yml` | 本番（ミニPC）向け。自作サービスはソースからローカルbuild。バンドルOSS（Jellyfin/Calibre-Web）を含むフルスタック構成 | `docker compose -p mediavault --env-file .env.production up -d --build` |

イメージ指定はenvに置かない。自作サービス（`mediavault-api`/`mediavault-web`/`mediavault-worker`/`mediavault-mcp`）は`build:`でビルドし、バンドルOSSのタグは`docker-compose.yml`に直接固定する（`:latest`は使わない）。

## 環境変数ファイルの構成

env ファイルは「**それを読み込むツールごとに1つ**」に分ける。同じ値を複数のファイルに書かない。

| ファイル | 読み込む主体 | 責務 | サンプル |
|---|---|---|---|
| `.env.test` | `docker-compose.test.yml` の `${...}` 展開 | ローカル統合環境のPostgres/pgAdmin認証情報 | `.env.test.example` |
| `.env.production` | `docker-compose.yml` の `${...}` 展開 | 本番の環境依存値と秘密（ホストパス・TZ/PUID・ネットワーク名・DB認証情報・APIキー）。イメージ指定は含まない | `.env.production.example` |
| `backend/.env` | `cargo run` / `cargo test`（`main.rs` の `dotenvy`）と test compose の `env_file:` | backend単体をホスト上で動かすためのアプリ設定 | `backend/.env.example` |
| `frontend/.env` | `vite` 開発サーバー（`yarn dev` / `yarn test:e2e`） | 開発サーバーのAPIプロキシ先のみ。未作成でも既定値で動く | `frontend/.env.example` |

原則:

- **compose 経由の起動では compose の `environment:` が唯一の権威**。`backend/.env` の値でも、`environment:` に同名キーがあればそちらが優先される（`docker-compose.test.yml` は `DATABASE_URL` と `*_STORAGE_PATH` をこの方法で上書きしている）。
- **`env_file:` より `environment:` の明示列挙を優先する。** 何がコンテナに渡るか読めるし、不要な秘密が入り込まない。本番 `docker-compose.yml` は `env_file:` を使わない。
- **`frontend` には秘密を渡さない。** `VITE_` 付きの変数はビルド時にJSバンドルへ平文で焼き込まれ、ブラウザから誰でも読める。
- **実ファイル（`.env*`）はコミットしない**（`.gitignore` で `.env.*` を除外し `.env*.example` のみ追跡）。**サンプルは必ずコミットする。**
- 本番の `.env.production` はリポジトリ外（例: `/srv/mediavault/.env.production`、パーミッション600）に置き `--env-file` の絶対パスで指すのが望ましい。

## 統合環境（frontend + backend + db）の起動（テスト用）

```bash
# 1. compose用の .env.test を準備する（.env.test.example からコピー）
cp .env.test.example .env.test

# 2. backendコンテナへ渡すアプリ設定も準備する（test compose が env_file で読む）
cp backend/.env.example backend/.env

# 3. 統合環境を起動する
docker compose -f docker-compose.test.yml --env-file .env.test up -d --build

# 4. 起動状態を確認する（db・backend・frontend が Up なら正常）
docker compose -f docker-compose.test.yml ps
```

起動後、ブラウザで [http://localhost](http://localhost) にアクセスする。フロントエンドはnginx経由で配信され、`/api/`宛のリクエストはnginxが自動的にbackend（`http://backend:8080`）へリバースプロキシする。`backend`（8080番、ホストからは開発用に公開）・`db`（5432番）に加え、`frontend`（80番）にアクセスできる。

`backend/mediavault-api/migrations/` 配下のSQLは `backend` 自身が起動時に適用する（[main.rs](backend/mediavault-api/src/main.rs) の `db::run_migrations`）。別途 `sqlx-cli` や `psql` で手動適用する必要はなく、専用の `migrate` サービスも存在しない。

停止するには:
```bash
docker compose -f docker-compose.test.yml down
```

## 本番環境（docker-compose.yml）の起動

ミニPC実機を想定した本番用構成。`mediavault-api`/`mediavault-web`/`mediavault-worker`/`mediavault-mcp`は本リポジトリのソースからローカルビルドし、Jellyfin・Calibre-Webをバンドルする。個々のコンテナはホストへポートを公開せず、外部リバースプロキシ用の`proxy-net`（本グループ外で作成・注入される）経由でのみ公開される。詳細な構成意図は[インフラ設計側のMediaVault/README.md](../homelab/デバイス/ミニPC/サービス/MediaVault/README.md)を参照。

DBマイグレーションは`mediavault-api`が起動時に自ら適用する（[main.rs](backend/mediavault-api/src/main.rs)の`db::run_migrations`）。専用の`migrate`サービスは持たない。

```bash
# 1. 本番用 .env を準備する（.env.production.example からコピーし値を書き換える）
cp .env.production.example .env.production

# 2. 起動する（proxy-net は事前に外部で作成されている前提）
#    worker/mcp は未実装（Dockerfile未作成）のため、ビルド対象から除外して起動する
docker compose -p mediavault --env-file .env.production up -d --build \
  postgres mediavault-api mediavault-web jellyfin calibre-web

# 3. 起動状態を確認する
docker compose -p mediavault --env-file .env.production ps
```

`mediavault-worker`/`mediavault-mcp`は`build:`（`backend/Dockerfile.worker`・`backend/Dockerfile.mcp`）を指定してあるが、**Dockerfileとソースがまだ無いためビルドできない**。実装が入るまではサービス名を明示して上記のように除外する。設計は[docs/backend/mediavault-worker/PRD.md](docs/backend/mediavault-worker/PRD.md)・[docs/backend/mediavault-mcp/PRD.md](docs/backend/mediavault-mcp/PRD.md)を参照。

環境依存値（`DATA_ROOT`/`ANIME_ROOT`/`LIVE_ACTION_ROOT`/`MANGA_ROOT`/`MEDIAVAULT_ROOT`/`TZ`/`PUID`/`PGID`/Postgres認証情報/`INTERNAL_API_KEY`/`CORS_ALLOWED_ORIGIN`/外部メタデータAPIキー/検索バックエンド/LLMエンドポイント等）はすべて`.env.production`に外だししてある。`.env.production`はコミットしない（`.gitignore`で除外済み）。

`INTERNAL_API_KEY` と各GHCRイメージタグ・Postgres認証情報は `${VAR:?}` で必須指定してあるため、未設定なら起動前にエラーになる。起動前に次のコマンドで全変数が解決することを確認できる。

```bash
docker compose -p mediavault --env-file .env.production config >/dev/null
```

## backend
```
cd backend
cargo init --name backend
cargo add tokio --features full
cargo add axum
cargo add serde --features derive
cargo add serde_json
cargo add tower
cargo add tower-http --features cors
```

### backend単体の起動（ホスト上で cargo run）

`backend/.env.example` を `backend/.env` にコピーし、必要に応じて値を変更する（`DATABASE_URL` は `localhost` のPostgresを指している）。

```bash
# Postgresだけコンテナで用意する
docker compose -f docker-compose.test.yml --env-file .env.test up -d db

cd backend
cp .env.example .env
cargo run -p mediavault-api      # main.rs が dotenvy で backend/.env を読む
```

`.env` はリポジトリにコミットしない（`.gitignore` で除外済み）。backend単体用の compose ファイルは存在しない（統合環境は `docker-compose.test.yml`）。

## frontend
```
cd frontend
yarn create vite . --template react-ts
yarn install
```

採用パッケージ: react-router-dom v7, @tanstack/react-query v5, react-hook-form, zod, sonner, Tailwind CSS v4 + shadcn/ui（`components.json`、`@/`エイリアス）, Vitest + Testing Library, Playwright。

### 開発コマンド
```bash
cd frontend
yarn dev        # 開発サーバー起動
yarn build      # 型チェック+ビルド
yarn lint       # ESLint
yarn test       # Vitestユニットテスト
yarn test:watch # Vitestウォッチモード
yarn test:e2e   # Playwright E2Eテスト（初回は npx playwright install が必要）
```

### shadcn/uiコンポーネント追加
```bash
cd frontend
npx shadcn@latest add <component-name>
```

### トラブルシューティング: shadcn CLIの「Could not load the workspace config」エラー

`package.json`が`"type": "module"`のESM環境のため、`vite.config.ts`・`vitest.config.ts`で`__dirname`を直接使うとshadcn CLIのワークスペース設定読み込みに失敗する。`path.dirname(fileURLToPath(import.meta.url))`で算出すること。また`tsconfig.json`（ルート）にも`compilerOptions.paths`（`@/*` → `./src/*`）を設定する必要がある（`tsconfig.app.json`のみでは`tsconfig-paths`が解決できない）。

## 詳細
- バックエンドの詳細草案は[docs/backend/PRD.md](docs/backend/PRD.md)を参照。
- フロントエンドの詳細草案は[docs/frontend/PRD.md](docs/frontend/PRD.md)を参照。

- 映画・アニメ・漫画・小説・ドラマ・ゲーム・論文/文献・書籍等のメタデータを一元管理するセルフホスト型アプリケーション。
- API/Webは`proxy-net`、API/PostgreSQLは`db-net`で接続し、ホストへ直接ポートを公開しない。
- アップロードファイルはMediaVault専用領域へ保存する。
  - 保存先ルートは`STORAGE_ROOT`（ホスト側`MEDIAVAULT_ROOT`と同じ`/srv/mediavault`）で、その配下に種別サブディレクトリ`video/` `image/` `audio/` `pdf/` `archive/` `other/`が作られる。サブディレクトリ名は`STORAGE_SUBDIR_*`で上書きできる。
  - `/srv/anime`・`/srv/live-action`・`/srv/manga`は読み取り専用で参照する。既存ファイルは`POST /items/:id/files`で絶対パスを登録してリンクし、コピー・移動しない。
  - 詳細は [docs/backend/mediavault-api/item-files.md](./docs/backend/mediavault-api/item-files.md) を参照。
