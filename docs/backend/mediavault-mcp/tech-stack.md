# MediaVault-mcp 技術スタック定義

## 🔧 生成情報
- **生成日**: 2026-08-07
- **生成ツール**: init-tech-stack
- **対象**: MediaVault-mcp（[PRD.md](./PRD.md) 準拠）
- **上位定義**: [バックエンド技術スタック](../tech-stack.md)
- **プロジェクトタイプ**: API/バックエンド（MCPサーバー・セルフホスト・単一ユーザー）

## 🎯 プロジェクト要件サマリー
- **想定ユーザー**: 単一ユーザー（コレクション所有者）＋ KnowledgeHub常駐エージェント
- **役割**: AIエージェントからMediaVaultを検索・登録・整理・更新するための窓口
- **データ所有**: 持たない。読み書きはすべて MediaVault-api 経由（PostgreSQL直接アクセス禁止）
- **トランスポート**: MVPは Streamable HTTP のみ。stdio は第2段階
- **デプロイ**: ミニPC上のDockerコンテナに常駐
- **性能**: 単一ユーザー・低同時実行。レスポンスサイズと呼び出し回数の最小化を優先

## ⚙️ MCPサーバー本体
- **言語**: Rust（edition 2024、最新stable）
- **MCP SDK**: `rmcp` 3.1系（最新安定版 3.1.1 / 2026-08-05 公開時点）
  - 有効化するfeature: `server`, `transport-streamable-http-server`, `macros`（axum連携用featureを含む）
  - `Cargo.lock` でバージョンをピンする。MCP仕様への追随が速いクレートのため `*` やワイルドカード指定はしない
- **HTTPサーバー**: axum 0.8 系（mediavault-api と同一）
- **非同期ランタイム**: tokio 1.x（`full`）
- **ワークスペース**: `backend/Cargo.toml` の members に `mediavault-mcp` を追加

### 選択理由
- `rmcp` の `#[tool]` / `#[tool_router]` マクロで Rust 構造体から JSON Schema を自動生成でき、PRD §14「APIレスポンス型をMCP内で重複定義しない」を満たせる
- axum / tokio / serde を mediavault-api と揃えることで、ワークスペース内の依存とビルドキャッシュを共有できる
- トランスポートが差し替え可能なため、第2段階の stdio 追加時にツール実装を変更せずに済む（PRD §11「stdioとStreamable HTTPでツールの意味・入出力を同一にする」）

## 🌐 MediaVault-api クライアント
- **HTTPクライアント**: `reqwest` 0.12（`json` feature）
- **配置**: `mediavault-mcp` クレート内の `api/` モジュールに自前実装する
- **認証**: 内部書き込みAPI呼び出し時は `INTERNAL_API_KEY` をヘッダーに付与。キーはツール引数・ツール結果・ログへ出力しない
- **エラー分類**: `ConnectionError` / `AuthError` / `ApiError { code, message }` を区別して返す（PRD §11）。MediaVault-api のエラーコードとメッセージを失わない
- **タイムアウト・リトライ**: 接続・読み取りタイムアウトを設定。冪等な GET のみ限定的にリトライし、書き込みは自動リトライしない

### `api-client-lib` との関係
`api-client-lib` は Jikan / TMDb / NDL / OpenLibrary / Steam / IGDB など**外部カタログ**のクライアントであり、MediaVault-api 自身を叩くクライアントは持たない。したがって:

- 外部カタログ検索に関わる型は `api-client-lib` を再利用する
- MediaVault-api の呼び出しクライアントと型は `mediavault-mcp` 内に実装する

将来、mediavault-api 側の型を共有クレートへ切り出す価値が出た場合は、そのタイミングで再検討する。

## 🔒 セキュリティ
- **Streamable HTTP の認証**: 静的Bearerトークンを MCPプロセス自身が検証する（PRD §10 の仮決定を採用）
  - トークンは環境変数 `MCP_AUTH_TOKEN` から読み込む
  - tower ミドルウェアで `Authorization: Bearer <token>` を検証し、`subtle` クレートで定数時間比較する
  - リバースプロキシでは認証を終端しない（内部経路がプロキシをバイパスするため）
  - 未設定・空文字の場合は起動を失敗させ、無認証で公開されることを防ぐ
- **認証除外**: プロセス死活監視用の `/healthz` のみ認証対象外とする。MCPツールの `health`（MediaVault-api 到達性確認）とは別物であり、後者は認証必須
- **秘密情報**: `INTERNAL_API_KEY`、外部APIキー、`MCP_AUTH_TOKEN` は `.env` / compose の環境変数で管理し、リポジトリにコミットしない
- **ツールの区別**: 読み取り／追記・更新をMCPのツールメタデータ（annotations）で区別する。削除・物理ファイル操作ツールはMVPで公開しない
- **入力検証**: ツール引数はスキーマで検証し、URL・ID・列挙値はサーバー側で再検証する

## 🛠️ 開発環境
- **コンテナ**: Docker + Docker Compose。`mediavault-mcp` を独立サービス・独立イメージとして定義する
  - マルチステージビルド（`cargo build --release -p mediavault-mcp`）
  - MediaVault-api とはコンテナ間HTTPで通信し、障害分離と個別再起動を可能にする
- **パッケージマネージャー**: Cargo（既存 workspace に追加）
- **設定**: `dotenvy` + 環境変数（`MEDIAVAULT_API_BASE_URL`、`INTERNAL_API_KEY`、`MCP_AUTH_TOKEN`、`MCP_BIND_ADDR`）
- **ログ**: `tracing` + `tracing-subscriber`（`env-filter`）。ツール名・所要時間・結果種別を構造化ログに出す。秘密情報は出さない
- **エラー型**: `thiserror`（ライブラリ層）+ `anyhow`（起動・組み立て層）

### テスト
- **単体テスト**: `cargo test`。ツールごとの入出力、曖昧解決時に書き込まないこと、部分失敗の返却、冪等性を検証する
- **統合テスト**: `wiremock` 0.6 で MediaVault-api をモックし、MCPツール単位のフローを検証する（mediavault-api と同じ構成）
- **実API結合テストは用意しない**（今回の方針）。必要になった段階で docker compose ベースのE2Eを追加検討する
- **Lint/フォーマット**: `cargo clippy --all-targets -- -D warnings` / `cargo fmt`
- **CI/CD**: GitHub Actions（fmt → clippy → test → build）

## 📁 推奨ディレクトリ構造

```
backend/
├── Cargo.toml                    # workspace（members に mediavault-mcp を追加）
├── mediavault-api/
├── api-client-lib/
└── mediavault-mcp/
    ├── Cargo.toml
    ├── Dockerfile
    ├── src/
    │   ├── main.rs               # 設定読み込み・axum起動・/mcp マウント
    │   ├── config.rs             # 環境変数
    │   ├── auth.rs               # Bearerトークン検証ミドルウェア
    │   ├── server.rs             # rmcp ServerHandler / tool_router
    │   ├── tools/                # 目的単位のMCPツール
    │   │   ├── mod.rs
    │   │   ├── search_library.rs
    │   │   ├── get_item_context.rs
    │   │   ├── search_external_catalog.rs
    │   │   ├── import_external_item.rs
    │   │   ├── create_item.rs
    │   │   ├── update_consumption.rs
    │   │   ├── organize_item.rs
    │   │   ├── relate_items.rs
    │   │   ├── add_access_link.rs
    │   │   ├── collection_overview.rs
    │   │   └── health.rs
    │   ├── api/                  # MediaVault-api クライアント
    │   │   ├── mod.rs
    │   │   ├── client.rs
    │   │   ├── error.rs
    │   │   └── models.rs
    │   ├── resolve/              # 名前→ID解決（タグ/カテゴリ/マイリスト/Item）
    │   └── result/               # 構造化レスポンス・部分失敗表現
    └── tests/                    # wiremock ベースの統合テスト
```

## 🚀 セットアップ手順

### 1. 開発環境準備
```bash
cd backend
cargo add -p mediavault-mcp rmcp --features server,transport-streamable-http-server,macros
docker compose up -d db mediavault-api
```

### 2. 主要コマンド
```bash
cargo run -p mediavault-mcp                       # MCPサーバー起動
cargo test -p mediavault-mcp                      # テスト
cargo clippy -p mediavault-mcp --all-targets -- -D warnings
cargo fmt
docker compose up -d --build mediavault-mcp       # コンテナ起動
```

### 3. 環境変数
```bash
MCP_BIND_ADDR=0.0.0.0:8081
MCP_AUTH_TOKEN=<ランダム生成した十分な長さのトークン>
MEDIAVAULT_API_BASE_URL=http://mediavault-api:8080
INTERNAL_API_KEY=<mediavault-api と同じ値>
```

## 📊 品質基準
- 型安全性: `unsafe` を使わない。ツール入出力はすべて構造体で定義する
- テスト: 曖昧な名称から誤ったItemへ書き込むケースが自動テスト上0件（PRD §12）
- 書き込みツールは対象ID・表示名・変更前後・部分失敗を必ず構造化して返す
- MCPからのDB直接アクセス0件（依存に `sqlx` を入れない）

## 🔄 PRDとの差分
- PRD §14「MediaVault-api呼び出しには `api-client-lib` の型とクライアントを再利用する」を、実態（`api-client-lib` は外部カタログ専用）に合わせて更新済み

## 🔄 更新履歴
- 2026-08-07: 初回生成（init-tech-stack により生成、[PRD.md](./PRD.md) 準拠）
