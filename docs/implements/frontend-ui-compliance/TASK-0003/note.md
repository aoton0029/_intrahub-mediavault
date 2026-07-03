# TASK-0003 実装ノート: Buttonコンポーネントのvariant拡張

## 1. 技術スタック
- React 18.3+ / TypeScript / Vite / Tailwind CSS v4 + shadcn/ui
- variant管理: `class-variance-authority` (cva)
- 参照元: frontend/src/components/ui/button.tsx, frontend/package.json

## 2. 開発ルール
- 既存の `frontend/src/components/ui/` shadcnコンポーネントを拡張する方針（新規コンポーネント作成は禁止）
- 既存shadcn標準variant（outline/secondary/destructive/link）は削除しない
- 参照元: docs/tasks/frontend-ui-compliance/TASK-0003.md

## 3. 関連実装
- テストパターン参考: frontend/src/components/common/ConfirmDialog.test.tsx（vitest + @testing-library/react、TC-ID命名規則）
- Buttonの既存利用箇所: frontend/src/components/common/ConfirmDialog.tsx, frontend/src/pages/SettingsPage.tsx 等15ファイル

## 4. 設計文書
- `.btn`系クラス定義: docs/frontend/ui/_shared.css（.btn, .btn-accent, .btn-ghost, .btn-sm, .btn-danger）
- デザイントークン: frontend/src/index.css（--bg-surface, --brand-accent, --accent-strong, --danger, --border, --text-primary, --text-muted 等、TASK-0001/0002で導入済み）
- Tailwind @theme マッピング: frontend/src/index.css内 `--color-card` = `--bg-surface`, `--color-destructive` = `--danger` 等

## 5. テスト関連情報
- テストフレームワーク: Vitest（vitest.config.ts, environment: jsdom, setupFiles: src/test/setup.ts）
- 実行コマンド: `yarn test`（vitest run）
- 新規作成: frontend/src/components/ui/button.test.tsx

## 6. 注意事項
- cvaのvariantキー名（accent/ghost/danger等）は資料に明記がなく、_shared.cssのクラス名から妥当な推測で命名した
- 既存の `default` variant はshadcn標準の accent色（--brand-accent）から `.btn` 基本配色（--bg-surface/--border/--text-primary）へ変更し、新設した `accent` variant がaccent色を引き継ぐ形とした（完了条件の「デフォルトvariantが.btnの基本スタイルを再現」を優先）
- 既存の `size="sm"` は元々 shadcn標準の別サイズ定義だったため、`.btn-sm`相当（padding 4px 10px, font-size 12px）に上書きした
