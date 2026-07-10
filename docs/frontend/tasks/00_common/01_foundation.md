# 01. Foundation（トークン・グローバルCSS・アイコン基盤）

対応: 設計書 §2, §4

## 前提ファイル

- 参照: `docs/frontend/ui/_shared.css`, `docs/frontend/design/00_common.md`
- 出力: `frontend/src/index.css`, `frontend/src/lib/cn.ts`, `frontend/vite.config.ts`（Tailwind v4プラグイン確認）
- 参照範囲はこの節に列挙したファイルとそこから直接importされるファイルに限る。それ以外のファイルを探すための横断的な探索は行わない。

## タスク一覧

- [x] `frontend/src/index.css` に `@import "tailwindcss";` と `@theme { ... }` ブロックを設計書 §2 のとおり定義する（color / font / layout トークン全量）
- [x] `:root[data-theme="light"]` オーバーライドブロックを設計書 §2 のとおり定義する
- [x] ダークをデフォルト（`:root` 直下の値）として採用し、`dark:` バリアントではなく `[data-theme="light"]` セレクタ方式で統一する（設計書§2の【要確認】方針に準拠）
- [x] Google Fonts等の外部フォント読込ではなく、`@fontsource-variable/geist` など既存依存を踏まえて `--font-ui` / `--font-display` / `--font-mono` の実フォント調達方法を決定する（Inter/Source Serif 4/JetBrains Monoが未導入なら追加 or 代替を選定し、Codexメモに記載）
- [x] `frontend/src/lib/cn.ts` を作成する（`clsx` + `tailwind-merge` を使った `cn()` ヘルパー、`tailwind-merge`は導入済み）
- [x] `main.tsx`（または相当のエントリ）で `index.css` をimportする
- [x] `react-icons/fi` の使用方針をコードコメントまたはREADMEに明記し、`lucide-react` は使用しない旨をlintルールやコードレビューで担保できるようにする（ESLintルール追加は任意、Codexメモに判断を記載）

## テストリスト

- [x] `yarn build` でTailwindのビルドエラーが出ないことを確認する
- [x] 簡易コンポーネントに `bg-bg-app` `text-text-primary` 等のユーティリティクラスを当て、ブラウザ/vitest上で意図した色（ダーク値）が適用されることを確認する
- [x] `document.documentElement.setAttribute('data-theme', 'light')` を行った状態で同コンポーネントの色がライト値に切り替わることを確認する（vitest + jsdom、またはStorybook等の手動確認）
- [x] `cn()` のユニットテスト（`frontend/src/lib/cn.test.ts`）: 重複クラスのマージ、条件付きクラスの結合を検証する

> Codexメモ: フォントは既存依存のみで完結させるため `Geist Variable` を UI / Display に採用し、Mono は system fallback とした。
> Codexメモ: `react-icons/fi` 統一は `frontend/README.md` と `frontend/eslint.config.js` の `no-restricted-imports` で担保した。
