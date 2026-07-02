# backend-frontend-integration 設計ヒアリング記録

**作成日**: 2026-07-02
**ヒアリング実施**: step4 既存情報ベースの差分ヒアリング

## ヒアリング目的

既存の要件定義書・受け入れ基準・コンテキストノート（`docs/spec/backend-frontend-integration/`）と、既存実装（`backend/Dockerfile`・`backend/docker-compose.yml`・`backend/.env.example`・`frontend/src/api/client.ts`・`frontend/vite.config.ts`）を確認したうえで、設計文書化にあたり未確定だった2点（REQ-301のファイル名選択、REQ-202のポート番号最終決定）を明確化するためヒアリングを実施しました。

なお、要件定義段階（`interview-record.md`）で既にアーキテクチャ・API疎通方式・ネットワーク分離方針・スコープ外事項の大部分は確定済みであったため、本ヒアリングは要件定義書の「残課題」欄に記載された2点に絞って実施しました。

## 質問と回答

### Q1: 統合用docker-composeファイル名（REQ-301の選択確定）

**質問日時**: 2026-07-02
**カテゴリ**: 技術選択
**背景**: 要件定義書REQ-301で `docker-compose.yml` または `docker-compose.integration.yml` のいずれかを許容する旨が🟡（黄信号）で記載されており、設計文書化にあたりファイルパスを一意に確定する必要があったため

**回答**: `docker-compose.yml`（ルート直下に配置）

**信頼性への影響**:
- REQ-301に対応する設計項目（ディレクトリ構造・ファイルパス記載）の信頼性レベルが 🟡 → 🔵 に向上
- アーキテクチャ設計書のディレクトリ構造セクションのファイルパスが一意に確定

---

### Q2: frontendのホスト公開ポート番号（REQ-202の最終決定）

**質問日時**: 2026-07-02
**カテゴリ**: 技術選択
**背景**: PRDでは80番ポートが「暫定案」として記載されており、要件定義書の残課題欄にも「最終決定」が必要と明記されていたため

**回答**: 80番（PRDの暫定案どおり）

**信頼性への影響**:
- REQ-202に対応する設計項目（システム構成図・データフロー図のポート表記）の信頼性レベルが 🟡 → 🔵 に向上
- データフロー図・アーキテクチャ図のポート番号表記が確定

---

## ヒアリング結果サマリー

### 確認できた事項
- 統合用docker-composeはルート直下の `docker-compose.yml` として配置する（`docker-compose.integration.yml` は不採用）
- `frontend` のホスト公開ポートは80番で確定する
- それ以外のアーキテクチャ方針（nginx一本化、backend/db非公開、API相対パス化、CI・selfhosted・Vite devサーバーのスコープ外化）は要件定義段階（`interview-record.md`）で既に確定済みであり、本設計ヒアリングでは変更なし

### 設計方針の決定事項
- ディレクトリ構造: `./docker-compose.yml`（新規）、`frontend/Dockerfile`（新規）、`frontend/nginx.conf`（新規）
- ポート構成: `frontend` = 80:80（公開）、`backend` = 8080（内部のみ）、`db` = 5432（内部のみ）
- 本設計では新規API・DBスキーマを追加しないため、`interfaces.ts`・`database-schema.sql`・`api-endpoints.md` は生成しない

### 残課題
- なし（要件定義書記載の残課題2件はいずれも本ヒアリングで確定）

### 信頼性レベル分布

**ヒアリング前**（要件定義書・受け入れ基準時点、設計文書化直前）:
- 🔵 青信号: 22
- 🟡 黄信号: 8
- 🔴 赤信号: 0

**ヒアリング後**（architecture.md・dataflow.md 作成後の集計）:
- 🔵 青信号: 24 (+2)
- 🟡 黄信号: 7 (-1)
- 🔴 赤信号: 0 (±0)

*内訳の差異はNFR-001（パフォーマンス数値未記載）・スケーラビリティ・可用性・エラーハンドリング等、要件定義段階から引き続き🟡（黄信号）のまま残る項目が存在するため*

## 関連文書

- **アーキテクチャ設計**: [architecture.md](architecture.md)
- **データフロー**: [dataflow.md](dataflow.md)
- **要件定義**: [requirements.md](../../spec/backend-frontend-integration/requirements.md)
- **要件定義段階ヒアリング記録**: [interview-record.md](../../spec/backend-frontend-integration/interview-record.md)
