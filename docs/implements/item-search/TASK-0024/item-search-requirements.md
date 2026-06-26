# TASK-0024 TDD要件定義書: GET /items/search 外部API検索エンドポイント

**機能名**: item-search（外部API検索エンドポイント）
**タスクID**: TASK-0024
**要件名**: item-search
**タスクタイプ**: TDD
**フェーズ**: Phase 3 - 外部API連携
**作成日**: 2026-06-26
**出力ファイル**: `docs/implements/item-search/TASK-0024/item-search-requirements.md`

---

## 信頼性レベルの凡例

- 🔵 **青信号**: EARS要件定義書・設計文書を参照し、ほぼ推測していない
- 🟡 **黄信号**: EARS要件定義書・設計文書からの妥当な推測
- 🔴 **赤信号**: 元資料にない推測

---

## 1. 機能の概要（EARS要件定義書・設計文書ベース）

- 🔵 **何をする機能か**: クエリパラメータ `media_type`（必須）・`q`（必須・検索語）を受け取り、TASK-0023で実装済みの `ExternalSearchService::search(media_type, &q)` を呼び出して、外部API（Jikan / TMDb 等）の検索結果を統一形式 `ExternalSearchResult` の配列として返す `GET /items/search` エンドポイント。
- 🔵 **解決する問題**: ユーザーが手動でアイテム情報を入力する負担を減らし、外部メタデータプロバイダから候補を取得してインポート（後続TASK-0025）の起点とする。
- 🔵 **想定ユーザー**: 単一ユーザー・セルフホスト運用のMediaVault利用者（クライアントアプリ経由）。
- 🔵 **システム内での位置づけ**: レイヤードアーキテクチャ（routes → handlers → services → repositories）の handlers 層に位置し、service 層（ExternalSearchService）と外部APIクライアント（api-client-lib）への橋渡しを担う。AppState は `PgPool` のみ保持するため、ハンドラ内で `ExternalSearchService::new(state.db.clone())` を都度構築する。
- **参照したEARS要件**: REQ-002（外部API検索）、TC-002-01 / TC-002-02 / TC-002-E01 / TC-002-E02、EDGE-001
- **参照した設計文書**: `docs/design/mediavault-backend/architecture.md`（レイヤード構造 L20-28, コンポーネント構成 L30-46）、`docs/design/mediavault-backend/api-endpoints.md`（外部API検索・インポート節）、`docs/design/mediavault-backend/dataflow.md`（機能1: 外部API検索）

---

## 2. 入力・出力の仕様（EARS機能要件・型定義ベース）

### 2.1 入力（クエリパラメータ）🔵

| パラメータ | 型 | 必須 | 制約 | 出典 |
|---|---|---|---|---|
| `media_type` | `MediaType`（enum） | 必須 | snake_case 文字列。許容値: anime, movie, drama, manga, novel, game, academic_book, paper（8種） | api-endpoints.md / models/item.rs L15-24 |
| `q` | `String` | 必須 | 検索語。空文字はハンドラで400化せず透過し `search(_, "")` を呼ぶ（TASK-0023踏襲） | api-endpoints.md / note.md L193 |

- 🔵 **DTO定義**: `backend/mediavault-api/src/models/item_search.rs`（新規）に以下を定義する。

```rust
#[derive(Debug, Deserialize)]
pub struct ItemSearchQuery {
    pub media_type: MediaType,
    pub q: String,
}
```

- 🔵 Axum の `Query<ItemSearchQuery>` extractor がデシリアライズに失敗した場合（必須欠落・`media_type` 不正値）は `400 VALIDATION_ERROR` を返す。

### 2.2 出力 🔵

- **成功（200 OK）**: `ApiOk<Vec<ExternalSearchResult>>`
  - `ExternalSearchResult` フィールド: `media_type`, `provider`（`Option<ApiProvider>`、Jikanは `null`）, `external_id`, `title`, `raw_data`（生JSON）
  - JSON形式: `{"success": true, "data": [ { ... } ]}`
- **エラー**: `ApiError`（`{"success": false, "error": {"code": ..., "message": ...}}`）

### 2.3 入出力の関係性とデータフロー 🔵

```
クライアント → GET /items/search?media_type=&q=
  → handlers::items::search_items（Query抽出・extractor失敗時400）
  → ExternalSearchService::new(db).search(media_type, &q)
  → 外部APIクライアント（Jikan/TMDb等）
  → Ok(Vec<ExternalSearchResult>) → 200 ApiOk
  → Err(ExternalSearchError) → ApiError マッピング（422 / 502）
```

- **参照したEARS要件**: REQ-002、TC-002-01 / 02
- **参照した設計文書**: api-endpoints.md（GET /items/search 契約）、`backend/mediavault-api/src/models/external_search.rs`（`ExternalSearchResult` / `ExternalSearchError`）、dataflow.md（機能1）

---

## 3. 制約条件（EARS非機能要件・アーキテクチャ設計ベース）

- 🔵 **アーキテクチャ制約**: AppState は `PgPool` のみ保持。`ExternalSearchService` は保持せずハンドラ内で都度構築（`ExternalSearchService::new(state.db.clone())`）。
- 🔵 **ルーティング制約**: `/items/search`（リテラル）を `/items/:id`（動的）より**前**に登録し誤マッチを防止する。登録は `routes/mod.rs` の `build_router` 内にフラット列挙する（`routes/items.rs` は実在しないため）。
- 🔵 **エラー耐性（NFR）**: 外部API障害時もサーバープロセスは panic / クラッシュしない。`?` 演算子で `Result` を伝播し、`ExternalSearchError` を `ApiError` へ変換する。
- 🔵 **エラーコード規約**: ワイヤーコードは大文字 SCREAMING_SNAKE_CASE。本タスクで返すコード:
  - `400 VALIDATION_ERROR`（既存 `ApiErrorCode::ValidationError`）
  - `422 API_KEY_NOT_CONFIGURED`（**新規 variant 必要**）
  - `502 EXTERNAL_API_TIMEOUT`（**新規 variant 必要**）
- 🟡 **既存コードとの差分（重要）**: 既存 `ApiErrorCode::ExternalApiError` は `EXTERNAL_API_ERROR` / 502 にマッピングされており、タスク要求コード `EXTERNAL_API_TIMEOUT` と文字列不一致。既存 enum には `ApiKeyNotConfigured`（422）も無い。よって `models/response.rs` に新 variant 2つ（例: `ApiKeyNotConfigured`→422 / `EXTERNAL_API_TIMEOUT`用variant→502）と `code_and_status` への追記が必要。
- 🟡 **エラー集約**: `api_client_lib::ApiError` の全 variant（Http / Auth / RateLimit / Parse / Timeout / Network）を一律 `502 EXTERNAL_API_TIMEOUT` へ集約する（EDGE-0023-04踏襲）。
- 🔵 **APIキー要否**: Jikan（anime/manga）はキー不要、TMDb 等キー必須プロバイダのみキー未登録時に 422 を返す（サービス層 TASK-0023 が `ApiKeyNotConfigured` を発生）。
- 🔵 **情報漏洩防止（NFR-0024-02）**: エラーレスポンスに DB 内部情報・外部API生エラー詳細を含めない（汎用メッセージ）。
- **参照したEARS要件**: NFR-0024-02、REQ-002（MUST）、EDGE-001 / EDGE-0023-04
- **参照した設計文書**: architecture.md（レイヤード構造）、api-endpoints.md（エラーコード表）、`models/response.rs`（既存 `ApiErrorCode`）、note.md 第6章

---

## 4. 想定される使用例（EARSエッジケース・データフローベース）

### 4.1 基本的な使用パターン 🔵

1. **anime検索（TC-002-01）**: `GET /items/search?media_type=anime&q=鬼滅` → 200、Jikan検索結果一覧。
2. **movie/drama検索（TC-002-02）**: `GET /items/search?media_type=movie&q=タイトル` → 200、TMDb検索結果一覧。

### 4.2 エッジ・エラーケース

- 🟡 **必須パラメータ欠落**: `GET /items/search?media_type=anime`（`q`欠落）→ `400 VALIDATION_ERROR`（Query extractor 拒否）。
- 🔵 **media_type 不正値**: `media_type=invalid` → `400 VALIDATION_ERROR`。
- 🟡 **APIキー未設定（TC-002-E01 / EDGE-001）**: TMDb等でキー未登録 → `ExternalSearchError::ApiKeyNotConfigured` → `422 API_KEY_NOT_CONFIGURED`。
- 🟡 **外部APIタイムアウト・障害（TC-002-E02）**: `ExternalSearchError::ExternalApiError` → `502 EXTERNAL_API_TIMEOUT`、プロセスは panic しない。
- 🟡 **q 空文字**: `q=` → 400化せず透過してサービス層に委譲（TASK-0023踏襲）。

- **参照したEARS要件**: TC-002-E01 / TC-002-E02、EDGE-001
- **参照した設計文書**: dataflow.md（機能1 正常系・エラーフロー）、api-endpoints.md（エラーコード表）

---

## 5. EARS要件・設計文書との対応関係

- **参照したユーザストーリー**: 外部APIからメタデータを検索しインポート候補を得る（REQ-002）
- **参照した機能要件**: REQ-002
- **参照した非機能要件**: NFR-0024-02（情報漏洩防止）、外部API障害耐性（panic非発生）
- **参照したEdgeケース**: EDGE-001（APIキー未設定）、EDGE-0023-04（ApiError全集約）
- **参照した受け入れ基準**: TC-002-01, TC-002-02, TC-002-E01, TC-002-E02、加えて必須欠落400の妥当推測テスト
- **参照した設計文書**:
  - **アーキテクチャ**: `docs/design/mediavault-backend/architecture.md`（L20-46）
  - **データフロー**: `docs/design/mediavault-backend/dataflow.md`（機能1: 外部API検索）
  - **型定義/共通DTO**: `backend/mediavault-api/src/models/external_search.rs`（`ExternalSearchResult` / `ExternalSearchError`）
  - **API仕様**: `docs/design/mediavault-backend/api-endpoints.md`（GET /items/search）
  - **既存エラー型**: `backend/mediavault-api/src/models/response.rs`（`ApiError` / `ApiErrorCode`）

---

## 6. 実装対象ファイル（相対パス）

| ファイル | 区分 | 内容 |
|---|---|---|
| `backend/mediavault-api/src/models/item_search.rs` | 新規 | `ItemSearchQuery` DTO |
| `backend/mediavault-api/src/models/response.rs` | 追記 | `ApiErrorCode` 新variant（422 `API_KEY_NOT_CONFIGURED` / 502 `EXTERNAL_API_TIMEOUT`）＋ `From<ExternalSearchError> for ApiError` |
| `backend/mediavault-api/src/handlers/items.rs` | 追記 | `search_items` ハンドラ＋インラインテスト |
| `backend/mediavault-api/src/routes/mod.rs` | 追記 | `/items/search` を `/items/:id` より前に登録 |

> 注: タスク本文が指す `errors.rs` / `routes/items.rs` は実在しないため、それぞれ `models/response.rs`（または `handlers/items.rs`）/ `routes/mod.rs` を実体とする。

---

## 7. tdd-red 着手前の確定事項（要決定）

1. 🟡 不正クエリ時に素の Axum 400 を返すか、統一 `ApiError`（`VALIDATION_ERROR` ボディ）を返すか（既存 routes/mod.rs L156-179 はボディ形式未検証）。
2. 🟡 `From<ExternalSearchError> for ApiError` の配置（`models/response.rs` 推奨）。
3. 🟡 新 `ApiErrorCode` variant 名の確定（例: `ApiKeyNotConfigured` / `ExternalApiTimeout`）。
4. 🟡 ExternalSearchService 実 PgPool 依存箇所は統合 `#[ignore]` 方針で扱う。
5. 🔵 q 空文字は透過（サービス層判定）。

---

## 8. 品質判定

```
✅ 高品質:
- 要件の曖昧さ: ほぼなし（決定事項は第7章に明記）
- 入出力定義: 完全（型・制約・JSON形式を明示）
- 制約条件: 明確（既存コードとのコード文字列差分まで特定）
- 実装可能性: 確実（前提TASK-0023実装済み、対象ファイル特定済み）
- 信頼性レベル分布: 🔵 多数 / 🟡 一部（エラーコード新規追加・必須欠落テストは妥当推測）/ 🔴 なし
```

**全体評価**: 高品質。tdd-red 着手前に第7章の決定事項のみ確定すれば実装に進める。
