# MediaVault-mcp

## 概要
MediaVaultのAIエージェント連携部分のPRD。AIエージェント（KnowledgeHub側常駐エージェント、Claude Codeなどの外部MCPクライアント）に対し、MediaVaultのデータ操作ツールを提供する薄いMCPアダプタ。
全体構想は[ルートPRD](../PRD.md)を参照。バックエンド側は[backend/PRD.md](../backend/PRD.md)、基本設計全体は[basic-design/00_overview.md](../basic-design/00_overview.md)を参照。

## 技術スタック
| 要素 | 技術 |
|------|------|
| サーバー実装 | MCPサーバー（言語・フレームワーク未確定） |
| データアクセス | `MediaVault-api` の `/api` のみ経由（DB直接アクセスなし） |
| デプロイ | Docker |

## 設計方針
- `MediaVault-mcp` は状態を持たず、すべてのデータ操作は `MediaVault-api` へのHTTP呼び出しに委譲する。
- ナレッジ（要約/wiki/embedding）の「生成方法」自体はmcpの責務ではない。呼び出し元エージェントがLLMで生成し、結果をmcpのツール経由で書き戻す。

## 提供ツール
| ツール | 対応API | 用途 |
|---|---|---|
| `search_items` | `GET /api/search` | 検索材料の収集 |
| `get_item` | `GET /api/items/{id}` | item詳細取得 |
| `get_item_text` | 抽出済み全文取得（`extract_text`ジョブの成果物） | wiki生成の材料取得 |
| `upsert_knowledge` | `POST /api/knowledge` | 要約/wiki/embedding結果の書き込み |
| `create_link` | `POST /api/items/{id}/links` | 外部リンク（`item_links`）の登録 |
| `enqueue_job` | `POST /api/jobs` | 大量処理・非同期処理をworkerへ委譲 |

## 利用経路

| 経路 | クライアント | 概要 |
|---|---|---|
| 内部経路 | KnowledgeHub常駐エージェント | 内部ネットワーク経由でmcpへ直接到達（外部リバースプロキシをバイパス） |
| 外部経路 | 外部MCPクライアント（Claude Code等） | 外部リバースプロキシ経由（`mcp.` FQDN）でmcpへ到達 |

典型的な利用フロー: エージェントが `search_items`/`get_item_text` で材料を収集 → 自身のLLMで要約/wiki/embeddingを生成 → `upsert_knowledge` で結果を書き戻す（大量データの処理は `enqueue_job` でworkerに委譲）。

## やらなくていいこと
- 要約/wiki/embeddingの生成ロジック自体をmcp内に実装すること（生成はエージェント側の責務）
- PostgreSQLへの直接アクセス（必ず `MediaVault-api` の `/api` を経由する）
- ユーザー管理・認証機能（単一ユーザー前提。外部経路のアクセス制御はインフラ設計側のリバースプロキシ/ネットワーク境界に委ねる）
