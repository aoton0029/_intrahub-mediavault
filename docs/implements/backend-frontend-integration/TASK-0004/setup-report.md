# TASK-0004 設定作業実行

## 作業概要

- **タスクID**: TASK-0004
- **作業内容**: ルート統合用 `docker-compose.yml` 新規作成（3コンテナ・ネットワーク分離）
- **実行日時**: 2026-07-02
- **実行者**: Claude Code (kairo-loop)

## 設計文書参照

- **参照文書**: docs/tasks/backend-frontend-integration/TASK-0004.md
- **関連要件**: REQ-001, REQ-002, REQ-003, REQ-101, REQ-201, REQ-202, REQ-301, REQ-401, REQ-402

## 実行した作業

### 1. ルート `docker-compose.yml` の新規作成

`db`（非公開）・`backend`（非公開、db healthy待ち）・`frontend`（80番ポート公開）の3サービスを定義した。

- `db`: 既存 `backend/docker-compose.yml` の定義を流用し `ports:` を削除
- `backend`: `build.context: ./backend` を指定し `ports:` なし。`env_file: ./backend/.env` を参照
- `frontend`: `build.context: ./frontend` を指定し、`80:80` を公開

### 2. 既存 `backend/docker-compose.yml` の無変更確認

`git diff backend/docker-compose.yml` が空であることを確認（本タスクでは同ファイルを一切変更していない）。

## 作業結果

- [x] ルート `docker-compose.yml` の作成完了（3サービス定義）
- [x] `backend`・`db` に `ports:` キーなし
- [x] `frontend` がホストの80番ポートに公開
- [x] `backend` が `db` の `service_healthy` を待って起動
- [x] `backend/docker-compose.yml` に差分なし

## 次のステップ

- `/tsumiki:direct-verify` を実行して設定を確認（`docker compose config` によるバリデーション等）
