# backend

## 開発コマンド

### ビルド
```bash
cargo build -p mediavault-api
```

### テスト実行
```bash
cargo test --workspace
# DB依存の統合テスト（#[ignore]付き）も含めて実行する場合
cargo test --workspace --all-targets -- --include-ignored
```

### Lint・フォーマット
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

