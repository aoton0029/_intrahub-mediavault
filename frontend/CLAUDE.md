# frontend (MediaVault)

React 18.3+ / TypeScript / Vite / Tailwind CSS v4 + shadcn/ui / TanStack Query v5 / react-router-dom v7。

## 開発コマンド

### テスト実行
```bash
# ユニットテスト（Vitest）
yarn test

# ウォッチモード
yarn test:watch

# E2Eテスト（Playwright、初回は npx playwright install が必要）
yarn test:e2e
```

### アプリケーション実行
```bash
# 開発サーバー起動
yarn dev

# ビルド（型チェック含む）
yarn build

# Lint
yarn lint
```

### shadcn/uiコンポーネント追加
```bash
npx shadcn@latest add <component-name>
```

### Dockerイメージビルド（nginx配信、TASK-0001/0002）
```bash
docker build -f frontend/Dockerfile -t mediavault-frontend-test frontend
docker run -d --name mv-frontend-test -p 8081:80 mediavault-frontend-test
curl http://localhost:8081/
docker rm -f mv-frontend-test
```

**注意**: `package.json` に `resolutions.vite` を設定している。`vitest` のpeer依存で解決される `vite` バージョンとroot `devDependencies` の `vite` バージョンが分岐すると、yarn classic (v1) の `yarn install --frozen-lockfile` が `Invariant Violation: could not find a copy of vite to link` で失敗するため（Docker等のクリーンインストール時に顕在化）。`vite` のバージョン範囲を変更する場合は `yarn install` でロックファイルを再生成し、`docker build` で確認すること。
