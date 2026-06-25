# TASK-0024 要件定義書: GET /items/search 実装（ExternalSearchServiceのHTTP層公開）

**作成日**: 2026-06-25
**関連タスク**: [TASK-0024](../../tasks/mediavault-backend/TASK-0024.md)
**関連ノート**: [note.md](note.md) TASK-0024セクション
**親要件**: [requirements.md](requirements.md) REQ-002 ・ [acceptance-criteria.md](acceptance-criteria.md) TC-002-01 / TC-002-02 / TC-002-E01 / TC-002-E02
**前提タスク要件**: [TASK-0023-requirements.md](TASK-0023-requirements.md)（`ExternalSearchService` 契約）
**設計文書**: [api-endpoints.md](../../design/mediavault-backend/api-endpoints.md)「外部API検索・インポート」節 ・ [architecture.md](../../design/mediavault-backend/architecture.md)（routes→handlers→services） ・ [dataflow.md](../../design/mediavault-backend/dataflow.md)「機能1」

**【信頼性レベル凡例】**:
- 🔵 **青信号**: タスク仕様・設計文書・既存コード（note.md記載）から確実な要件
- 🟡 **黄信号**: タスク仕様・設計文書から妥当な推測による要件
- 🔴 **赤信号**: 推測による要件（本ドキュメントには無し）

---

## 1. 機能の概要

`GET /items/search` は、TASK-0023で実装した `ExternalSearchService::search(media_type, query)` をHTTP層へ公開するエンドポイントである。クエリパラメータ `media_type`（必須）・`q`（必須・検索語）を受け取り、外部API検索結果一覧 `Vec<ExternalSearchResult>` を `200` で返す。`ExternalSearchService` が返すエラー種別をハンドラ層でHTTPステータス（422 / 502）へマッピングする。🔵 *タスク概要 L16-17・note.md L5-6より*

- **何をする機能か**: `media_type` に対応する単一プロバイダ（Jikan/TMDb/NDL/OpenLibrary/IGDB等）へ検索リクエストをディスパッチし、共通DTO `ExternalSearchResult` の配列を返す。🔵 *タスク概要・TASK-0023-requirements.md 第2章より*
- **解決する問題**: フロントエンド／呼び出し元が、プロバイダ固有の認証・型差異を意識せず「media_typeとクエリ」だけで外部メタデータ検索を行えるHTTP APIを提供する。🔵 *architecture.md L27/L46・TASK-0023-requirements.md 第1章より*
- **想定ユーザー**: アイテム追加フロー（外部API検索→`POST /items/import`）を行うクライアント。🔵 *dataflow.md「機能1」より*
- **システム内での位置づけ**: Phase3「外部API連携」のHTTP公開層。前提TASK-0023の `ExternalSearchService`・TASK-0022の `find_by_provider` の上に乗り、後続TASK-0025の `POST /items/import` の前段となる。🔵 *依存タスク L19-21より*
- **対象外**: 実外部API呼び出しを伴うE2E（統合テスト要件で別途確認）、`/items/import` による登録、複数プロバイダ併用・AniList補完・Steam切替（TASK-0023の単一プロバイダ方針を継承）。🔵 *統合テスト要件 L93-94・TASK-0023-requirements.md REQ-0023-501より*

**参照したEARS要件**: REQ-002
**参照した設計文書**: api-endpoints.md（外部API検索・インポート）, architecture.md（レイヤードアーキテクチャ）, dataflow.md「機能1」

## 2. 入力・出力の仕様（GET /items/search クエリ契約）

### エンドポイント 🔵 *api-endpoints.md「外部API検索・インポート」・タスク L17より*

```
GET /items/search?media_type={media_type}&q={query}
```

### 入力: クエリパラメータDTO 🔵 *タスク実装詳細1 L33-43・note.md L34-35より*

`backend/mediavault-api/src/models/item_search.rs`（新規）に定義する。

```rust
#[derive(Debug, Deserialize)]
pub struct ItemSearchQuery {
    pub media_type: MediaType,
    pub q: String,
}
```

| パラメータ | 型 | 必須 | 制約 | 信頼性 |
|---|---|---|---|---|
| `media_type` | `MediaType`（8 variant: anime/movie/drama/manga/novel/game/academic_book/paper） | ✅ 必須 | 既存`MediaType`（`models/item.rs` L15-24、`Deserialize`実装済み）を再利用。snake_case文字列でデシリアライズ。不正値はデシリアライズ失敗→400 | 🔵 *note.md L35・L81より* |
| `q` | `String` | ✅ 必須 | 検索クエリ文字列（タイトル等）。空文字バリデーションは本タスクでは要件未指定（サービス層へ透過）。 | 🔵🟡 *タスク L37-40・TASK-0023-requirements.md 入力仕様より* |

- いずれかが未指定、または `media_type` が不正値の場合、Axumの `Query<ItemSearchQuery>` extractorのデシリアライズが失敗し、`400 VALIDATION_ERROR` を返す。🔵 *タスク L42・完了条件 L26より*
- **注記**: Axumの `Query` Rejectionは既定で素の400を返すため、統一 `ApiError`（`VALIDATION_ERROR`）形式のレスポンスボディを返すには `Query` Rejectionのカスタムハンドリングが必要か実装時に確認する（既存 `GET /items` の不正値テスト `routes/mod.rs` L156-179 はボディ形式を検証していない）。🟡 *note.md L27より妥当な推測*

### 出力（成功時 200） 🔵 *note.md L32・TASK-0023-requirements.md 第3章より*

`ApiOk<Vec<ExternalSearchResult>>` をそのまま返却する。`ExternalSearchResult`（`models/external_search.rs` L17-26、Serialize実装済み）:

```rust
pub struct ExternalSearchResult {
    pub media_type: MediaType,
    pub provider: Option<ApiProvider>, // Jikan(anime/manga)時はNone
    pub external_id: String,
    pub title: String,
    pub raw_data: serde_json::Value,
}
```

### データフロー 🔵 *note.md L8-11・dataflow.md「機能1」より*

`routes/mod.rs build_router` → `handlers::items::search_items` → ハンドラ内で `ExternalSearchService::new(state.db.clone())` を都度構築 → `service.search(query.media_type, &query.q).await` → `Ok` は `ApiOk` で200、`Err(ExternalSearchError)` は `From<ExternalSearchError> for ApiError` でHTTPステータスへマッピング。

**参照したEARS要件**: REQ-002
**参照した設計文書**: api-endpoints.md（外部API検索結果一覧）, TASK-0023-requirements.md 第3章（ExternalSearchService契約）

## 3. ハンドラ設計（ExternalSearchServiceの都度構築）

🚨 **AppStateは `ExternalSearchService` を保持しない**。`backend/mediavault-api/src/main.rs` L17-20 の `AppState { db: PgPool, internal_api_key: String }` にはサービスインスタンスのフィールドが無く、本リポジトリにサービス層インスタンスをAppStateへ注入する前例も存在しない。🔵 *note.md L8-11より*

- **設計方針**: ハンドラ内で `ExternalSearchService::new(state.db.clone())` を**呼び出しごとに構築**する。`PgPool` は内部 `Arc` 保持のため `clone()` は安価。🔵 *note.md L10より*
- `ExternalSearchService::new(pool: PgPool) -> Self`（`services/external_search.rs` L163-169）は所有型 `PgPool` を受け取る設計のため、`&state.db` ではなく `state.db.clone()` を渡す。🔵 *note.md L10より*
- 既存ハンドラ（`handlers/items.rs` 等）は `state.db` を `item_repository::*` へ直接渡す薄いパターンのみで、ハンドラ内でサービスを構築するのは本タスクが初。🔵 *note.md L11より*
- **実装ファイル**: `backend/mediavault-api/src/handlers/items.rs`（インライン `#[cfg(test)] mod tests` 規約継続）。🔵 *タスク実装詳細2 L48・note.md L38より*

```rust
pub async fn search_items(
    State(state): State<AppState>,
    Query(query): Query<ItemSearchQuery>,
) -> Result<ApiOk<Vec<ExternalSearchResult>>, ApiError> {
    let service = ExternalSearchService::new(state.db.clone());
    let results = service.search(query.media_type, &query.q).await?; // From<ExternalSearchError> で変換
    Ok(ApiOk::new(results))
}
```

- **テスト容易性**: `ExternalSearchService` はPgPool直接構築でモック化困難なため、エラー型→`ApiError`マッピング部分（`From` 実装）を関数分離して単体テスト可能にする。🟡 *note.md L38より妥当な推測*

## 4. エラーマッピング仕様

`ExternalSearchError`（`models/external_search.rs` L33-40、`impl std::error::Error` 済み）を `ApiError` へ変換する `impl From<ExternalSearchError> for ApiError` を新規追加する。🔵 *note.md L31・タスク実装詳細3 L50-53より*

🚨 **新規 `ApiErrorCode` variantが必要**（既存variantはワイヤーコード文字列が要件と不一致）。`backend/mediavault-api/src/models/response.rs` の `ApiErrorCode` enum と `code_and_status()`（`code_and_status` 相当のマッピング）に以下を追加する。🔵 *note.md L24-26・L44より*

| ExternalSearchError variant | 追加する ApiErrorCode | HTTPステータス | ワイヤーコード | 既存variantを流用しない理由 | 信頼性 |
|---|---|---|---|---|---|
| `ApiKeyNotConfigured(ApiProvider)` | `ApiKeyNotConfigured`（新規） | `422 Unprocessable Entity` | `"API_KEY_NOT_CONFIGURED"` | 既存 `UnprocessableEntity`（422）はコード文字列が `"UNPROCESSABLE_ENTITY"` 固定で要件と不一致。🔵 | 🔵 *タスク L27・note.md L25より* |
| `ExternalApiError(api_client_lib::ApiError)` | `ExternalApiTimeout`（新規） | `502 Bad Gateway` | `"EXTERNAL_API_TIMEOUT"` | 既存 `ExternalApiError`（502）はコード文字列が `"EXTERNAL_API_ERROR"` で要件と不一致。新規variant追加が要件のコード文字列要求に最忠実。🔵 | 🔵 *タスク L28・note.md L26/L44より* |
| （クエリ欠落・media_type不正値） | `ValidationError`（既存流用） | `400 Bad Request` | `"VALIDATION_ERROR"` | 既存 `ApiErrorCode::ValidationError`（response.rs L51/L96）をそのまま使用。`Query` extractorのデシリアライズ失敗で発生。🔵 | 🔵 *note.md L27より* |

- `api-client-lib::ApiError` の6 variant（`Http{status,body}` / `Auth` / `RateLimit{retry_after}` / `Parse` / `Timeout` / `Network`）はすべて `ExternalApiError` 経由で `502 EXTERNAL_API_TIMEOUT` に集約される。🔵 *note.md L70・TASK-0023-requirements.md 第4章より*
- `?` 演算子で `Result` を伝播させ **panicさせない**（プロセスはクラッシュしない）。🔵 *タスク完了条件 L28・テストケース4 L80-84より*
- **実装ファイル**: タスクファイルが指す `backend/mediavault-api/src/errors.rs` は**実在しない**。`From<ExternalSearchError> for ApiError` は `handlers/items.rs` 内、または `models/response.rs` への追記（既存規約）に読み替える。🔵 *note.md L31・L43より*

## 5. ルート登録仕様

🚨 タスクファイルが指す `backend/mediavault-api/src/routes/items.rs` は**実在しない**。`routes/` 配下は `mod.rs` のみで、全エンドポイントが `build_router` 関数内に `.route(...)` をフラットに列挙する単一ファイル構成。🔵 *note.md L13-14より*

- `backend/mediavault-api/src/routes/mod.rs` の `build_router` 内に追記する。🔵 *note.md L14より*
- **`/items/search` を `/items/:id` より前に登録する**。既存ルートは `/items`（GET一覧+POST）と `/items/:id`（GET/PATCH/DELETE、L44-49）のみ。🔵 *タスク注意事項 L106・note.md L15より*
- Axum 0.8 はリテラルパス（`/items/search`）を動的パス（`/items/:id`）より優先マッチするため登録順序自体の実害は低いが、タスク注意事項・可読性・将来のバージョン変更への安全策として前方に置く。🔵 *note.md L15より*

```rust
.route("/items", get(list_items_handler).post(create_item_handler))
// 【TASK-0024】: GET /items/search（外部API検索）を /items/:id より前に登録
.route("/items/search", get(search_items))
.route("/items/:id", get(get_item_handler).patch(update_item_handler).delete(delete_item_handler))
```

## 6. 機能要件（EARS記法）

### 通常要件
- REQ-0024-01: システムは `GET /items/search?media_type=&q=` を受理し、`Query<ItemSearchQuery>` で `media_type`・`q` をデシリアライズしなければならない。🔵 *タスク L17/L33-42より*
- REQ-0024-02: システムはハンドラ内で `ExternalSearchService::new(state.db.clone())` を構築し、`search(media_type, &q)` を呼び出さなければならない。🔵 *note.md L8-11より*
- REQ-0024-03: `media_type=anime`（または manga）の場合、システムはJikan経由の検索結果を `200` で返さなければならない。🔵 *完了条件 L24・TC-002-01より*
- REQ-0024-04: `media_type=movie`（または drama）の場合、システムはTMDb経由の検索結果を `200` で返さなければならない。🔵 *完了条件 L25・TC-002-02より*
- REQ-0024-05: システムは成功時 `ApiOk<Vec<ExternalSearchResult>>` を返さなければならない。🔵 *note.md L32より*

### 条件付き要件
- REQ-0024-101: `media_type` または `q` のいずれかが未指定、または `media_type` が不正値の場合、システムは `400 VALIDATION_ERROR` を返さなければならない。🔵 *完了条件 L26・タスク L42より*
- REQ-0024-102: `ExternalSearchService` が `ApiKeyNotConfigured` を返した場合、システムは `422` ・ワイヤーコード `"API_KEY_NOT_CONFIGURED"` を返さなければならない。🔵 *完了条件 L27・TC-002-E01・EDGE-001より*
- REQ-0024-103: `ExternalSearchService` が `ExternalApiError`（タイムアウト等）を返した場合、システムは `502` ・ワイヤーコード `"EXTERNAL_API_TIMEOUT"` を返し、プロセスをクラッシュさせてはならない。🔵 *完了条件 L28・TC-002-E02より*

### 制約要件
- REQ-0024-401: システムは `models/response.rs` の `ApiErrorCode` に新規variant（`ApiKeyNotConfigured`→`"API_KEY_NOT_CONFIGURED"`/422、`ExternalApiTimeout`→`"EXTERNAL_API_TIMEOUT"`/502）を追加しなければならない（既存variant流用はコード文字列不一致を生むため不可）。🔵 *note.md L24-26/L44より*
- REQ-0024-402: システムは `/items/search` を `/items/:id` より前に `routes/mod.rs` の `build_router` 内へ登録しなければならない。🔵 *タスク注意事項 L106・note.md L15より*
- REQ-0024-403: システムは `ExternalSearchService::search` インターフェース（TASK-0023）を変更してはならない。🔵 *TASK-0023-requirements.md 契約より*

## 7. 非機能要件
- NFR-0024-01: システムは独自のレスポンス形式を新設せず、成功は `ApiOk`、失敗は `ApiError`（`IntoResponse` 済み）に統一しなければならない。🔵 *note.md L31-32・既存規約より*
- NFR-0024-02: 外部APIエラー・APIキー未設定時にDB内部情報や外部API生エラー詳細をクライアントへ漏洩させてはならない（既存 `ApiError` の汎用メッセージ方針踏襲）。🟡 *TASK-0022/0012のdb_error方針からの妥当な推測*
- NFR-0024-03: テストは既存 `routes/mod.rs` の `test_app_state()` + `#[ignore]`（`cargo test -- --ignored`）統合テストパターンに合流させなければならない。🔵 *note.md L39より*

## 8. 想定される使用例（Given/When/Then・エッジケース）

### シナリオ1: anime検索でJikan結果が返る（TC-002-01） 🔵
- **Given**: `ExternalSearchService`（モック）が `media_type=anime` に検索結果配列を返す
- **When**: `GET /items/search?media_type=anime&q=鬼滅`
- **Then**: `200` が返り、ボディにJikanの検索結果一覧が含まれる

### シナリオ2: movie/drama検索でTMDb結果が返る（TC-002-02） 🔵
- **Given**: `ExternalSearchService`（モック）が `media_type=movie` にTMDb結果を返す
- **When**: `GET /items/search?media_type=movie&q=タイトル`
- **Then**: `200` が返り、TMDbの検索結果一覧が含まれる

### シナリオ3: APIキー未設定で422（TC-002-E01, EDGE-001） 🟡
- **Given**: `ExternalSearchService` が `ApiKeyNotConfigured` を返す
- **When**: `GET /items/search?media_type=movie&q=タイトル`
- **Then**: `422` が返り、ワイヤーコードは `API_KEY_NOT_CONFIGURED`

### シナリオ4: 外部APIタイムアウトで502（TC-002-E02） 🟡
- **Given**: `ExternalSearchService` が `ExternalApiError`（`ApiError::Timeout`等）を返す
- **When**: `GET /items/search?media_type=movie&q=タイトル`
- **Then**: `502` が返り、ワイヤーコードは `EXTERNAL_API_TIMEOUT`、プロセスはpanicしない

### エッジケース
- EDGE-0024-01: `q` パラメータ欠落 → `400 VALIDATION_ERROR`。🟡 *タスクテストケース5 L86-90より*
- EDGE-0024-02: `media_type` パラメータ欠落 → `400 VALIDATION_ERROR`。🔵 *完了条件 L26より*
- EDGE-0024-03: `media_type` 不正値（例 `?media_type=foo`）→ `Query` デシリアライズ失敗 → `400 VALIDATION_ERROR`。🔵 *タスク L42より*
- EDGE-0024-04: `/items/search` が `/items/:id` に誤マッチしない（リテラル優先・前方登録）。🔵 *note.md L15より*
- EDGE-0024-05: `api-client-lib::ApiError` のいずれの variant（Http/Auth/RateLimit/Parse/Timeout/Network）も `502 EXTERNAL_API_TIMEOUT` に集約され、panicしない。🔵 *note.md L70より*

**参照したEARS要件**: REQ-002, TC-002-01 / TC-002-02 / TC-002-E01 / TC-002-E02, EDGE-001
**参照した設計文書**: dataflow.md「機能1」, api-endpoints.md（外部API検索・インポート）

## 9. 完了基準（タスクファイル6条件との対応）

| # | タスクファイル完了条件（L24-29） | 本要件での対応 | 信頼性 |
|---|---|---|---|
| 1 | `?media_type=anime&q=...` でJikan検索結果が返る（TC-002-01） | REQ-0024-03・シナリオ1 | 🔵 |
| 2 | `?media_type=movie&q=...`（drama）でTMDb検索結果が返る（TC-002-02） | REQ-0024-04・シナリオ2 | 🔵 |
| 3 | `media_type`・`q` 未指定時に `400 VALIDATION_ERROR` | REQ-0024-101・EDGE-0024-01/02/03 | 🔵 |
| 4 | APIキー未設定時に `422 API_KEY_NOT_CONFIGURED`（TC-002-E01, EDGE-001） | REQ-0024-102・第4章マッピング表・シナリオ3 | 🔵 |
| 5 | 外部APIタイムアウト時に `502 EXTERNAL_API_TIMEOUT` を返しクラッシュしない（TC-002-E02） | REQ-0024-103・第4章マッピング表・シナリオ4 | 🔵 |
| 6 | 単体テストがすべて成功する | 第8章シナリオ1-4 + エッジケースのテスト化（tdd-testcases） | 🔵 |

### 追加完了基準（実コード構成に基づく）
- [ ] `models/item_search.rs`（新規）に `ItemSearchQuery` が定義されている。🔵 *note.md L34-35*
- [ ] `models/response.rs` の `ApiErrorCode` に `API_KEY_NOT_CONFIGURED`(422)・`EXTERNAL_API_TIMEOUT`(502) の新規variantが追加されている。🔵 *note.md L24-26*
- [ ] `From<ExternalSearchError> for ApiError` が `handlers/items.rs` または `models/response.rs` に実装されている（`errors.rs` は不在のため）。🔵 *note.md L31*
- [ ] `routes/mod.rs build_router` 内で `/items/search` が `/items/:id` より前に登録されている。🔵 *note.md L15*

## 10. EARS要件・設計文書との対応関係

- **参照した機能要件**: REQ-002
- **参照した非機能要件**: （本HTTP層に直接対応するNFRは無し。エラー非漏洩/ログ方針はTASK-0022踏襲）
- **参照したEdgeケース**: TC-002-E01 / TC-002-E02 / EDGE-001（本ドキュメントで EDGE-0024-01〜05 を新設）
- **参照した受け入れ基準**: TC-002-01 / TC-002-02 / TC-002-E01 / TC-002-E02
- **参照した設計文書**:
  - **API仕様**: api-endpoints.md「外部API検索・インポート」（`GET /items/search` クエリ契約・エラーコード）
  - **アーキテクチャ**: architecture.md（routes→handlers→services レイヤード方針）
  - **データフロー**: dataflow.md「機能1: 外部API検索→アイテム追加」
  - **前提契約**: TASK-0023-requirements.md（`ExternalSearchService` API・`ExternalSearchError`・`ExternalSearchResult`）
  - **既存コード現況**: note.md TASK-0024セクション（AppState=PgPoolのみ・routes/mod.rsフラット構成・errors.rs不在・ApiErrorCode新規variant要件・item_search.rs新規）

## 11. 信頼性レベルサマリー

| カテゴリ | 🔵 | 🟡 | 🔴 | 合計 |
|---|---|---|---|---|
| 入出力仕様 | 4 | 2 | 0 | 6 |
| ハンドラ設計 | 4 | 1 | 0 | 5 |
| エラーマッピング表 | 3 | 0 | 0 | 3 |
| 機能要件（通常） | 5 | 0 | 0 | 5 |
| 機能要件（条件付き） | 3 | 0 | 0 | 3 |
| 機能要件（制約） | 3 | 0 | 0 | 3 |
| 非機能要件 | 2 | 1 | 0 | 3 |
| エッジケース | 4 | 1 | 0 | 5 |
| 完了基準 | 6 | 0 | 0 | 6 |

**全体評価**: 高品質（赤信号なし）。黄信号は (1) `q` 空文字バリデーション未指定、(2) `Query` Rejectionの統一レスポンス整形要否、(3) エラーマッピング部の関数分離によるテスト容易化、(4) エラー非漏洩NFRの踏襲推測 に限定。いずれもtdd-red着手前に確定可能。

---

## 次フェーズへの引き渡し事項

- `tdd-testcases` フェーズでは、シナリオ1〜4を中核とし、EDGE-0024-01〜05（パラメータ欠落/不正値による400、ルート誤マッチ防止、全ApiError集約502）を追加洗い出しすること。
- `tdd-red` 着手前に以下を確定すること:
  1. **`Query` Rejection整形**: 不正クエリで統一 `ApiError`（`VALIDATION_ERROR`ボディ）を返すか、素のAxum 400で許容するか（第2章注記・note.md L27）。
  2. **`From<ExternalSearchError>` の配置**: `handlers/items.rs` 内か `models/response.rs` か（`errors.rs` は不在、note.md L31）。
  3. **エラーマッピングのテスト容易化**: `From` 実装をDB非依存で単体テストする方針（note.md L38）。
  4. **新規 `ApiErrorCode` variant名**: `ApiKeyNotConfigured` / `ExternalApiTimeout`（または同等）の確定（第4章）。
