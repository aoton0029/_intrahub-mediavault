# TASK-0033 設定作業実行

## 作業概要

- **タスクID**: TASK-0033
- **作業内容**: GitHub ActionsによるバックエンドCIパイプライン構築（`.github/workflows/backend-ci.yml`）
- **実行日時**: 2026-06-29
- **実行者**: Claude Code（kairo-implement / direct-setup）

## 設計文書参照

- **参照文書**: docs/tasks/mediavault-backend/TASK-0033.md, backend/docker-compose.yml, backend/.env.example, backend/Cargo.toml, backend/mediavault-api/migrations/
- **関連タスク**: TASK-0032（統合テスト、`#[ignore]`付きDB依存テストを内包）

## 実行した作業

### 1. 既存環境構成の調査

- `backend/docker-compose.yml`: `postgres:16`サービス、`POSTGRES_USER/PASSWORD/DB`環境変数、`pg_isready`ヘルスチェックを確認。CIのPostgresバージョン・ヘルスチェック方式をこれに揃えた。
- `backend/.env.example`: `DATABASE_URL=postgresql://mediavault:changeme@db:5432/mediavault`、`INTERNAL_API_KEY`を確認。CI用環境変数はホスト名を`db`→`localhost`に変更し同じ認証情報を採用（CIでは`services:`コンテナがホスト側ポートにマップされるため）。
- `backend/Cargo.toml`: workspaceメンバーは`mediavault-api`・`api-client-lib`の2クレート。`cargo fmt --all` / `cargo clippy --all-targets --all-features` / `cargo test --workspace --all-targets`をworkspace全体に対して実行する構成とした。
- `backend/mediavault-api/migrations/`: sqlxマイグレーションファイル4件（`*.up.sql`/`*.down.sql`）を確認。`.sqlx`オフラインメタデータディレクトリは存在しないため、`cargo build`/`clippy`時にsqlxマクロがDATABASE_URLへの実接続でクエリ検証を行う前提（マイグレーション適用後でなければビルド自体が失敗する点に注意し、ワークフロー内でマイグレーション適用をlint/test実行より前に配置した）。
- `.github/workflows/`: 既存ワークフローなし（新規作成）。
- `rust-toolchain.toml`: 存在せず。`dtolnay/rust-toolchain@stable`でstable版を明示的にセットアップする構成とした。

### 2. ワークフローファイルの作成

**作成ファイル**: `.github/workflows/backend-ci.yml`

主な構成:
- トリガー: `push`（`main`ブランチ）・`pull_request`（全般）
- `services.postgres`: `postgres:16`、ポート5432をホストへマップ、`pg_isready`ヘルスチェック
- ジョブ環境変数: `DATABASE_URL`（localhost向け）、`INTERNAL_API_KEY`（CI専用ダミー値、実APIキー不要）
- ステップ:
  1. `actions/checkout@v4`
  2. `dtolnay/rust-toolchain@stable`（`rustfmt`, `clippy`コンポーネント込み）
  3. `Swatinem/rust-cache@v2`（`~/.cargo`・`target/`キャッシュ、`workspaces: backend`指定）
  4. `cargo install sqlx-cli --no-default-features --features postgres,rustls --locked`
  5. `sqlx migrate run --source mediavault-api/migrations`
  6. `cargo fmt --all -- --check`
  7. `cargo clippy --all-targets --all-features -- -D warnings`
  8. `cargo test --workspace --all-targets -- --include-ignored`（TASK-0032の`#[ignore]`付き統合テストも含めて実行）

### 3. 依存関係のインストール

CI上でのみ`sqlx-cli`をインストールする設定とした（ローカル開発環境へのインストールは対象外、既存`docker-compose.yml`/開発フローに影響なし）。

### 4. データベースの初期化

CIジョブ内で`sqlx migrate run`を実行し、`postgres`サービスコンテナへマイグレーションを適用する設定とした（ローカルDBへの変更は行っていない）。

## 作業結果

- [x] `.github/workflows/backend-ci.yml`の作成
- [x] PostgreSQLサービスコンテナ設定
- [x] `sqlx migrate run`によるマイグレーション適用ステップ
- [x] `cargo fmt --check`ステップ
- [x] `cargo clippy --all-targets --all-features -- -D warnings`ステップ
- [x] `cargo test --workspace --all-targets -- --include-ignored`ステップ（TASK-0032統合テスト含む）
- [x] `main`へのpush・pull_request全般をトリガーに設定
- [x] `Swatinem/rust-cache`によるキャッシュ設定

## 遭遇した問題と解決方法

### 問題1: sqlxのオフラインメタデータ（`.sqlx/`）が存在しない

- **発生状況**: `cargo build`/`clippy`実行前にDBへのマイグレーション適用が必須であることが判明（オフラインモード未導入のため）。
- **解決方法**: ワークフロー内で`sqlx migrate run`を`fmt`/`clippy`/`test`の各ステップより前に配置し、ビルド時点でDBスキーマが適用済みの状態を保証した。

### 問題2: プロジェクトルートにREADME.mdが存在しない

- **発生状況**: direct-setupの手順ではREADME.mdへの記録が推奨されているが、本リポジトリにはまだREADME.mdが存在しない（TASK-0034「README・起動手順整備」が未着手のため）。
- **解決方法**: README.md新規作成はTASK-0034の責務であるため本タスクでは作成せず、CI関連の情報は本setup-report.mdに記録した。TASK-0034着手時にCIバッジ・実行方法の追記を推奨事項として申し送る。

## 次のステップ

- `/tsumiki:direct-verify` を実行して設定を確認（YAML構文・既存ファイルとの整合性チェック）
- 実際のpush/PRでのCI実行確認はGitHub上でのみ可能なため、ローカルではYAML検証とコマンド構文の妥当性確認までとする
