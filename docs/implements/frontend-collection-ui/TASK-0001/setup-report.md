# TASK-0001 設定作業実行

## 作業概要

- **タスクID**: TASK-0001
- **作業内容**: `frontend/`へのプロジェクト依存パッケージ追加・Tailwind CSS v4/shadcn-ui初期化・テスト基盤（Vitest/Playwright）整備
- **実行日時**: 2026-06-30
- **実行者**: Claude Code (kairo-implement)

## 設計文書参照

- **参照文書**: docs/tasks/frontend-collection-ui/TASK-0001.md, docs/spec/frontend-collection-ui/note.md, docs/design/frontend-collection-ui/architecture.md
- **関連要件**: note.md「技術スタック」「実装状況」

## 実行した作業

### 1. パッケージマネージャの確認

`frontend/`に`package-lock.json`と`yarn.lock`が両方存在したが、note.md「実装状況」記載の`yarn create vite . --template react-ts`の記述、および`yarn.lock`のタイムスタンプがより新しいことからyarnを採用した。

### 2. ランタイム依存の追加

```bash
yarn add react-router-dom@^7 @tanstack/react-query@^5 react-hook-form zod sonner
```

### 3. 開発依存（テスト基盤）の追加

```bash
yarn add -D vitest @testing-library/react @testing-library/jest-dom jsdom @playwright/test
yarn add -D @testing-library/dom  # peerDependency未解決のため追加（yarn classicはpeerDepsを自動インストールしない）
```

### 4. Tailwind CSS v4導入

```bash
yarn add tailwindcss @tailwindcss/vite
```

`vite.config.ts`に`@tailwindcss/vite`プラグインを追加し、`@`エイリアス（`./src`）を設定。`src/index.css`に`@import "tailwindcss";`を追加。

**ESM対応の修正**: `package.json`が`"type": "module"`のため、`vite.config.ts`で`__dirname`を直接使用できない。`path.dirname(fileURLToPath(import.meta.url))`で解決するよう修正した（この修正がないとshadcn CLIのワークスペース設定読み込みが失敗する）。

### 5. shadcn/ui初期化

```bash
npx shadcn@4.10.0 init -t vite -b radix --no-monorepo -p nova -f -y
```

**遭遇した問題**: `npx shadcn@latest init`（v4.12.0）でworkspace設定読み込みエラー「Could not load the workspace config」が発生。原因切り分けの結果、以下2点が必要だった。
1. `vite.config.ts`の`__dirname`をESM互換に修正（上記4参照）
2. ルート`tsconfig.json`にも`compilerOptions.paths`（`@/*` → `./src/*`）を追加（`tsconfig-paths`ライブラリが`references`を辿らず`tsconfig.json`を直接読むため、`tsconfig.app.json`のみの設定では解決できない）

これにより`shadcn@4.10.0`での初期化が成功し、`components.json`・`src/components/ui/button.tsx`・`src/lib/utils.ts`が生成され、`src/index.css`にCSS変数ベースのデザイントークンが追記された（TASK-0002で詳細上書き予定）。

`shadcn`パッケージは`dependencies`に誤って追加されていたため`devDependencies`に移動した。

### 6. TypeScript設定の修正

TypeScript 6系で`baseUrl`が非推奨警告（TS5101）となるためビルドエラーが発生。`moduleResolution: "bundler"`下では`baseUrl`なしで`paths`のみで解決可能なため、`tsconfig.json`・`tsconfig.app.json`双方から`baseUrl`を削除した。

`tsconfig.app.json`の`types`に`vitest/globals`・`@testing-library/jest-dom`を追加し、テストファイルでのグローバルAPI型解決を可能にした。

### 7. テスト基盤設定

- `vitest.config.ts`を新規作成（jsdom環境・`globals: true`・`setupFiles: ['./src/test/setup.ts']`・`tests/e2e/**`を除外）
- `src/test/setup.ts`で`@testing-library/jest-dom/vitest`をインポート
- `playwright.config.ts`を新規作成（`testDir: './tests/e2e'`、`webServer`でdevサーバー自動起動）
- 動作確認用ダミーテスト: `src/App.test.tsx`（Vitest）、`tests/e2e/smoke.spec.ts`（Playwright）

### 8. package.jsonスクリプト整備

```json
{
  "scripts": {
    "dev": "vite",
    "build": "tsc -b && vite build",
    "lint": "eslint .",
    "preview": "vite preview",
    "test": "vitest run",
    "test:watch": "vitest",
    "test:e2e": "playwright test"
  }
}
```

### 9. ESLint調整

shadcn生成の`src/components/ui/button.tsx`が`buttonVariants`を併せてexportするため`react-refresh/only-export-components`に抵触。`src/components/ui/**`に対してのみ当該ルールを無効化する設定を`eslint.config.js`に追加（shadcn/ui生成コンポーネントの一般的なパターンに対する標準的な緩和）。

### 10. Playwrightブラウザインストール

```bash
npx playwright install --with-deps chromium
```

## 作業結果

- [x] react-router-dom, @tanstack/react-query, react-hook-form, zod, sonner を追加
- [x] shadcn/ui初期化によりcomponents.json生成
- [x] Tailwind CSS v4導入（vite.config.ts設定済み）
- [x] Vitest, @testing-library/react, jsdom 追加・jsdom環境設定済み
- [x] Playwright導入・playwright.config.ts生成
- [x] package.jsonにdev/build/test/test:e2e/lintスクリプト定義
- [x] `yarn build` がエラーなく完了
- [x] `yarn test` （ダミーテスト）成功
- [x] `yarn lint` エラーなし
- [x] `yarn test:e2e` （Playwrightデフォルトに準ずるスモークテスト）成功

## 遭遇した問題と解決方法

### 問題1: shadcn CLI「Could not load the workspace config」エラー

- **発生状況**: `npx shadcn@latest init`実行時、components.json書き込み後にワークスペース設定読み込みで失敗
- **原因**: (1) `vite.config.ts`で`__dirname`をESM文脈で直接参照していた、(2) ルート`tsconfig.json`に`paths`設定がなく`tsconfig-paths`ライブラリが`@/*`エイリアスを解決できなかった
- **解決方法**: `import.meta.url`からの`__dirname`算出に修正、ルート`tsconfig.json`にも`paths`を追加

### 問題2: `@testing-library/react`の`screen`等が型解決できずビルドエラー

- **発生状況**: `yarn build`実行時、`src/App.test.tsx`で`screen`がexportされていないという型エラー
- **原因**: `@testing-library/dom`（`@testing-library/react`のpeerDependency）がインストールされていなかった（yarn classicはpeerDepsを自動解決しない）
- **解決方法**: `yarn add -D @testing-library/dom`で明示的に追加

### 問題3: TypeScript 6での`baseUrl`非推奨警告によるビルド失敗

- **発生状況**: `yarn build`実行時、TS5101エラーで停止
- **解決方法**: `moduleResolution: "bundler"`下では不要なため`baseUrl`を削除し`paths`のみ残した

### 問題4: Vitestが`tests/e2e/`配下のPlaywrightテストも実行しようとして失敗

- **発生状況**: `yarn test`実行時、Playwright専用構文（`test()`）をVitestが解釈しようとしてエラー
- **解決方法**: `vitest.config.ts`の`test.exclude`に`tests/e2e/**`を追加

## 次のステップ

- `/tsumiki:direct-verify` を実行して設定を確認
- TASK-0002（Tailwindデザイントークン設定）でshadcn初期化時に生成されたCSS変数を本来のダークテーマトークンへ置き換える
