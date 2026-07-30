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

### 起動（Postgresはコンテナ、apiはホストで cargo run）
backend単体用の compose ファイルは無い。Postgresはリポジトリルートの `docker-compose.test.yml` から起動する。

```bash
# リポジトリルートで
docker compose -f docker-compose.test.yml --env-file .env.test up -d db

cd backend
cp .env.example .env             # DATABASE_URL は localhost を指す
cargo run -p mediavault-api      # main.rs が dotenvy で backend/.env を読む
```

環境変数ファイルの責務分担はルートの `README.md`「環境変数ファイルの構成」を参照。

