# バックエンドとフロントエンドの結合

## 目的
バックエンドとフロントエンドの結合テストやWebアプリとしての動作確認を容易にするため、docker-composeで統合環境を構築する。
[バックエンド技術スタック](docs\backend\tech-stack.md)
[フロントエンド技術スタック](docs\frontend\tech-stack.md)

## 前提・スコープ
- 対象: `backend/`（Rust/Axum, port 8080）と `frontend/`（React/Vite）を1つのdocker-composeで結合し、ローカル環境・selfhosted環境の両方で起動確認できるようにする
- 非対象（今回は概要言及のみ、詳細は別途検討）: 本番デプロイの自動化、CI/CDの再構築、認証まわりの強化

## 提案アーキテクチャ
### 単体で起動する場合
- サービス構成（案）: `frontend` / `backend` / `db` の3コンテナ
  - `backend`: 既存の `backend/Dockerfile`（マルチステージビルド）をそのまま利用
  - `db`: `backend/docker-compose.yml` の Postgres定義（ヘルスチェック付き）を流用
  - `frontend`: 現状Dockerfile未整備のため新規作成が必要（Vite build成果物をnginx等で静的配信する案）
- ネットワーク: 単一の内部networkでfrontend/backend/dbを接続
- API疎通: frontend→backend間の通信方式
  - `backend` と `db` はホストにポート公開しない（`ports:` を設定せず、composeの内部networkのみで到達可能にする）。外部から直接APIやDBを叩けないようにする
  - `frontend` コンテナはnginxでビルド成果物を配信し、ホストにのみポート公開する（例: `80:80`）。ブラウザからのアクセス窓口はfrontendのみ
  - ブラウザ→backendの直接通信は行わない。frontend nginxが `/api/` 宛のリクエストを `http://backend:8080/api/` へリバースプロキシする
    ```nginx
    location /api/ {
        proxy_pass http://backend:8080/api/;
    }
    location / {
        try_files $uri /index.html;
    }
    ```
  - この構成により、ブラウザからは常にfrontendと同一オリジンでの通信となるため、CORS設定が不要になる
  - フロントエンドのAPI呼び出し（`frontend/src/api/client.ts` の `apiClient`）は環境ごとのURL出し分けをやめ、相対パス `/api/v1` を使用する（開発時はVite devサーバーのプロキシ設定で同様に `backend:8080` へ転送する）
- ポート（暫定案）: backend=8080（非公開、内部networkのみ）、db=5432（非公開、内部networkのみ）、frontend=80(nginx配信、ホスト公開)

## ディレクトリ・ファイル構成案
- ルート直下に統合用 `docker-compose.yml`（もしくは `docker-compose.integration.yml`）を新設
- `frontend/Dockerfile` を新規作成（現状バックエンドのみDockerfileが存在）
- 必要に応じて `frontend/nginx.conf` 等の配信設定を追加


## 検証方法（概要）
- `docker compose up` で全サービスが起動すること
- frontendからbackend APIへの疎通確認（ブラウザ操作 or curl）
- DBマイグレーションが正常に完了すること

