# フロントエンド技術スタック定義

## 🔧 生成情報
- **生成日**: 2026-06-22
- **生成ツール**: init-tech-stack
- **プロジェクトタイプ**: Webアプリケーション（フロントエンド、単一ユーザー・セルフホスト）
- **対象PRD**: [docs/frontend/PRD.md](PRD.md)

## 🚀 フロントエンド
- **フレームワーク**: React 18.3+
- **言語**: TypeScript 5.7+
- **ビルドツール**: Vite 6
- **状態管理**: TanStack Query 5（サーバー状態） + React内蔵state/useContext（UI状態）
- **ルーティング**: React Router v7
- **UI/スタイリング**: Tailwind CSS 4 + shadcn/ui

### 選択理由
- React + TypeScriptはルートPRDで指定済みの確定スタック
- 単一ユーザー・セルフホスト前提のため、グローバル状態管理は最小限（Redux等は過剰）。サーバーから取得するメタデータ（コレクション一覧・詳細等）はTanStack Queryでキャッシュ・再取得を簡潔に扱う
- 一覧・絞り込み・カード表示・フォーム（手動追加・編集）など画面数が多いため、shadcn/uiのコンポーネント資産とTailwindの組み合わせで構築速度を優先
- Viteは開発体験・ビルド速度に優れ、SPA構成（SSR不要・認証画面なし）に最適

## 🛠️ 開発環境
- **パッケージマネージャー**: pnpm 9+
- **テストフレームワーク**: Vitest 2+ + Testing Library
- **E2Eテスト**: Playwright 1.49+
- **リンター・フォーマッター**: ESLint 9+ + Prettier 3+
- **型チェック**: tsc (TypeScript Compiler)

### 選択理由
- Vite + Vitestは同一エコシステムで設定を共通化でき、高速に統合可能
- 一覧フィルタ・フォーム入力・画面遷移など実際の操作確認が重要な画面が多いため、PlaywrightでE2Eをカバー

## 📊 品質基準
- **テストカバレッジ**: 主要なフォーム・一覧フィルタ・詳細表示ロジックを優先的にカバー
- **型安全性**: API（バックエンド）から返るレスポンス型はバックエンドのスキーマ定義と同期
- **アクセシビリティ**: 基本的なWCAG 2.1 AA準拠（フォーム・ナビゲーション中心）

## 📁 推奨ディレクトリ構造

```
frontend/
├── src/
│   ├── components/      # 共通UIコンポーネント（shadcn/ui含む）
│   ├── pages/           # 画面コンポーネント（一覧/詳細/検索追加/手動追加/マイリスト/タグ管理/スタッフ管理/設定）
│   ├── features/        # 機能単位のロジック（フィルタ、関連付け、視聴記録等）
│   ├── hooks/           # カスタムフック
│   ├── api/             # APIクライアント・TanStack Queryフック
│   ├── types/           # TypeScript型定義
│   ├── lib/             # ユーティリティ
│   └── App.tsx
├── public/
├── tests/
├── package.json
├── tsconfig.json
├── vite.config.ts
└── tailwind.config.ts
```

## 🚀 セットアップ手順

### 1. 開発環境準備
```bash
pnpm install
pnpm dev
```

### 2. 主要コマンド
```bash
pnpm dev        # 開発サーバー起動
pnpm build      # ビルド
pnpm test       # Vitest実行
pnpm test:e2e   # Playwright実行
pnpm lint       # ESLint
```

## 🔄 更新履歴
- 2026-06-22: 初回生成（init-tech-stackにより自動生成）
