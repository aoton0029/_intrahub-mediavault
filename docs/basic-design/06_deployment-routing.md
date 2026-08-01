# MediaVault 基本設計 — デプロイ経路とオリジン統合(CORS)

← [00_overview.md](00_overview.md) / [01_architecture.md](01_architecture.md)

MediaVaultは2つのデプロイモードで動く。ブラウザ視点でフロント(MediaVault-web)とAPI(MediaVault-api)を同一オリジンに統合する仕組みが、モードごとに異なる層で実現されている。この違いを理解せずに片方を「重複」と誤認して削除しないよう、ここに明記する。

## 2つのデプロイモード

| モード | 定義場所 | 公開方法 |
|---|---|---|
| 単体起動 | `MediaVault/docker-compose.yml`（Caddyなし） | `mediavault-web`のポートを直接公開して使う |
| 本番 | `IntraHub-Compose/docker-compose.mediavault.yml` + Caddy | `mediavault.{$BASE_DOMAIN}`（Caddy経由） |

## オリジン統合の仕組み

### 単体起動: nginxが `/api/` をプロキシする

`frontend/nginx.conf` は `mediavault-web` コンテナ内のnginxで、`/api/` 宛のリクエストを `backend`（`mediavault-api` のネットワークエイリアス、[docker-compose.yml:69-73](../../docker-compose.yml#L69-L73)）へリバースプロキシする([nginx.conf:16-20](../../frontend/nginx.conf#L16-L20))。

```
Browser → mediavault-web(nginx) ─┬─ /api/*  → mediavault-api:8080
                                  └─ それ以外 → 静的ファイル(SPA)
```

ブラウザからは常に `mediavault-web` の1オリジンしか見えないため、CORSは発生しない。**この経路は単体起動時にCORSを回避するための必須の仕組みであり、削除してはいけない。**

### 本番: Caddyがパスベースで振り分ける

`IntraHub-Compose/services/caddy/sites/mediavault.caddy` が、`mediavault.{$BASE_DOMAIN}` という単一オリジン配下で `/api/*` と それ以外をパスベースで振り分ける。

```
Browser → Caddy(mediavault.{$BASE_DOMAIN}) ─┬─ /api/*  → mediavault-api:8080
                                             └─ それ以外 → mediavault-web:80
```

Caddyが `/api/*` を先取りして直接 `mediavault-api` へ転送するため、リクエストは `mediavault-web` 内蔵nginxまで到達しない。したがって本番運用では nginx の `/api/` プロキシブロックは**実質使われない(が無害)**。これは重複バグではなく、単体起動モードのために必要な設定が本番では単に不要になっているだけである。

### まとめ

| モード | オリジン統合を行う層 | nginxの `/api/` プロキシ |
|---|---|---|
| 単体起動 | `mediavault-web` 内蔵nginx | 使われる(必須) |
| 本番(Caddy配下) | Caddy | 到達しない(無害・削除不要) |

いずれのモードでもブラウザは単一オリジンしか見ないため、通常運用でCORSは発生しない。

## `CORS_ALLOWED_ORIGIN` が実際に必要になるケース

`mediavault-api` の `CORS_ALLOWED_ORIGIN` 環境変数(バックエンド `main.rs` で使用)は、上記いずれのデプロイモードでも通常は参照されない。これが意味を持つのは、**フロントの開発サーバー(`yarn dev`、Vite等)を別ポートで起動し、ブラウザがそこから `mediavault-api` へ直接fetchする場合**のみである。この場合フロントとAPIのオリジンが異なるため、ブラウザがCORSチェックを行い、APIが許可オリジンを応答に含める必要がある。

`backend/.env.example` の既定値 `http://localhost:5173` はこの開発サーバー用のものであり、単体起動・本番いずれのcomposeでも実際にはこの値は使われない。
