# TASK-0007 設定作業実行

## 作業概要

- **タスクID**: TASK-0007
- **作業内容**: README.mdへの統合環境起動手順ドキュメント追記
- **実行日時**: 2026-07-02
- **実行者**: Claude Code (kairo-loop)

## 設計文書参照

- **参照文書**: docs/tasks/backend-frontend-integration/TASK-0007.md
- **関連要件**: NFR-201

## 実行した作業

### 1. README.md「統合環境（frontend + backend + db）の起動」セクション追加

- ルート`.env.example`/`backend/.env.example`からのコピー手順
- `docker compose up -d --build`実行手順
- `docker compose ps`での起動確認手順
- アクセスURL（`http://localhost`）
- backend/db非公開の説明
- DBマイグレーション初回適用に関する補足（TASK-0006の実施過程で判明した既存の運用要件）
- `docker compose down`での停止手順

既存の`backend`単体起動セクション（`cd backend && docker compose up -d db`）は変更していない。

## 作業結果

- [x] README.mdに統合環境の起動手順を追記
- [x] `.env`準備手順を記載
- [x] アクセスURL（`http://localhost`）を記載

## 次のステップ

- `/tsumiki:direct-verify` を実行して手順通りに起動できるか確認
