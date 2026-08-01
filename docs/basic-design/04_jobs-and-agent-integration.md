# MediaVault 基本設計 — ジョブ/エージェント連携

← [00_overview.md](00_overview.md)

本ページは `MediaVault-worker` のジョブモデルと `MediaVault-mcp` のツール境界、KnowledgeHubとの責務分界を整理する。各コンポーネントの詳細は [../backend/mediavault-worker/PRD.md](../backend/mediavault-worker/PRD.md)・[../backend/mediavault-mcp/PRD.md](../backend/mediavault-mcp/PRD.md) を参照。

## ジョブモデル

ジョブは `jobs` テーブルで管理され、`MediaVault-worker` がポーリングして実行する。2種類に大別される。

テーブル定義・enqueue の契約・状態遷移・リトライ方針・結果のフロントエンドへの反映方法は [05_job-queue.md](05_job-queue.md) を参照。

### パイプラインジョブ（自動・エージェント非関与）

`MediaVault-api` がファイル登録時に自動でenqueueする。

| ジョブ種別 | 内容 |
|---|---|
| `extract_text` | PDF等からのテキスト抽出（全文検索インデックス用） |
| `index` | 検索インデックスの更新 |
| `resolve_links` | Jellyfin/Calibre-Web APIを呼び出し `item_links` へ登録 |

### エージェント駆動ジョブ（判断はエージェント、実行のみworker）

`MediaVault-mcp` 経由でKnowledgeHub側エージェントがenqueueする。

| ジョブ種別 | 内容 |
|---|---|
| `wiki` | 要約/wikiページ生成 → `knowledge` へ格納 |
| `embed` | embedding生成 → `knowledge` へ格納 |

`MediaVault-worker` はデフォルトではこれら生成ロジックの「どう作るか」を自前実装しない。自前実行する場合のみ `LLM_BASE_URL`/`LLM_API_KEY` に依存する（インフラ設計側 `設計.md` 参照）。

## MediaVault-mcp のツール境界

`MediaVault-mcp` はAIエージェント向けの薄いアダプタであり、`MediaVault-api` の `/api` のみを呼び出す（DB直接アクセスはしない）。提供ツール（案）:

| ツール | 対応API | 用途 |
|---|---|---|
| `search_items` | `GET /api/search` | 検索材料の収集 |
| `get_item` | `GET /api/items/{id}` | item詳細取得 |
| `get_item_text` | 抽出済み全文取得 | wiki生成の材料 |
| `upsert_knowledge` | `POST /api/knowledge` | 生成結果の書き込み |
| `create_link` | `POST /api/items/{id}/links` | 外部リンク登録 |
| `enqueue_job` | `POST /internal/jobs` | 大量処理をworkerへ委譲（内部APIキー認証。[05_job-queue.md](05_job-queue.md)） |

典型フロー: エージェントが `search_items`/`get_item_text` で材料収集 → 自身のLLMで生成 → `upsert_knowledge` で書き戻し（大量処理は `enqueue_job` でworkerに委譲）。

## 利用経路

| 経路 | クライアント | 備考 |
|---|---|---|
| 内部経路 | KnowledgeHub常駐エージェント | 内部ネットワーク経由でmcpに到達（リバースプロキシをバイパス） |
| 外部経路 | 外部MCPクライアント（Claude Code等） | リバースプロキシ経由（`mcp.` FQDN）でmcpに到達 |

ネットワーク上の到達経路の詳細（`proxy-net`分離等）はインフラ設計側 `設計.md` を参照。

## KnowledgeHubとの責務分界

- ナレッジ「生成」（要約/wiki/embeddingを実際にどう作るか）はMediaVault内に実装しない。KnowledgeHub側エージェントの責務とし、MediaVaultはmcp経由のツール（材料提供・結果格納・ジョブenqueue）のみを提供する（[00_overview.md](00_overview.md) 設計原則6）。
- 横断的な全文検索（`items` + `knowledge` をまたぐ検索）はKnowledgeHub側の責務。MediaVaultの `/api/search` は自身の `items` のみを対象とする（[03_api-design.md](03_api-design.md)）。

## やらなくていいこと

- MediaVault-worker/mcp が生成ロジック（要約/wiki/embeddingの中身）を自前実装すること（既定ではKnowledgeHub側エージェントに委譲）
- MediaVault-mcp がDBに直接アクセスすること（必ず `/api` 経由）
- KnowledgeHubの生成物は`vault-mcp`経由で`/srv/knowledge/vault/10_Knowledge`へ保存し、MediaVaultのアイテムIDを出典として記録する。

## 関連ドキュメント

- [05_job-queue.md](05_job-queue.md) — `jobs` テーブル/enqueue契約/結果反映の詳細設計
- [../backend/mediavault-worker/PRD.md](../backend/mediavault-worker/PRD.md)
- [../backend/mediavault-mcp/PRD.md](../backend/mediavault-mcp/PRD.md)
- [03_api-design.md](03_api-design.md)
