# バックエンド技術スタック定義

## 🔧 生成情報
- **生成日**: 2026-06-21
- **生成ツール**: init-tech-stack
- **対象**: MediaVault バックエンド（[backend/docs/PRD.md](./PRD.md) 準拠）
- **プロジェクトタイプ**: API/バックエンド（セルフホスト・単一ユーザー）

## 🎯 プロジェクト要件サマリー
- **想定ユーザー**: 単一ユーザー（認証・ログイン機能なし）
- **デプロイ**: Docker（セルフホスト）
- **外部連携**: ファイルサーバー監視・巡回バッチは別プロセス・別言語（Python等）でも可、HTTP経由で連携

## ⚙️ バックエンド
- **言語**: Rust（最新stable）
- **Webフレームワーク**: Axum
- **DBアクセス**: sqlx（コンパイル時SQLチェック、async対応、Rustエコシステムでの実績重視）
- **データベース**: PostgreSQL（Dockerコンテナ）
- **ワークスペース構成**: Cargo workspace で `mediavault`（API本体）と `api-client-lib`（外部API連携・データモデル）を同一リポジトリで管理
- **内部REST API認証**: APIキー固定値 + ヘッダー検証（単一ユーザー前提の簡易認証。`Authorization` ヘッダー等でAPIキーを検証するミドルウェアをAxumに実装）

### 選択理由
- sqlxはコンパイル時にSQLを検証でき、PRDの正規化されたテーブル構成（items + 詳細テーブルJOIN）との相性が良い
- 単一ユーザー運用のためJWT等の複雑な認証基盤は不要、APIキー検証で十分
- ファイルサーバー監視・巡回バッチ等は別言語（Python等）で動かす想定のため、本体はRustに専念し、連携はHTTP API経由とする

## 💾 データベース設計
- **メインDB**: PostgreSQL（Docker）
- **マイグレーション管理**: sqlx-cli（`sqlx migrate`）
- **ファイルストレージ**: アプリコンテナ外のファイルサーバーHDDにバインドマウント
  - **MediaVault専用領域**（アップロード書込先）: `STORAGE_ROOT`（`/srv/mediavault`）配下のアップロード領域 `files/`（`STORAGE_SUBDIR_FILES` で上書き可）に、**アイテムIDごとのフォルダ**を切って保存する（`files/{item_id}/{uuid}.{ext}`）
  - **実データ領域**（読み取り専用）: `/srv/anime`・`/srv/live-action`・`/srv/manga`。Jellyfin・Calibre-Webも参照する
  - DB（`item_files.path`）は、アップロードなら**保存先ベースディレクトリからの相対パス**、リンク登録なら**実データ領域の絶対パス**を保持する。バイナリ本体はAPI外で管理（[item-files.md](./mediavault-api/item-files.md)）

## 🛠️ 開発環境
- **コンテナ**: Docker + Docker Compose（Postgresコンテナ + アプリコンテナをローカルで再現）
- **パッケージマネージャー**: Cargo（workspace構成）
- **マイグレーションCLI**: sqlx-cli
- **CI/CD**: GitHub Actions（テスト・Lint・ビルドの自動化）
- **Lint/フォーマット**: `cargo clippy` + `cargo fmt`
- **テスト**: `cargo test`（sqlxのテスト用DB接続を利用した統合テストを含む）

## 🔒 セキュリティ
- **認証**: APIキー1本による簡易認証（ユーザー管理機能は持たない）
- **通信**: セルフホスト環境のため、必要に応じてリバースプロキシ（Caddy/nginx等）でHTTPS終端
- **環境変数**: APIキー・外部APIキー（Jikan/TMDb/NDL等）は`.env`で管理し、リポジトリにコミットしない
- **入力検証**: Axum側でのリクエストボディ・パスパラメータのバリデーション

## 📁 推奨ディレクトリ構造（Cargoワークスペース）

```
backend/
├── Cargo.toml                # workspace定義
├── mediavault-api/            # API本体（Axum）
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── routes/            # エンドポイント定義
│   │   ├── handlers/          # ハンドラ
│   │   ├── db/                # sqlxクエリ・接続管理
│   │   ├── middleware/        # APIキー検証等
│   │   └── models/            # DBモデル
│   ├── migrations/            # sqlx-cliマイグレーション
│   └── tests/
├── api-client-lib/             # 外部API連携・共有データモデル
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── jikan.rs
│       ├── tmdb.rs
│       ├── ndl.rs
│       ├── openlibrary.rs
│       ├── steam.rs
│       ├── igdb.rs
│       └── models.rs
├── docs/
│   ├── PRD.md
│   └── tech-stack.md          # このファイル
└── docker-compose.yml
```

## 🚀 セットアップ手順（想定）

### 1. 開発環境準備
```bash
cargo install sqlx-cli --no-default-features --features postgres
docker compose up -d db
sqlx migrate run
```

### 2. 主要コマンド
```bash
cargo run -p mediavault-api      # API起動
cargo test                       # 全テスト実行
cargo clippy --all-targets       # Lint
cargo fmt                        # フォーマット
sqlx migrate add <name>          # マイグレーション追加
```

## 📝 カスタマイズ方法
このファイルはPRD・実装の進行に応じて更新してください。特に外部API連携先（Jikan/TMDb/NDL等）の追加・変更、インポート/エクスポート形式の追加時はapi-client-libの構成見直しが必要です。

## 🔄 更新履歴
- 2026-06-21: 初回生成（init-tech-stackにより自動生成、[backend/docs/PRD.md](./PRD.md)準拠）
