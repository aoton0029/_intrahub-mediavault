# TASK-0001 設定確認・動作テスト

## 確認概要

- **タスクID**: TASK-0001
- **確認内容**: `frontend/Dockerfile` のビルド・起動確認
- **実行日時**: 2026-07-02
- **実行者**: Claude Code (kairo-loop)

## 設定確認結果・動作テスト結果

### 1. イメージビルド成功

```bash
docker build -f frontend/Dockerfile -t mediavault-frontend-test frontend
```

- 初回ビルドは `yarn install --frozen-lockfile` が `Invariant Violation: could not find a copy of vite to link in /app/node_modules/vitest/node_modules` で失敗した。
- 原因調査の結果、`yarn.lock` 内で `vite` が2バージョン（root向け `8.0.16` とvitestのpeer依存解決による `8.1.1`）に解決されており、yarn classic (v1) のリンク処理がこの二重解決に対応できず失敗する既知の挙動であることが判明した。
- `frontend/package.json` に `"resolutions": { "vite": "^8.0.12" }` を追加し `yarn install` でロックファイルを再生成することで、`vite` の解決が単一バージョンに収束し解消した。
- 修正後、`docker build` は成功。

### 2. nginxコンテナ単体起動確認

```bash
docker run -d --name mv-frontend-test -p 8081:80 mediavault-frontend-test
curl -s -o /dev/null -w "%{http_code}\n" http://localhost:8081/
# => 200
```

## 発見された問題と解決

### 問題1: yarn.lockでのvite二重解決によるdocker build失敗

- **発見方法**: `docker build` 実行時のエラーログ
- **重要度**: 高（ビルド自体が失敗するため）
- **自動解決**: `frontend/package.json` に `resolutions.vite` を追加し `yarn install` でロックファイルを再生成
- **解決結果**: 解決済み（`yarn vitest run` 182テスト全通過、`docker build` 成功を確認）

## 全体的な確認結果

- [x] `frontend/Dockerfile` がマルチステージビルドで構成されている
- [x] `docker build -f frontend/Dockerfile frontend` がエラーなく成功する
- [x] ビルド成果物（`dist/`）が `/usr/share/nginx/html` に配置される
- [x] 単体起動確認（`curl` で200応答）

## 次のステップ

- TASK-0006にて `docker compose up` によるフル統合起動確認を実施
