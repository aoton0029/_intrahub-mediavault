# backend

## 開発コマンド

### ビルド
```bash
cargo build -p mediavault-api
```

### テスト実行
```bash
cargo test --workspace
# DB依存の統合テスト（#[ignore]付き、TASK-0032）も含めて実行する場合
cargo test --workspace --all-targets -- --include-ignored
```

### Lint・フォーマット（CI: TASK-0033で同一コマンドを実行）
```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

### Docker Compose（Postgres + アプリ）
```bash
cd backend
cp .env.example .env
docker compose up -d db
docker compose ps
```

### CI（GitHub Actions）
`.github/workflows/backend-ci.yml`が`push`（main）・`pull_request`をトリガーに、Postgresサービスコンテナ起動→`sqlx migrate run`→`cargo fmt --check`→`cargo clippy -D warnings`→`cargo test --include-ignored`を実行する。
