# backend

## 開発コマンド

### ビルド
```bash
cargo build -p mediavault-api
```

### テスト実行
```bash
cargo test --workspace
```

### Docker Compose（Postgres + アプリ）
```bash
cd backend
cp .env.example .env
docker compose up -d db
docker compose ps
```
