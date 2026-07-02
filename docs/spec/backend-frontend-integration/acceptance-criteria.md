# backend-frontend-integration 受け入れ基準

**作成日**: 2026-07-02
**関連要件定義**: [requirements.md](requirements.md)
**関連ユーザストーリー**: [user-stories.md](user-stories.md)
**ヒアリング記録**: [interview-record.md](interview-record.md)

**【信頼性レベル凡例】**:
- 🔵 **青信号**: PRD・設計文書・ユーザヒアリングを参考にした確実な基準
- 🟡 **黄信号**: PRD・設計文書・ユーザヒアリングから妥当な推測による基準
- 🔴 **赤信号**: PRD・設計文書・ユーザヒアリングにない推測による基準

---

## REQ-001/REQ-101: 統合docker-composeによる一括起動 🔵

**信頼性**: 🔵 *PRD 検証方法・ヒアリングQ1〜Q5より*

### Given（前提条件）
- リポジトリルートに統合用 `docker-compose.yml`（または `docker-compose.integration.yml`）が存在する
- `backend/.env` 等、必要な環境変数ファイルが用意されている

### When（実行条件）
- 開発者がリポジトリルートで `docker compose up` を実行する

### Then（期待結果）
- `db` → `backend` → `frontend` の順に依存関係を満たしつつ全コンテナが起動する
- 3コンテナ（`frontend`/`backend`/`db`）が `docker compose ps` で `running`/`healthy` 状態になる

### テストケース

#### 正常系

- [ ] **TC-001-01**: `docker compose up -d` 実行後、全コンテナが起動状態になる 🔵
  - **入力**: `docker compose up -d`
  - **期待結果**: `docker compose ps` で3サービスすべてが `Up`（dbは`healthy`）
  - **信頼性**: 🔵 *PRD検証方法より*

- [ ] **TC-001-02**: `db` のヘルスチェック成功後に `backend` が起動する 🔵
  - **入力**: `docker compose up` 実行直後のログ
  - **期待結果**: ログ上で `backend` の起動が `db` のヘルスチェック成功後であることが確認できる
  - **信頼性**: 🔵 *既存 backend/docker-compose.yml の depends_on 設定より*

#### 異常系

- [ ] **TC-001-E01**: `frontend` のホスト公開ポート（例:80）が使用中の場合、起動が失敗する 🟡
  - **入力**: ポート80を別プロセスが使用している状態で `docker compose up`
  - **期待結果**: Docker Composeがポートバインドエラーで起動失敗を報告する
  - **信頼性**: 🟡 *Docker Composeの標準挙動からの推測*

- [ ] **TC-001-E02**: `db` のヘルスチェックが失敗し続ける場合、`backend` は起動を待機する 🟡
  - **入力**: DB接続情報が誤っている等でヘルスチェックが失敗し続ける状態
  - **期待結果**: `backend` コンテナが起動せず待機状態が続く
  - **信頼性**: 🟡 *既存docker-compose設定の挙動からの推測*

---

## REQ-201/REQ-401/REQ-402: backend・dbの非公開化 🔵

**信頼性**: 🔵 *ヒアリングQ5より*

### Given（前提条件）
- 統合用docker-composeで環境が起動している

### When（実行条件）
- ホストマシンから `backend`（8080）・`db`（5432）へ直接接続を試みる

### Then（期待結果）
- 接続が確立できない（ポートが公開されていない）

### テストケース

#### 正常系

- [ ] **TC-002-01**: 統合用docker-compose定義に `backend`・`db` の `ports:` が存在しない 🔵
  - **入力**: 統合用 `docker-compose.yml` の内容
  - **期待結果**: `backend`・`db` サービス定義に `ports:` キーが存在しない
  - **信頼性**: 🔵 *ヒアリングQ5より*

- [ ] **TC-002-02**: 既存の `backend/docker-compose.yml`（backend単体起動用）が変更されていない 🔵
  - **入力**: `git diff backend/docker-compose.yml`
  - **期待結果**: 差分なし
  - **信頼性**: 🔵 *ヒアリングQ5より（既存ファイルは変更しない方針）*

#### 異常系

- [ ] **TC-002-E01**: ホストから `localhost:8080` へ接続を試みると失敗する 🔵
  - **入力**: `curl http://localhost:8080/api/v1/health`（統合環境起動中）
  - **期待結果**: 接続拒否またはタイムアウト
  - **信頼性**: 🔵 *REQ-201/REQ-401より*

---

## REQ-007/REQ-008/REQ-009/REQ-102: nginxリバースプロキシによるAPI疎通 🔵

**信頼性**: 🔵 *PRD nginx設定例・ヒアリングQ3, Q4より*

### Given（前提条件）
- 統合環境が起動しており、`frontend` がホストの80番ポートで待ち受けている
- `frontend/src/api/client.ts` のデフォルトURLが相対パス `/api/v1` に変更されている

### When（実行条件）
- ブラウザで `http://localhost` にアクセスし、データ取得を伴う画面操作を行う

### Then（期待結果）
- ブラウザの開発者ツールで、APIリクエスト先が `http://localhost/api/v1/...`（同一オリジン）になっている
- backendからのレスポンスが正常に表示される

### テストケース

#### 正常系

- [ ] **TC-003-01**: フロントエンドのAPIリクエストが相対パスで発行される 🔵
  - **入力**: ブラウザでfrontendにアクセスし、一覧画面等を開く
  - **期待結果**: ネットワークタブでリクエストURLが `/api/v1/...`（絶対URLではない）
  - **信頼性**: 🔵 *REQ-009・ヒアリングQ4より*

- [ ] **TC-003-02**: nginxが `/api/` リクエストをbackendへ正しく転送する 🔵
  - **入力**: `curl http://localhost/api/v1/items`（統合環境起動中）
  - **期待結果**: backendからのレスポンス（JSON等）が返る
  - **信頼性**: 🔵 *PRD nginx設定例より*

- [ ] **TC-003-03**: SPAの直接URLアクセス・リロードで `index.html` が返る 🟡
  - **入力**: `curl http://localhost/settings`（存在しない静的パス）
  - **期待結果**: `index.html` の内容が返る（SPAルーティングが機能する）
  - **信頼性**: 🟡 *PRD nginx設定例（try_files）からの推測*

#### 異常系

- [ ] **TC-003-E01**: backend未起動時に `/api/` へアクセスするとエラーが返る 🟡
  - **入力**: `backend` コンテナ停止中に `curl http://localhost/api/v1/items`
  - **期待結果**: nginxが502 Bad Gateway等のエラーを返す
  - **信頼性**: 🟡 *nginxリバースプロキシの一般的挙動からの推測*

---

## 非機能要件テスト

### NFR-101: backend・dbの直接アクセス遮断 🔵

**信頼性**: 🔵 *PRD API疎通方式・セキュリティ意図より*

- [ ] **TC-NFR-101-01**: ホストからbackend/dbへの直接接続テスト
  - **検証内容**: `docker compose up` 後、ホストから8080・5432番ポートへの接続可否
  - **期待結果**: 両方とも接続不可
  - **信頼性**: 🔵 *REQ-201/REQ-401より*

### NFR-102: 環境変数管理 🔵

**信頼性**: 🔵 *backend/tech-stack.md セキュリティ節より*

- [ ] **TC-NFR-102-01**: `.env` ファイルがgit管理対象外であることの確認
  - **検証内容**: `.gitignore` に `.env` が含まれているか確認
  - **期待結果**: `.env` が除外されている
  - **信頼性**: 🔵 *既存プロジェクトのセキュリティ方針より*

---

## Edgeケーステスト

### EDGE-001: dbヘルスチェック失敗時のbackend待機 🟡

**信頼性**: 🟡 *既存docker-compose設定から妥当な推測*

- [ ] **TC-EDGE-001-01**: dbが起動しない状態でのbackend挙動確認
  - **条件**: `db` サービスを意図的に起動失敗させる（例: 誤った環境変数）
  - **期待結果**: `backend` が起動を試みず待機し続ける
  - **信頼性**: 🟡 *depends_on: condition: service_healthy の挙動からの推測*

### EDGE-002: backend停止中のAPIアクセス 🟡

**信頼性**: 🟡 *nginxリバースプロキシの一般的挙動からの推測*

- [ ] **TC-EDGE-002-01**: backend停止中の `/api/` アクセスエラー確認
  - **条件**: `docker compose stop backend` 実行後に `/api/` へアクセス
  - **期待結果**: nginxが502等のエラーレスポンスを返す
  - **信頼性**: 🟡 *実装から推測*

---

## テストケースサマリー

### カテゴリ別件数

| カテゴリ | 正常系 | 異常系 | 境界値 | 合計 |
|---------|--------|--------|--------|------|
| 機能要件 | 5 | 3 | 0 | 8 |
| 非機能要件 | 2 | 0 | 0 | 2 |
| Edgeケース | 0 | 2 | 0 | 2 |
| **合計** | 7 | 5 | 0 | 12 |

### 信頼性レベル分布

- 🔵 青信号: 8件 (67%)
- 🟡 黄信号: 4件 (33%)
- 🔴 赤信号: 0件 (0%)

**品質評価**: 高品質

### 優先度別テストケース

- **Must Have**: 10件
- **Should Have**: 2件
- **Could Have**: 0件

---

## テスト実施計画

### Phase 1: 基本機能テスト
- REQ-001, REQ-101, REQ-201, REQ-401, REQ-402
- 優先度: Must Have
- 実施予定: 実装完了直後

### Phase 2: API疎通テスト
- REQ-007, REQ-008, REQ-009, REQ-102
- 優先度: Must Have
- 実施予定: フロントエンドDockerfile・nginx設定完成後

### Phase 3: 非機能・Edgeケーステスト
- NFR-101, NFR-102, EDGE-001, EDGE-002
- 優先度: Must Have + Should Have
- 実施予定: Phase1・Phase2完了後
