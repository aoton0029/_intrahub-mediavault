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

現時点で該当するジョブ種別はない。ナレッジの生成も格納もKnowledgeHub側で完結するため（後述「KnowledgeHubとの責務分界」）、`MediaVault-worker` が実行するのはパイプラインジョブのみである。

KnowledgeHub側エージェントは `MediaVault-mcp` の `enqueue_job` を通じて、上記パイプラインジョブ（`extract_text` / `index` / `resolve_links`）の再実行を依頼できる。`MediaVault-worker` は要約/wiki/embeddingの生成ロジックを持たず、LLMエンドポイントにも依存しない。

## MediaVault-mcp のツール境界

`MediaVault-mcp` はAIエージェント向けの薄いアダプタであり、`MediaVault-api` の `/api` のみを呼び出す（DB直接アクセスはしない）。ツールの全体像・MVP範囲は [../backend/mediavault-mcp/PRD.md](../backend/mediavault-mcp/PRD.md) §7 を正とする。ナレッジ生成に関わる部分だけを抜き出すと次のとおり:

| ツール | 対応API | 用途 |
|---|---|---|
| `search_library` | `GET /api/v1/items` | 検索材料の収集 |
| `get_item_context` | `GET /api/v1/items/{id}` ほか | item詳細と関連情報の取得 |
| `get_item_text` | 抽出済み全文取得（新設が必要） | wiki生成の材料 |
| `add_access_link` | `POST /api/v1/items/{id}/links` | 外部リンク登録 |
| `enqueue_job` | `POST /internal/jobs` | 大量処理をworkerへ委譲（内部APIキー認証。[05_job-queue.md](05_job-queue.md)） |

典型フロー: エージェントが `search_library`/`get_item_context`/`get_item_text` で材料収集 → 自身のLLMで生成 → KnowledgeHub側の `vault-mcp` でVaultへ書き込み（大量の抽出・索引処理は `enqueue_job` でworkerに委譲）。生成結果を `MediaVault-api` へ書き戻すツールは提供しない。

## 利用経路

| 経路 | クライアント | 備考 |
|---|---|---|
| 内部経路 | KnowledgeHub常駐エージェント | 内部ネットワーク経由でmcpに到達（リバースプロキシをバイパス） |
| 外部経路 | 外部MCPクライアント（Claude Code等） | リバースプロキシ経由（`mcp.` FQDN）でmcpに到達 |

ネットワーク上の到達経路の詳細（`proxy-net`分離等）はインフラ設計側 `設計.md` を参照。

## KnowledgeHubとの責務分界

- **ナレッジの正本はKnowledgeHub Vault**（`/srv/knowledge/vault/10_Knowledge`）。MediaVaultはナレッジ本文（要約/wiki/embedding）を所有せず、材料（itemメタデータ・抽出済み全文）の提供に限定する。
- したがってナレッジは「生成」（どう作るか）だけでなく「格納」もKnowledgeHub側の責務である。MediaVaultはmcp経由で材料提供とジョブenqueueのみを提供する（[00_overview.md](00_overview.md) 設計原則6）。
- KnowledgeHubの生成物は `vault-mcp` 経由で `/srv/knowledge/vault/10_Knowledge` へ保存し、MediaVaultのアイテムIDを出典として記録する。MediaVault側からVaultノートへの逆引き参照はMVPでは持たない。
- 横断的な全文検索（MediaVaultの `items` とVaultのナレッジをまたぐ検索）はKnowledgeHub側の責務。MediaVaultの検索は自身の `items` のみを対象とする（[03_api-design.md](03_api-design.md)）。

## やらなくていいこと

- MediaVault-worker/mcp が生成ロジック（要約/wiki/embeddingの中身）を自前実装すること
- MediaVault が生成結果を格納するテーブル・APIを持つこと（正本はKnowledgeHub Vault）
- MediaVault-mcp がDBに直接アクセスすること（必ず `/api` 経由）

## 関連ドキュメント

- [05_job-queue.md](05_job-queue.md) — `jobs` テーブル/enqueue契約/結果反映の詳細設計
- [../backend/mediavault-worker/PRD.md](../backend/mediavault-worker/PRD.md)
- [../backend/mediavault-mcp/PRD.md](../backend/mediavault-mcp/PRD.md)
- [03_api-design.md](03_api-design.md)
