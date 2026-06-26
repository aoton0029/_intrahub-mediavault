# TASK-0024 TDD開発コンテキストノート: GET /items/search 実装

**タスク**: TASK-0024 - GET /items/search エンドポイント実装
**プロジェクト**: mediavault-backend
**作成日**: 2026-06-26
**関連要件**: [docs/spec/mediavault-backend/TASK-0024-requirements.md](../../../spec/mediavault-backend/TASK-0024-requirements.md), [docs/spec/mediavault-backend/TASK-0024-testcases.md](../../../spec/mediavault-backend/TASK-0024-testcases.md)

---

## 1. 技術スタック

### 言語・フレームワーク
- **言語**: Rust (edition 2024)
- **Webフレームワーク**: Axum 0.8.9
- **非同期ランタイム**: Tokio 1.52.3
- **データベース**: PostgreSQL (sqlx 0.8)
- **JSON処理**: serde_json 1.0.150
- **参照元**: backend/mediavault-api/Cargo.toml

### アーキテクチャパターン
- **レイヤード構造**: routes → handlers → services → repositories → db
- **特徴**: 単一ユーザー・小規模運用前提。CQRSやマイクロサービス不採用
- **参照元**: docs/design/mediavault-backend/architecture.md L20-28

### 外部API連携
- **既存ライブラリ**: api-client-lib ワークスペース内プロジェクト（Jikan/TMDb/NDL/OpenLibrary/Steam/IGDB/AniList クライアント実装済み）
- **インターフェース**: `ApiClient` トレイト（execute メソッド）
- **ディスパッチ方式**: ExternalSearchService（media_type → provider 振り分け）による静的ディスパッチ
- **参照元**: backend/mediavault-api/src/services/external_search.rs

---

## 2. 開発ルール

### コーディング規約
- **ハンドラ設計**: Axum extractors を活用（Query, Json, State, Path等）
- **エラーハンドリング**: `Result<T, ApiError>` 統一。`?` 演算子で自動伝播・panic防止
- **DTO定義**: serde::Deserialize + Serialize で自動導出
- **テスト配置**: ハンドラ・モデルファイル内に `#[cfg(test)] mod tests` でインライン配置（ルート＆統合テストは routes/mod.rs）

### エラー処理方針
- **APIレスポンス形式統一**: 成功=`ApiOk<T>`、失敗=`ApiError` をそのまま `IntoResponse`
- **エラーコード文字列**: ワイヤーコードは snake_case（`API_KEY_NOT_CONFIGURED`等）
- **情報漏洩防止**: DB内部情報・外部API生エラー詳細をクライアントへ返さない
- **参照元**: docs/spec/mediavault-backend/TASK-0024-requirements.md 第4章・NFR-0024-02

### ルーティング方針
- **登録順序**: リテラルパス（`/items/search`）を動的パス（`/items/:id`）より前に登録して誤マッチ防止
- **配置**: 全エンドポイントが `routes/mod.rs` 内 `build_router` 関数に `.route(...)` でフラット列挙
- **参照元**: docs/spec/mediavault-backend/TASK-0024-requirements.md 第5章

### テスト規約
- **ユニットテスト**: DB非依存の変換・デシリアライズ検証（models/response.rs, models/item_search.rs）
- **ルーター統合**: `build_router(state)` + `tower::ServiceExt::oneshot` で駆動（既存 routes/mod.rs テストパターン踏襲）
- **E2E統合**: `#[ignore]` 付与で `cargo test -- --ignored` 実行（実DB+外部APIモック前提）
- **参照元**: docs/spec/mediavault-backend/TASK-0024-testcases.md 第0章・第5章

---

## 3. 関連実装

### 前提TASK-0023（ExternalSearchService）
- **実装ファイル**: backend/mediavault-api/src/services/external_search.rs
- **DTO型**:
  - `ExternalSearchResult`: media_type, provider (Option), external_id, title, raw_data
  - `ExternalSearchError`: ApiKeyNotConfigured(ApiProvider) | ExternalApiError(api_client_lib::ApiError)
- **主要メソッド**: `ExternalSearchService::new(pool: PgPool) -> Self`
  - `async fn search(media_type: MediaType, query: &str) -> Result<Vec<ExternalSearchResult>, ExternalSearchError>`
- **特性**: PgPool を直接構築・所有のためモック化困難

### 既存ハンドラパターン
- **ファイル**: backend/mediavault-api/src/handlers/items.rs
- **実装例**: `create_item_handler`, `list_items_handler` 等
- **パターン**:
  - `State(state): State<AppState>` で依存注入
  - `Query<Dto>` で クエリパラメータ抽出
  - 成功→`ApiOk::new(data)` でレスポンス構築
  - 失敗→`Err(ApiError::...)` で伝播
- **参照元**: backend/mediavault-api/src/handlers/items.rs L24-49

### AppState 構造
- **ファイル**: backend/mediavault-api/src/main.rs
- **定義**: `AppState { db: PgPool, internal_api_key: String }`
- **特徴**: ExternalSearchService を保持しない（本タスクで都度構築）
- **参照元**: docs/spec/mediavault-backend/TASK-0024-requirements.md 第3章 L79-86

### エラーマッピング既存パターン
- **ファイル**: backend/mediavault-api/src/models/response.rs
- **型**: ApiErrorCode enum (ValidationError, ItemNotFound等)
- **実装**: `IntoResponse` で ステータスコード + ワイヤーコード JSON を構築
- **課題**: 新規 variant (ApiKeyNotConfigured/ExternalApiTimeout) 追加が必要（既存variant はコード文字列が要件不一致）
- **参照元**: docs/spec/mediavault-backend/TASK-0024-requirements.md 第4章 L105-115

### MediaType enum
- **ファイル**: backend/mediavault-api/src/models/item.rs L15-24
- **Variant**: anime, movie, drama, manga, novel, game, academic_book, paper（8個）
- **デシリアライズ**: snake_case 文字列として既に Deserialize 実装済み
- **参照元**: docs/spec/mediavault-backend/TASK-0024-requirements.md 第2章 L52

---

## 4. 設計文書

### API 仕様（外部API検索・インポート）
- **ファイル**: docs/design/mediavault-backend/api-endpoints.md
- **対象**: GET /items/search エンドポイント定義
- **クエリ契約**: media_type（必須）、q（必須・検索語）
- **成功レスポンス**: 200、ApiOk<Vec<ExternalSearchResult>>
- **エラーコード**: 
  - 400 VALIDATION_ERROR（パラメータ欠落・不正値）
  - 422 API_KEY_NOT_CONFIGURED（APIキー未設定）
  - 502 EXTERNAL_API_TIMEOUT（外部API障害）

### アーキテクチャ（レイヤード構造）
- **ファイル**: docs/design/mediavault-backend/architecture.md
- **関連セクション**: コンポーネント構成 L30-46（APIサーバー・外部APIクライアント）
- **設計判断**: routes → handlers → services → repositories の単方向依存

### データフロー（機能1: 外部API検索）
- **ファイル**: docs/design/mediavault-backend/dataflow.md
- **フロー**: ユーザー/クライアント → GET /items/search → ハンドラ → ExternalSearchService → 外部API → ExternalSearchResult 配列返却
- **エラーフロー**: ExternalSearchError → ApiError マッピング → HTTP ステータス

### 要件定義書
- **ファイル**: docs/spec/mediavault-backend/TASK-0024-requirements.md
- **対象**: TASK-0024 に固有の機能要件・非機能要件・エッジケース定義
- **要件記号**:
  - 🔵 確実な要件（設計文書・受け入れ基準準拠）
  - 🟡 妥当な推測
  - 🔴 推測による要件（本タスクは無し）

### テストケース定義書
- **ファイル**: docs/spec/mediavault-backend/TASK-0024-testcases.md
- **テストケース数**: 13個（正常系2 + 異常系8 + 境界値3）
- **テスト分類**:
  - ユニット（DB非依存）: From変換・デシリアライズ・マッピング検証
  - ルーター統合（DB非依存）: extractor 検証・誤マッチ防止
  - E2E統合（`#[ignore]` 付与）: 実DB+外部APIモック+200成功経路

---

## 5. テスト関連情報

### テストフレームワーク
- **テスト実行**: `cargo test -p mediavault-api`（ユニット・ルーター）、`cargo test -- --ignored`（統合）
- **非同期対応**: `#[tokio::test]` マクロで async テスト関数対応
- **ルーター駆動**: `tower::ServiceExt::oneshot(request)` で単一リクエスト駆動（既存パターン）
- **外部APIモック**: `wiremock` 0.6（HTTP mock server）で既知応答返却

### 既存テストパターン（routes/mod.rs）
- **ファイル**: backend/mediavault-api/src/routes/mod.rs（`#[cfg(test)] mod tests` セクション）
- **初期化**: `test_app_state()` で実 DB コネクション取得（`.env` + DATABASE_URL 前提）
- **ルーター構築**: `build_router(state)` で全ルート登録済みルーターを生成
- **不正値テスト例**: GET /items?media_type=invalid でステータス 400 確認（L156-179）
- **特徴**: `#[ignore]` + 実DB 前提のため `cargo test -- --ignored` で別実行

### 統合テスト実行環境前提
- **Database setup**: `docker compose up -d db`（postgres コンテナ起動）
- **環境変数**: `.env` に `DATABASE_URL=postgresql://...` 設定
- **外部API**: wiremock + ベースURL注入による HTTP モック（実外部API非接続）
- **参照元**: backend/CLAUDE.md

### テストコメント規約（日本語指針）
- **テスト目的**: 「【テスト目的】: 〜を確認する」で要件対応を明示
- **テスト内容**: 「【テスト内容】: 〜を実行する」で具体処理を記述
- **期待される動作**: 「【期待される動作】: 〜であること」で期待値を記述
- **信頼性レベル**: 「🔵/🟡 信頼性レベル: 〜より」で根拠を明示
- **参照元**: docs/spec/mediavault-backend/TASK-0024-testcases.md 第5章 L286-316

---

## 6. 注意事項

### 実装上の制約
1. **AppState に ExternalSearchService 保持しない**: ハンドラ内で `ExternalSearchService::new(state.db.clone())` を都度構築する
   - 理由: AppState が PgPool のみ保持する設計・既存規約踏襲
   - 参照元: docs/spec/mediavault-backend/TASK-0024-requirements.md 第3章 L79-86

2. **errors.rs ファイル不在**: タスクファイルが指すファイルが実在しないため、`From<ExternalSearchError> for ApiError` は models/response.rs または handlers/items.rs に実装する必要がある
   - 参照元: docs/spec/mediavault-backend/TASK-0024-requirements.md 第4章 L115

3. **routes/items.rs ファイル不在**: タスクファイルが指すファイルが実在しない。全ルート登録は routes/mod.rs build_router 内で行う
   - 参照元: docs/spec/mediavault-backend/TASK-0024-requirements.md 第5章 L119

4. **Query Rejection 整形要否**: 不正クエリで素の Axum 400 を返すか、統一 ApiError（VALIDATION_ERROR ボディ）を返すか、tdd-red 前確定が必要
   - 既存パターン: routes/mod.rs L156-179 の不正値テストはボディ形式検証なし
   - 参照元: docs/spec/mediavault-backend/TASK-0024-requirements.md 第2章 注記・第9章 引き渡し 1

### 要件上の重要ポイント
- **外部APIエラーの集約**: api-client-lib::ApiError の 6 variant（Http/Auth/RateLimit/Parse/Timeout/Network）すべてが 502 EXTERNAL_API_TIMEOUT へ集約される（panic 非発生を確保）
- **APIキー必須判定**: TASK-0022 で実装の `find_by_provider` 呼び出しで省略可能（Jikan はキー不要）。TMDb 等キー必須プロバイダのみ 422 を返す
- **ルート登録順序**: Axum 0.8 ではリテラル（`/items/search`）が動的（`/items/:id`）より優先マッチするが、タスク注意事項・可読性・将来バージョン変更への安全策として前方登録を強制
- **q パラメータの空文字**: TASK-0023 と同じ透過方針で、ハンドラで 400 化せず `search(_, "")` を呼ぶ

### セキュリティ・パフォーマンス
- **情報漏洩防止**: エラーレスポンスに DB 内部情報・外部API 生エラー詳細を含めない（既存 ApiError 汎用メッセージ方針踏襲）
- **外部API 障害耐性**: `?` 伝播で panic しない。502 マッピングで顧客に影響を最小化
- **コネクションプール**: 単一ユーザー・セルフホスト想定で小規模コネクションプール（既存設定踏襲）

---

## 7. ファイル一覧（相対パス）

### 実装対象
- `backend/mediavault-api/src/models/item_search.rs`（新規）- ItemSearchQuery DTO
- `backend/mediavault-api/src/handlers/items.rs`（追記）- search_items ハンドラ + テスト
- `backend/mediavault-api/src/models/response.rs`（追記）- ApiErrorCode 新variant + From 実装
- `backend/mediavault-api/src/routes/mod.rs`（追記）- /items/search ルート登録

### 参照文書
- `docs/spec/mediavault-backend/TASK-0024-requirements.md` - 要件定義書
- `docs/spec/mediavault-backend/TASK-0024-testcases.md` - テストケース定義書
- `docs/design/mediavault-backend/api-endpoints.md` - API 仕様
- `docs/design/mediavault-backend/architecture.md` - アーキテクチャ
- `docs/design/mediavault-backend/dataflow.md` - データフロー
- `docs/spec/mediavault-backend/requirements.md` - 統合要件定義（REQ-002）
- `backend/mediavault-api/Cargo.toml` - 依存関係

### 既存参照コード
- `backend/mediavault-api/src/services/external_search.rs` - ExternalSearchService（前提TASK-0023）
- `backend/mediavault-api/src/models/external_search.rs` - ExternalSearchResult/Error DTO
- `backend/mediavault-api/src/models/item.rs` - MediaType enum（既存）
- `backend/mediavault-api/src/handlers/items.rs` - 既存ハンドラパターン（create_item_handler等）
- `backend/mediavault-api/src/routes/mod.rs` - ルーター実装＆テストパターン
- `backend/mediavault-api/src/main.rs` - AppState 定義

---

## 8. 次ステップ

### tdd-red 着手前確定事項
1. **Query Rejection 整形**: 統一 `ApiError`（`VALIDATION_ERROR`ボディ）を返すか決定
2. **From 実装配置**: `handlers/items.rs` か `models/response.rs` か決定
3. **ApiErrorCode variant 名**: `ApiKeyNotConfigured` / `ExternalApiTimeout` 確定
4. **ExternalSearchService テスト**: 実 PgPool 必須のため統合 `#[ignore]` 方針確定
5. **q 空文字方針**: ハンドラは透過・サービス層で判定（TASK-0023 踏襲）
6. **wiremock 追加**: `Cargo.toml` の `[dev-dependencies]` へ追加（または確認）

### 実装順序推奨
1. ItemSearchQuery DTO 定義（models/item_search.rs）
2. ApiErrorCode 新variant 追加（models/response.rs）
3. From<ExternalSearchError> for ApiError 実装
4. search_items ハンドラ実装（handlers/items.rs）
5. ルート登録（routes/mod.rs）
6. テスト実装（各ファイルのインライン + routes テスト）

