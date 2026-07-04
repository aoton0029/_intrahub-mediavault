# frontend (MediaVault)


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
