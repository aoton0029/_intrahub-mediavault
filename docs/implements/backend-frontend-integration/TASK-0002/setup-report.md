# TASK-0002 設定作業実行

## 作業概要

- **タスクID**: TASK-0002
- **作業内容**: frontend/nginx.conf 新規作成（リバースプロキシ + SPAフォールバック）
- **実行日時**: 2026-07-02
- **実行者**: Claude Code (kairo-loop)

## 設計文書参照

- **参照文書**: docs/tasks/backend-frontend-integration/TASK-0002.md, architecture.md, dataflow.md
- **関連要件**: REQ-006, REQ-007, REQ-008

## 実行した作業

### 1. frontend/nginx.conf の新規作成

```nginx
server {
    listen 80;
    server_name _;
    root /usr/share/nginx/html;
    index index.html;

    location /api/ {
        proxy_pass http://backend:8080/api/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    location / {
        try_files $uri /index.html;
    }
}
```

**設定内容**:
- `/api/` へのリクエストを `http://backend:8080/api/` へリバースプロキシ（サービス名 `backend` はTASK-0004で定義）
- それ以外は `try_files $uri /index.html` によりSPAへフォールバック

## 作業結果

- [x] frontend/nginx.conf の作成完了
- [ ] `docker build`（TASK-0001のDockerfile経由）での動作確認（direct-verifyで実施）

## 次のステップ

- `/tsumiki:direct-verify` を実行して設定を確認（TASK-0001のDockerfileとあわせてビルド確認）
