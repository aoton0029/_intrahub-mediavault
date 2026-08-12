# mediavault-mcp × intrahub-mastra 連携エンドポイント設計

**作成日**: 2026-08-11
**関連設計**: [architecture.md](architecture.md) / [mcp-tools.md](mcp-tools.md) / [interfaces.rs](interfaces.rs)
**関連PRD**: [MediaVault-mcp PRD](../PRD.md) §7.2・§8 / [intrahub-mastra PRD](../../../../../intrahub-mastra/docs/草案PRD.md)

> **注**: 本書は `intrahub-mastra`（Knowledge Vault生成Agent群）が MediaVault-mcp をMCPクライアントとして利用するために必要な、第2段階ツール `get_item_text` の詳細設計と、既存MVPツールの利用範囲を定義する。`mcp-tools.md` §「第2段階で追加するツール」を具体化するものであり、MVPツール（`search_library`, `get_item_context` 等）の仕様はそのまま流用する。

**【信頼性レベル凡例】**:
- 🔵 **青信号**: 要件定義書・設計文書・既存API仕様・PRDを参考にした確実な定義
- 🟡 **黄信号**: それらから妥当な推測による定義
- 🔴 **赤信号**: それらにない推測による定義

---

## 1. 前提: intrahub-mastraが必要とする操作 🔵

**信頼性**: 🔵 *intrahub-mastra PRD §9・§11・§13・§15 より*

`mediaResearchAgent`（Read Only）が `generateItemKnowledgeWorkflow` の中で呼び出すツールは以下の3つに限定される。

| ステップ | ツール | 用途 |
|---|---|---|
| 1 | `search_library` | トピック・作品名から対象Itemを解決する（US-01相当） |
| 2 | `get_item_context` | Item本体・関連作品・スタッフ・ファイル一覧などを取得する |
| 3 | `get_item_text` | ファイルの抽出済み全文を取得し、`ResearchResult` 生成の材料にする（**本書で新設**） |

書き込み系ツール（`create_item` / `update_consumption` / `organize_item` / `relate_items` / `add_access_link`）は `mediaResearchAgent` には一切公開しない。これはMediaVault-mcp側の権限制御ではなく、Mastra側でAgentに渡すツール集合を絞ることで実現する（intrahub-mastra PRD §8原則6・§15）。MediaVault-mcp側は将来的にトークンのスコープ分離（read-onlyトークン）を検討可能だが、MVPでは同一トークンで運用する。

## 2. get_item_text 🔵

**信頼性**: 🔵 *PRD §7.2 US-10・§8 `GET /api/v1/items/{id}/text` より*

### 目的

Itemに紐づくファイルの抽出済み全文を、ページまたはチャンク単位で取得する。MCP自身は要約・embeddingを生成しない（原則7）。

### 前提条件

- MediaVault-apiに `GET /api/v1/items/{item_id}/text` を新設する（本書のAPI要求）。
- 全文抽出処理（PDF/EPUB等からのテキスト抽出）は別途 `enqueue_job` の対象とし、本書のスコープ外とする。抽出未実行のファイルは `not_extracted` を返す。

### 入力スキーマ 🟡

**信頼性**: 🟡 *`get_item_context` のfile一覧構造・PRD US-10「ページまたはチャンク単位」から妥当推測*

```json
{
  "item_id": "string (required)",
  "file_id": "string (optional, 省略時は主ファイルを対象とする)",
  "chunk": {
    "index": "number (optional, 0起点)",
    "size": "number (optional, 既定4000文字)"
  }
}
```

- `file_id` を省略した場合、Itemに複数ファイルがあり主ファイルを一意に決められないときは `ambiguous` を返し、`file_id` の候補一覧を提示する。
- `chunk` を省略した場合は先頭チャンク（`index: 0`）を返す。

### 出力スキーマ 🟡

```json
{
  "outcome": "success",
  "item_id": "string",
  "file_id": "string",
  "extracted_at": "string (ISO8601) | null",
  "chunk": {
    "index": 0,
    "size": 4000,
    "total_chunks": 12,
    "text": "string"
  },
  "error": null
}
```

| `outcome` | 意味 |
|---|---|
| `success` | 指定チャンクを返した |
| `not_found` | `item_id` / `file_id` が存在しない |
| `not_extracted` | ファイルは存在するが全文抽出が未実行（`enqueue_job` での抽出依頼を促すメッセージを含む） |
| `ambiguous` | `file_id` が未指定でItemに複数ファイルがある |
| `error` | MediaVault-api接続エラー等 |

### 非機能要求 🔵

**信頼性**: 🔵 *PRD §11 より*

- 1回のレスポンスに全文を詰め込まない。`total_chunks` を返し、Agent側がループで取得する。
- `mediaResearchAgent` は取得したチャンクを `ResearchResult.sources[].fileId` / `chunks[].extractedAt` に記録し、出典追跡を維持する（intrahub-mastra PRD §13）。

### annotations 🔵

**信頼性**: 🔵 *mcp-tools.md 既存ツールの分類方式に準拠*

`get_item_text` は読み取り専用ツールであり、`readOnlyHint: true` を付与する。

## 3. MediaVault-apiへの要求（追記） 🔵

**信頼性**: 🔵 *PRD §8 の既存要求を本連携向けに具体化*

| API | 要求 |
|---|---|
| `GET /api/v1/items/{id}/text` | `file_id` 省略時の主ファイル解決、`chunk.index`/`chunk.size` によるオフセット取得、`extracted_at` の返却に対応する |
| 全文抽出ジョブ | `enqueue_job` の `job_type: "extract_text"` として、対象 `item_id` / `file_id` を受け取り、完了後に `get_item_text` が `not_extracted` を返さなくなることを保証する |

## 4. 認証・接続経路 🔵

**信頼性**: 🔵 *PRD §10・mcp-tools.md「認証」より*

- `intrahub-mastra` はStreamable HTTPで `POST /mcp` に接続し、`Authorization: Bearer {MCP_AUTH_TOKEN}` を使用する。既存のMVP認証方式をそのまま利用し、mastra専用の認証方式は設けない。
- `intrahub-mastra` はミニPC上の内部ネットワークから接続する常駐エージェントであるため、PRD §10の「内部経路」に該当する。リバースプロキシを経由しない接続を前提とする。
- `MCP_AUTH_TOKEN` は `intrahub-mastra` の環境変数として別途管理し、リポジトリに含めない。

## 5. 対象外（本書のスコープ外） 🔵

**信頼性**: 🔵 *intrahub-mastra PRD §5・MediaVault-mcp PRD §13 より*

- `enqueue_job` / `get_job` / `list_jobs` / `cancel_job` の詳細設計（PRD §7.2に別途記載、必要になった時点で設計する）
- Knowledge Vault側の `vault-mcp`（`search_notes` / `create_note` 等）の設計。これは `intrahub-mastra` 側のリポジトリ・別ドキュメントで扱う。
- MediaVault-mcpからのナレッジ取得・保存ツールの追加（両PRDの原則により提供しない）

## 関連文書

- [MediaVault-mcp PRD](../PRD.md)
- [mcp-tools.md](mcp-tools.md) — 既存MVPツール（`search_library` / `get_item_context` 等）の詳細仕様
- [intrahub-mastra PRD](../../../../../intrahub-mastra/docs/草案PRD.md) — 本連携を利用するAgent/Workflow設計
