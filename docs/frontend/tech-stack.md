# プロジェクト技術スタック定義

## 🔧 生成情報
- **生成日**: 2026-07-04
- **生成ツール**: init-tech-stack
- **プロジェクトタイプ**: Webアプリケーション（セルフホスト型）
- **チーム規模**: 個人開発
- **開発期間**: プロトタイプ/MVP

## 🎯 プロジェクト要件サマリー
- **パフォーマンス**: 軽負荷（セルフホスト・単一ユーザー前提、レスポンス重視不要）
- **セキュリティ**: 基本レベル（単一ユーザー運用、認証・ログイン機能なし）
- **既存連携**: 既存API連携（MediaVault Backend API / 外部メタデータAPI）
- **技術スキル**: 限定的（学習しながら進める）
- **学習コスト許容度**: バランス重視
- **デプロイ先**: VPS/専用サーバー（セルフホスト）
- **予算**: コスト最小化

## 🚀 フロントエンド
- **フレームワーク**: React 18.3+
- **言語**: TypeScript
- **ビルドツール**: Vite
- **UI/スタイリング**: Tailwind CSS v4 + shadcn/ui
- **データ取得/状態管理**: TanStack Query v5
- **ルーティング**: react-router-dom v7
- **パッケージマネージャー**: Yarn (Classic v1)

### 選択理由
- 個人開発・限定的な技術スキルという前提のもと、既に導入済みかつドキュメント化されているスタックをそのまま踏襲し、学習コストと手戻りを避ける
- shadcn/ui + Tailwind v4 によりUIモック（[docs/frontend/ui](../frontend/ui)）への準拠実装がしやすい
- TanStack Query v5 でAPI通信のキャッシュ・再検証を簡潔に扱える

## ⚙️ バックエンド
- **フレームワーク**: Rust + Actix-web
- **言語**: Rust
- **データベース**: PostgreSQL（DBサーバーコンテナ、Docker Compose管理）
- **マイグレーション**: sqlx migrate
- **APIクライアント**: api-client-lib（Rust、外部メタデータAPI連携用）
- **認証**: なし（単一ユーザー前提、ログイン画面を持たない）

### 選択理由
- 既存実装（`cargo build -p mediavault-api`）をそのまま採用
- 軽負荷・基本セキュリティレベルの要件に対し、型安全性の高いRust+Actix-webは過剰でも不足でもなく妥当
- CIで `cargo fmt` / `cargo clippy -D warnings` / `cargo test --include-ignored` を実行し品質を担保

## 💾 データベース設計
- **メインDB**: PostgreSQL（Docker Compose `db` サービス）
- **ファイルストレージ**: ローカルファイルパス管理（作品ファイル・リンク・トレーラーURLなどをテーブルで管理、[docs/PRD.md](PRD.md) 参照）

### 設計方針
- 単一ユーザー・セルフホスト運用のため、マルチテナント設計は行わない
- media_type・タグ・カテゴリ等でのフィルタリングを前提としたスキーマ

## 🛠️ 開発環境
- **コンテナ**: Docker + Docker Compose（Postgres起動、フロントエンドはnginx配信）
- **パッケージマネージャー**:
  - フロントエンド: Yarn (Classic v1)
  - バックエンド: Cargo
- **テストフレームワーク**:
  - フロントエンド: Vitest（ユニット）
  - バックエンド: `cargo test`（DB依存の統合テストは `--include-ignored`）
- **E2Eテスト**: Playwright（`yarn test:e2e`、初回 `npx playwright install` 必要）
- **リンター・フォーマッター**:
  - フロントエンド: `yarn lint`
  - バックエンド: `cargo fmt` + `cargo clippy -D warnings`

## ☁️ インフラ・デプロイ
- **フロントエンド**: Dockerイメージ（nginx配信）をVPS/専用サーバーで実行
- **バックエンド**: Dockerコンテナ（`mediavault-api`）をVPS/専用サーバーで実行
- **データベース**: 同一VPS上のPostgreSQLコンテナ（Docker Compose）

## 🔒 セキュリティ
- **認証**: なし（単一ユーザー運用前提、ログイン画面を持たない仕様）
- **HTTPS**: セルフホスト環境に応じてリバースプロキシ等で設定（要検討事項）
- **バリデーション**: サーバーサイド（Actix-web側）でのバリデーション
- **環境変数**: `.env` / `.env.example` で管理
- **依存関係**: `cargo clippy`・`yarn lint` による静的チェックをCIで実施

## 📊 品質基準
- **テストカバレッジ**: 主要フロー（CRUD・一覧・検索追加）をVitest/Playwright/cargo testでカバー
- **コード品質**: `yarn lint` / `cargo clippy -D warnings`
- **型安全性**: TypeScript（フロントエンド）、Rust型システム（バックエンド）
- **UI準拠**: [docs/frontend/ui](../frontend/ui) モックに厳密準拠（[docs/frontend/PRD.md](../frontend/PRD.md) より）

## 📁 ディレクトリ構造（実態）

```
./ (プロジェクトルート)
├── frontend/                 # React + TypeScript (Vite)
│   ├── src/
│   ├── CLAUDE.md
│   └── Dockerfile
├── backend/                  # Rust (Actix-web, mediavault-api)
│   ├── src/
│   ├── CLAUDE.md
│   └── docker-compose.yml
├── docs/
│   ├── PRD.md                # ルートPRD
│   ├── tech-stack.md         # 本ファイル
│   ├── frontend/PRD.md
│   ├── frontend/ui/          # UIモック
│   └── backend/
├── .github/
│   └── workflows/
│       └── backend-ci.yml
└── README.md
```

## 🚀 セットアップ手順

### 1. フロントエンド
```bash
cd frontend
yarn install
yarn dev
```

### 2. バックエンド
```bash
cd backend
cp .env.example .env
docker compose up -d db
cargo build -p mediavault-api
```

### 3. 主要コマンド
```bash
# フロントエンド
yarn test        # ユニットテスト
yarn test:e2e     # E2Eテスト
yarn lint         # Lint
yarn build        # ビルド（型チェック含む）

# バックエンド
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

## 📝 カスタマイズ方法

このファイルはプロジェクトの進行に応じて更新してください：

1. **技術の追加**: 新しいライブラリ・ツールを追加した場合
2. **要件の変更**: パフォーマンス・セキュリティ要件が変化した場合
3. **インフラの変更**: デプロイ先・スケール要件が変わった場合

## 🔄 更新履歴
- 2026-07-04: 初回生成（init-tech-stackにより、既存CLAUDE.md/PRD.mdの内容を集約して自動生成）
