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

### 読み取り専用トークン（任意）

`MCP_READONLY_TOKEN` を設定すると、そのトークンで接続したクライアントには**読み取り専用ツールしか見えず、書き込みツールを呼び出せない**。読み取りしか必要としないクライアント（知識生成エージェント等）へはこちらを配る。

```bash
# .env
MCP_AUTH_TOKEN=<全権トークン>
MCP_READONLY_TOKEN=<読み取り専用トークン>
```

| トークン | `tools/list` に見えるツール |
|---|---|
| `MCP_AUTH_TOKEN` | 13個すべて |
| `MCP_READONLY_TOKEN` | 6個（`health` / `search_library` / `search_external_catalog` / `get_item_context` / `collection_overview` / `list_citations`） |

読み取り専用セッションが書き込みツールを直接呼んでも、「そのようなツールは存在しない」として拒否され MediaVault-api へは到達しない。両者に同じ値を設定した場合は**起動に失敗する**（読み取り専用のつもりで全権トークンを配る事故を防ぐため）。

`MCP_READONLY_TOKEN` を設定しない場合は従来どおり `MCP_AUTH_TOKEN` 単独運用となり、全ツールが公開される。

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

## 提供ツール

| ツール | 種別 | 用途 |
|---|---|---|
| `health` | 読み取り | api への到達性確認 |
| `search_library` | 読み取り | 所蔵確認 |
| `search_external_catalog` | 読み取り | 外部カタログ検索（所蔵確認ではない） |
| `get_item_context` | 読み取り | 作品の詳細・関連・シリーズ・引用件数をまとめて取得 |
| `collection_overview` | 読み取り | コレクション全体の統計 |
| `list_citations` | 読み取り | 記録済みの引用を一覧 |
| `import_external_item` | 書き込み | 外部検索結果を登録 |
| `create_item` | 書き込み | 手動登録 |
| `update_consumption` | 書き込み | 視聴・読了状況の記録 |
| `organize_item` | 書き込み | タグ・カテゴリ・マイリストの付与 |
| `relate_items` | 書き込み | 作品同士の関連付け |
| `add_access_link` | 書き込み | URL の追加 |
| `add_citation` | 書き込み | 引用の記録 |

削除系ツールは公開しない（REQ-141）。引用の更新・削除も提供しない（REQ-905）。エンドポイント単位の露出可否は [design/api-tool-mapping.md](design/api-tool-mapping.md) を参照。

## 補足

- ヘルスチェックは `/healthz` を使う。これは `mediavault-api` の状態に依存しないため、api 障害時に mcp コンテナが unhealthy 判定され再起動ループに陥ることを防ぐ（REQ-121, TASK-0007）。
- リバースプロキシ経由の外部公開は未確定のため、本構成では LAN 内公開（ポート `8081`）までを対象とする。
- `get_item_text` と jobs 系ツールは MediaVault-api 側が未実装のため提供していない（[design/api-tool-mapping.md](design/api-tool-mapping.md) §6）。
