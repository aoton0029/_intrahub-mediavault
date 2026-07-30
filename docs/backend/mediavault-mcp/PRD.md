# MediaVault-mcp

## 概要
MediaVaultのAIエージェント連携部分のPRD。AIエージェント（KnowledgeHub側常駐エージェント、Claude Codeなどの外部MCPクライアント）に対し、MediaVaultのデータ操作ツールを提供する薄いMCPアダプタ。
全体構想は[ルートPRD](../PRD.md)を参照。バックエンド側は[backend/PRD.md](../backend/PRD.md)、基本設計全体は[basic-design/00_overview.md](../basic-design/00_overview.md)を参照。

## 技術スタック
| 要素 | 技術 |
|------|------|
| サーバー実装 | Rust + [`rmcp`](https://github.com/modelcontextprotocol/rust-sdk)（MCP公式Rust SDK） |
| 非同期ランタイム | tokio |
| トランスポート | Streamable HTTP（常駐サーバー用）／ stdio（Claude Code等のローカル起動用） |
| データアクセス | `api-client-lib` 経由で `MediaVault-api` の `/api` を呼ぶ（DB直接アクセスなし） |
| デプロイ | Docker |

バックエンド（Axum）と同じRustで実装し、Cargoワークスペースの1クレートとして `MediaVault-api` / `api-client-lib` と同一リポジトリで管理する。

### rmcp の使い方（前提）
- 依存: `cargo add rmcp --features server`（HTTP常駐時は該当トランスポートのfeatureも有効化）
- ツール定義は proc macro で行う。
  - `#[tool_router]` … ツールを持つimplブロックに付与し、ディスパッチを生成
  - `#[tool(description = "...")]` … 個々のツールハンドラ関数に付与
  - `#[tool_handler]` … `ServerHandler` 実装を生成
- 引数は `Parameters<T>` で受け取り、`T` に `serde::Deserialize` + `schemars::JsonSchema` を derive すると `inputSchema` / `outputSchema` が自動生成される。docコメントがツール説明としてスキーマに載る。
- 起動は `service.serve(transport).await?` → `server.waiting().await?`。

```rust
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SearchItemsParams {
    /// 検索クエリ
    query: String,
    /// media_type での絞り込み（任意）
    media_type: Option<String>,
}

#[tool_router(server_handler)]
impl MediaVaultMcp {
    #[tool(description = "MediaVaultのアイテムを検索する")]
    async fn search_items(&self, Parameters(p): Parameters<SearchItemsParams>) -> ... { }
}
```

## 設計方針
- `MediaVault-mcp` は状態を持たず、すべてのデータ操作は `MediaVault-api` へのHTTP呼び出しに委譲する。
- HTTPクライアントは自前実装せず `api-client-lib` の型・クライアントを再利用し、リクエスト/レスポンスの型定義をAPI側と一元化する。
- ナレッジ（要約/wiki/embedding）の「生成方法」自体はmcpの責務ではない。呼び出し元エージェントがLLMで生成し、結果をmcpのツール経由で書き戻す。
- ツールの入出力スキーマは Rust 構造体を単一の情報源とし、手書きのJSON Schemaは持たない。
- `MediaVault-api` のエラーは MCP のツールエラーへ変換して返す（ステータスコード・エラーメッセージを失わない形で伝搬させる）。

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
| 内部経路 | KnowledgeHub常駐エージェント | 内部ネットワーク経由でmcpへ直接到達（外部リバースプロキシをバイパス）。トランスポートはStreamable HTTP |
| 外部経路 | 外部MCPクライアント（Claude Code等） | 外部リバースプロキシ経由（`mcp.` FQDN）でmcpへ到達。トランスポートはStreamable HTTP |
| ローカル起動 | 同一マシン上のMCPクライアント | バイナリを直接起動しstdioで接続（デバッグ・単体検証用） |

典型的な利用フロー: エージェントが `search_items`/`get_item_text` で材料を収集 → 自身のLLMで要約/wiki/embeddingを生成 → `upsert_knowledge` で結果を書き戻す（大量データの処理は `enqueue_job` でworkerに委譲）。

## やらなくていいこと
- 要約/wiki/embeddingの生成ロジック自体をmcp内に実装すること（生成はエージェント側の責務）
- PostgreSQLへの直接アクセス（必ず `MediaVault-api` の `/api` を経由する）
- ユーザー管理・認証機能（単一ユーザー前提。外部経路のアクセス制御はインフラ設計側のリバースプロキシ/ネットワーク境界に委ねる）
- MCPプロトコル自体の実装（`rmcp` に委ねる。プロトコルレイヤを自前で書かない）
- HTTPクライアント・APIレスポンス型の再実装（`api-client-lib` を使う）
- MCPのResources / Promptsの提供（初期スコープはToolsのみ）

## 未確定事項
- `rmcp` のバージョン固定方針（crates.io の安定版か、git dev チャンネルか）
- 外部経路での認証をリバースプロキシ任せにするか、MCP側でAPIキーヘッダを検証するか
- `get_item_text` の対応API（`MediaVault-api` 側に全文取得エンドポイントを新設する必要がある）
