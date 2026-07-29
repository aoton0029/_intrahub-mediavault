# MediaVault

## docker-compose ファイルの使い分け

| ファイル | 用途 | 起動コマンド |
|---|---|---|
| `docker-compose.test.yml` | ローカルでbackend・frontend・dbをまとめてbuildし結合確認するための統合環境（旧 `docker-compose.yml`） | `docker compose -f docker-compose.test.yml --env-file .env up -d --build` |
| `docker-compose.yml` | 本番（ミニPC）向け。自作サービスはGHCRのイメージを参照し、ローカルbuildは行わない。バンドルOSS（Jellyfin/Calibre-Web）を含むフルスタック構成 | `docker compose -p mediavault --env-file .env.production up -d` |

`backend/docker-compose.yml`（backend単体起動用）は上記いずれとも別ファイルであり、既存のbackend単体起動手順に影響しない。

## 統合環境（frontend + backend + db）の起動（テスト用）

```bash
# 1. ルートの .env を準備する（.env.example からコピー）
cp .env.example .env

# 2. backend単体起動用の .env も必要（backend/docker-compose.yml がbuild時に参照）
cp backend/.env.example backend/.env

# 3. 統合環境を起動する
docker compose -f docker-compose.test.yml --env-file .env up -d --build

# 4. 起動状態を確認する（db・backend・frontend が Up、migrate は Exited (0) なら正常）
docker compose -f docker-compose.test.yml ps
```

起動後、ブラウザで [http://localhost](http://localhost) にアクセスする。フロントエンドはnginx経由で配信され、`/api/`宛のリクエストはnginxが自動的にbackend（`http://backend:8080`）へリバースプロキシする。`backend`（8080番）・`db`（5432番）はホストに公開されず、`frontend`（80番）のみアクセス可能。

統合環境では `db` の起動後に `migrate` ワンショットサービスが `backend/mediavault-api/migrations/` 配下のSQLを `sqlx migrate run` で適用してから backend を起動する。初回起動時に別途 `sqlx-cli` や `psql` で手動適用する必要はない。`migrate` はワンショットのため、`docker compose ps` 上で `Exited (0)` になるのが正常。

停止するには:
```bash
docker compose -f docker-compose.test.yml down
```

## 本番環境（docker-compose.yml）の起動

ミニPC実機を想定した本番用構成。`mediavault-web`/`mediavault-api`/`mediavault-worker`/`mediavault-mcp`はGHCRへpush済みのイメージ（コミットSHAタグまたはSemVerタグ固定）を参照し、Jellyfin・Calibre-Webをバンドルする。個々のコンテナはホストへポートを公開せず、外部リバースプロキシ用の`proxy-net`（本グループ外で作成・注入される）経由でのみ公開される。詳細な構成意図は[インフラ設計側のMediaVault/README.md](../インフラ設計/デバイス/ミニPC/サービス/MediaVault/README.md)を参照。

```bash
# 1. 本番用 .env を準備する（.env.production.example からコピーし値を書き換える）
cp .env.production.example .env.production

# 2. 起動する（proxy-net は事前に外部で作成されている前提）
docker compose -p mediavault --env-file .env.production up -d

# 3. 起動状態を確認する（migrate は Exited (0) なら正常）
docker compose -p mediavault --env-file .env.production ps
```

環境依存値（`DATA_ROOT`/`MEDIA_ROOT`/`DOCUMENTS_ROOT`/`TZ`/`PUID`/`PGID`/Postgres認証情報/検索バックエンド/LLMエンドポイント/GHCRイメージ参照等）はすべて`.env.production`に外だししてある。`.env.production`はコミットしない（`.gitignore`で除外済み）。

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

### Docker Compose（Postgres + アプリ）
`backend/.env.example` を `backend/.env` にコピーし、必要に応じて値を変更する。

```
cd backend
cp .env.example .env
docker compose up -d db          # Postgresコンテナのみ起動（開発時）
docker compose ps                # healthyになっていることを確認
docker compose up -d             # アプリコンテナも含めて起動
```

`.env` はリポジトリにコミットしない（`.gitignore` で除外済み）。

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
- バックエンドの詳細草案（機能要件・内部REST API・データベース・api-client-lib）は[backend/docs/PRD.md](../backend/docs/PRD.md)を参照。
- フロントエンドの詳細草案（UI機能要件・画面構成）は[frontend/docs/PRD.md](../frontend/docs/PRD.md)を参照。

- 映画・アニメ・漫画・小説・ドラマ・ゲーム・論文/文献・書籍等のメタデータを一元管理するセルフホスト型アプリケーション。
- コンテナは`selfhosted-net`・`db-net`([PostgreSQL](../PostgreSQL/README.md)利用)・`ai-net`([RAG-Service](../RAG-Service/README.md)の`POST /ingest`呼び出し用)に参加する。Caddy経由(`app.home.lan`)で公開するため`proxy-net`にも参加し、ホスト直接ポート公開は行わない。
- アップロードファイルはコンテナ内ではなくファイルサーバー用HDDへ直接保存する（バインドマウント）。
  - PDF: `/srv/files/pdf`（[Calibre-Web](../Calibre-Web/README.md)のライブラリパスと共用）。アップロードされたPDFはCalibre-Webからも自動でライブラリ認識され、MediaVault側の作品詳細にCalibre-Webの閲覧URL(`calibre.home.lan`)へのリンクを保持して直接遷移できるようにする。
  - 画像: `/srv/media/photos`（Jellyfin/Sambaの`photos`共有と同一パスを共用）。
  - OCRテキスト: `/srv/files/ocr`（[RAG-Service](../RAG-Service/README.md)が`/ingest`処理時に書き込み、レスポンスで受け取った`ocr_text_path`をPostgreSQLに保存し、作品詳細から参照する）。

```yaml
# selfhosted/docker-compose.yml (例)
services:
  mediavault:
    image: <未定>
    networks:
      - selfhosted-net
      - db-net      # PostgreSQL利用
      - ai-net      # RAG-Service呼び出し(PDF全文ベクトル化トリガー)
      - proxy-net   # Caddy経由で公開
    volumes:
      - /srv/files/pdf:/data/pdf      # PDFアップロード保存先（Calibre-Webと共用）
      - /srv/media/photos:/data/photos  # 画像アップロード保存先（Samba/Jellyfin photosと共用）
      - /srv/files/ocr:/data/ocr:ro     # OCRテキスト参照用（RAG-Serviceが書き込み、本サービスは読み取り専用）
    # ports: ホスト直接公開はしない方針。Caddy経由(app.home.lan)とする
```
