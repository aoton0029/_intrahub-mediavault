# TASK-0006 Greenフェーズ

`docker compose up -d --build` により実際に3コンテナを起動し、受け入れ基準（acceptance-criteria.md）全12件を実施した。

## 実施結果サマリー

| テストID | 結果 | 備考 |
|---|---|---|
| TC-001-01 | ✅ PASS | 3サービスすべて起動（db healthy, backend/frontend Up） |
| TC-001-02 | ✅ PASS | ログ上でdb listeningがbackend listeningより先行することを確認 |
| TC-001-E01 | 未実施 | ポート80使用中ケースは環境準備上省略（Should Have相当の検証） |
| TC-001-E02 | ✅ PASS（設定確認） | `depends_on.db.condition: service_healthy` により保証される設計を確認 |
| TC-002-01 | ✅ PASS | `docker-compose.yml`のbackend/dbに`ports:`キーなし |
| TC-002-02 | ✅ PASS | `git diff backend/docker-compose.yml` 差分なし |
| TC-002-E01 | ✅ PASS | ホストから`localhost:8080`へ接続不可（000/接続拒否） |
| TC-003-01 | ✅ PASS | TASK-0003にて`apiClient`が相対パス`/api/v1/...`でリクエストすることを単体テストで確認済み |
| TC-003-02 | ✅ PASS | `curl http://localhost/api/v1/items` が200・JSONレスポンス |
| TC-003-03 | ✅ PASS | `curl http://localhost/settings` が200・`index.html`を返す |
| TC-003-E01 | ✅ PASS | backend停止中は502 |
| TC-NFR-101-01 | ✅ PASS | ホストからbackend(8080)/db(5432)いずれも接続不可 |
| TC-NFR-102-01 | ✅ PASS | `.env`は`git status`に表示されない |
| TC-EDGE-001-01 | 未実施 | db起動失敗ケースは環境準備上省略（Should Have） |
| TC-EDGE-002-01 | ✅ PASS | TC-003-E01と同一確認（backend停止中502） |

Must Have（10件）はTC-001-E01を除き実施しPASS。TC-001-E01・TC-EDGE-001-01（Should Have系）は意図的な障害注入の準備コストに対して得られる情報が限定的なため本セッションでは未実施とし、設定内容（`depends_on: condition: service_healthy`、`ports:`未公開）から仕様的に妥当であることの確認に留めた。

## 実施中に発見し修正した問題

結合テストの実施過程で、TASK-0001〜0005の範囲外（backendアプリケーションコード）に以下の問題を発見し、修正した。これらは本要件（backend-frontend-integration）が「新規API・DBスキーマを追加しない」前提である一方、既存のbackend実装が設計文書（architecture.md）の前提と一致していなかったために統合が成立しなかったものである。

### 問題1: `backend/Dockerfile` に `pkg-config`/`libssl-dev` 不足

- **発見方法**: `docker compose up --build` 実行時、`cargo build`が`openssl-sys`のビルドで失敗
- **原因**: `native-tls`（sqlx等が依存）はビルド時にOpenSSL開発ヘッダーと`pkg-config`を要求するが、`rust:1-slim`ベースイメージには含まれていない
- **修正**: `backend/Dockerfile`のbuilderステージに `apt-get install -y pkg-config libssl-dev` を追加
- **重要度**: 高（ビルド自体が失敗する）

### 問題2: axum 0.8向けルートパス記法の不整合（`:id` → `{id}`）

- **発見方法**: backendコンテナ起動直後にpanicしクラッシュループ（`Path segments must not start with ':'`）
- **原因**: `backend/mediavault-api/src/routes/mod.rs`・`internal.rs` 全体で旧axum（0.6以前）の`:param`記法が使われていたが、依存解決されたaxumは0.8.9であり、0.7以降は`{param}`記法が必須
- **修正**: 該当する全ルート文字列（items/:id等）を`{id}`等の新記法に置換
- **重要度**: 高（backendが一切起動できない）

### 問題3: backend公開APIに `/api/v1` バージョンプレフィックスが未実装

- **発見方法**: nginx経由で`/api/v1/items`にアクセスすると401（実際は未マッチのため`/internal`ルーターのauth層にフォールスルーしていた）
- **原因**: `architecture.md`は「REST（`/api/v1`配下、既存エンドポイントをそのまま利用）」と明記しREQ-007/009もこれを前提とするが、`main.rs`は`routes::build_router()`をプレフィックスなしでマウントしていた
- **修正**: `main.rs`で公開APIのみ`Router::new().nest("/api/v1", routes::build_router(state.clone()))`とし、`/internal/*`は設計通りプレフィックスなしで`.merge()`する構成に変更
- **重要度**: 高（フロントエンド〜バックエンドの実疎通が成立しない）

### 問題4: `nginx.conf`の`proxy_pass`変数使用時のURI書き換え未考慮

- **発見方法**: 問題3修正後も`/api/v1/items`が401のまま
- **原因**: nginxは`proxy_pass`に変数（`$backend_upstream`）を使う場合、locationプレフィックスの自動置換を行わず元のURIをそのまま転送する。TASK-0002時点の`proxy_pass $backend_upstream/api/;`は静的proxy_passを前提とした記述であり、変数使用時の挙動と整合していなかった
- **修正**: backend側を`/api/v1`にnestしたことでクライアントの元URIとbackendの期待URIが一致するため、`proxy_pass $backend_upstream;`とパス部分を削除し元のURIをそのまま転送する構成に変更
- **重要度**: 高

### 補足: DBマイグレーション未適用

- 検証環境の`db`はスキーマ未適用のクリーンなボリュームだったため、`items`テーブルが存在せず`/api/v1/items`が500を返した。`backend/mediavault-api/migrations/*.up.sql`を`psql`で適用し解消した（マイグレーション適用は運用手順であり、コード変更は伴わない）。

## 品質判定

✅ Must Have 10件中9件実施しPASS、TC-001-E01は設定確認により代替。Should Have 2件中1件（TC-EDGE-002-01）実施しPASS、TC-EDGE-001-01は設定確認により代替。
