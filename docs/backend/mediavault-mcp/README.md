# mediavault-mcp

MediaVault のメタデータ検索・登録・整理を MCP (Model Context Protocol) 経由で操作するためのサーバー。`mediavault-api` とは別コンテナで動作し、`mediavault-api` が停止していても独立して動き続ける（REQ-121）。

## 起動

リポジトリルートで:

```bash
cp .env.example .env
# MCP_AUTH_TOKEN を openssl rand -base64 48 で生成した値に設定する
docker compose up -d --build mediavault-mcp
```

ヘルスチェック:

```bash
curl http://localhost:8081/healthz
```

## MCPクライアントからの接続設定

エンドポイントは `http://<ホスト>:8081/mcp`（Streamable HTTP）。`Authorization: Bearer <MCP_AUTH_TOKEN>` ヘッダーが必須（`/healthz` を除く）。

### Claude Code

```bash
claude mcp add --transport http mediavault http://<ホスト>:8081/mcp \
  --header "Authorization: Bearer <MCP_AUTH_TOKEN の値>"
```

`<ホスト>` は同一マシンなら `localhost`、LAN内の別端末からは mediavault-mcp を動かしているホストの IP アドレスに置き換える。`<MCP_AUTH_TOKEN の値>` は `.env` に設定した実際のトークンに置き換える（このファイルには書かない）。

### 設定ファイルで指定する場合（例: `mcp.json` 相当）

```json
{
  "mcpServers": {
    "mediavault": {
      "type": "http",
      "url": "http://<ホスト>:8081/mcp",
      "headers": {
        "Authorization": "Bearer <MCP_AUTH_TOKEN の値>"
      }
    }
  }
}
```

## 動作確認

```bash
# ビルドと起動
docker compose build mediavault-mcp
docker compose up -d mediavault-mcp
docker compose ps                 # healthy になること

# コンテナ間通信
docker compose exec mediavault-mcp curl -f http://mediavault-api:8080/api/v1/health

# api を止めても mcp は生き続ける（REQ-121）
docker compose stop mediavault-api
curl http://localhost:8081/healthz              # 200 のまま
docker compose ps mediavault-mcp                # healthy のまま
docker compose start mediavault-api
```

`MCP_AUTH_TOKEN` を空にして `docker compose up mediavault-mcp` すると、コンテナは起動に失敗する（トークン値はログに出力されない）。

## 補足

- ヘルスチェックは `/healthz` を使う。これは `mediavault-api` の状態に依存しないため、api 障害時に mcp コンテナが unhealthy 判定され再起動ループに陥ることを防ぐ（REQ-121, TASK-0007）。
- リバースプロキシ経由の外部公開は未確定のため、本構成では LAN 内公開（ポート `8081`）までを対象とする。
