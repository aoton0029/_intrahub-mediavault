# TASK-0006 Redフェーズ

本タスクはコード実装ではなくインフラの結合テスト実施タスクのため、「失敗するテスト」は「まだ `.env` 未作成・`docker compose up` 未実施の状態」に相当する。

## Red確認

```bash
docker compose ps
# => no configuration file provided もしくは何も起動していない状態（環境未構築のため当然失敗）
```

この状態から `.env`/`backend/.env` を作成し `docker compose up -d` を実行することで、Greenフェーズにて受け入れ基準12件を検証する。
