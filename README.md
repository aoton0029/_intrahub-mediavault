# MediaVault

## 統合環境（frontend + backend + db）の起動

ローカルでbackend・frontend・dbをまとめて起動し結合確認するための、リポジトリルート専用の統合`docker-compose.yml`。`backend/docker-compose.yml`（backend単体起動用）とは別ファイルであり、既存のbackend単体起動手順に影響しない。

```bash
# 1. ルートの .env を準備する（.env.example からコピー）
cp .env.example .env

# 2. backend単体起動用の .env も必要（backend/docker-compose.yml がbuild時に参照）
cp backend/.env.example backend/.env

# 3. 統合環境を起動する
docker compose up -d --build

# 4. 起動状態を確認する（frontend/backend/dbの3サービスが Up になっていること。dbは healthy）
docker compose ps
```

起動後、ブラウザで [http://localhost](http://localhost) にアクセスする。フロントエンドはnginx経由で配信され、`/api/`宛のリクエストはnginxが自動的にbackend（`http://backend:8080`）へリバースプロキシする。`backend`（8080番）・`db`（5432番）はホストに公開されず、`frontend`（80番）のみアクセス可能。

初回起動時にDBスキーマが空の場合は、`backend/mediavault-api/migrations/`配下のSQLを適用する必要がある（`sqlx-cli`または`psql`を利用）。

停止するには:
```bash
docker compose down
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