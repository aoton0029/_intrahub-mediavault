# mediavault-backend 開発ノート

## TASK-0012: PATCH /items/:id 部分更新実装

### 技術スタック（backend/mediavault-api/Cargo.toml）
- Rust edition 2024、workspace resolver "3"（backend/Cargo.toml）
- axum 0.8.9 / tokio 1.52.3 (full) / sqlx 0.8 (postgres, runtime-tokio, macros, chrono, uuid)
- serde 1.0.228 (derive) / serde_json 1.0.150 / uuid 1 (v4, serde) / chrono 0.4 (serde)
- dotenvy 0.15 / tracing + tracing-subscriber / tower 0.5.3 / tower-http 0.7.0 (cors)
- api-client-lib（ワークスペース内、外部API連携用クレート、本タスクでは未使用）

### 既存 UpdateItemRequest（src/models/item.rs L104-119）
```rust
pub struct UpdateItemRequest {
    pub title: Option<String>,
    pub original_title: Option<String>,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    pub release_date: Option<NaiveDate>,
    pub homepage_url: Option<String>,
    pub status: Option<ItemStatus>,
    pub consumed_date: Option<NaiveDate>,
    pub rating: Option<f32>,
    pub is_favorite: Option<bool>,
}
```
- `media_type`, `source`, `external_id` は更新不可のためフィールド自体に存在しない。
- TASK-0008時点ではDTOのデシリアライズのみ実装済み（テスト: `update_item_request_deserializes_partial_fields`、L274-282）。**title空文字のバリデーション関数は未実装**（`validate_title`はCreateItemRequest専用、`parse_create_item_request`同様の`parse_update_item_request`はまだ存在しない）。TASK-0012で新規に用意する必要がある。
- `parse_item_id(raw: &str) -> Result<Uuid, ApiError>`（L226-233）がパスパラメータUUIDパース済み関数として再利用可能（GET /items/:idで使用中）。

### item_repository.rs の既存パターン（src/repositories/item_repository.rs）
- `db_error(err: sqlx::Error) -> ApiError`（L35-40）: sqlxエラーを`tracing::error!`でログし、クライアントには`ApiErrorCode::InternalError`の汎用メッセージのみ返す。DB内部情報を漏洩させない方針。新規repository関数でも必ずこれを通すこと。
- `QueryBuilder<'_, Postgres>`によるSQL動的構築パターンが`push_item_filters`（L101-158, GET /items一覧のWHERE句構築用）に既にある。本タスクの動的UPDATE文もこの`sqlx::QueryBuilder` + `push_bind`方式を踏襲する。SET句は「1件目はカンマなし、2件目以降はカンマ区切り」というhas_condition方式（WHERE/ANDのmacro_rules!パターンと同型）が流用できる。
- `get_item_by_id(pool, id) -> Result<Option<Item>, ApiError>`（L235-246）: 存在しない場合は`None`を返し、404判定はハンドラ側に委ねる。PATCH実装でも「UPDATE実行→影響行数0なら404」または「事前にget_item_by_idで存在確認」のいずれかのパターンを選べる。タスク完了条件は「更新対象が0件だった場合ITEM_NOT_FOUND」なので、UPDATE文の`RETURNING`句がfetch_optionalで空ならNotFoundとする実装が自然（list/createで使われている`sqlx::query_as(...).fetch_one/fetch_optional`パターンに合わせる）。
- `create_item`（L48-91）は`pool.begin()`によるトランザクション例。PATCHは単一テーブル更新のみなので通常トランザクション不要だが、パターンとして参考可。
- 全フィールドNoneの場合は「何もUPDATEせず現在の状態を返す」とタスク仕様に明記（L43）。QueryBuilderでSET句が0件のときはUPDATE文を実行せず、`get_item_by_id`相当の取得のみ行う分岐が必要。

### エラーハンドリング規約（src/models/response.rs）
- `ApiErrorCode`列挙: `ValidationError`→400, `Unauthorized`→401, `ItemNotFound`→404, `UnprocessableEntity`→422, `InternalError`→500, `ExternalApiError`→502。
- `ApiError::new(code, message)`で構築。`IntoResponse`実装済みでハンドラから`Err(ApiError)`としてそのまま返せる（`Result<T, ApiError>`戻り値パターン、handlers/items.rsの既存ハンドラ参照）。
- 成功時は`ApiOk::new(data)`（200固定）。201が必要な場合は`(StatusCode::CREATED, Json(ApiOk::new(item))).into_response()`のように手動構築（`created_response`関数参照、handlers/items.rs L43-46）。PATCHは200のため`ApiOk<Item>`をそのまま戻り値型にできる（`get_item_handler`の`Result<ApiOk<ItemDetail>, ApiError>`参照）。

### DBトリガー（database-schema.sql L359-368）
- `trg_items_updated_at` がitemsテーブルのBEFORE UPDATEで`update_updated_at_column()`を実行し`NEW.updated_at = CURRENT_TIMESTAMP`を自動設定。アプリ側で`updated_at`をUPDATE文のSET句に含める必要はない（含めても上書きされる）。

### API仕様（api-endpoints.md L104-119）
- `PATCH /items/:id`: リクエスト例 `{ "rating": 4.5, "is_favorite": true }`、成功時200で更新後item、`ITEM_NOT_FOUND`（404）。

### テスト規約（既存ファイルから収集）
- ユニットテストは実装ファイル末尾に`#[cfg(test)] mod tests`としてインライン配置（別ファイルなし）。`models/item.rs`、`repositories/item_repository.rs`、`handlers/items.rs`それぞれに同パターン。
- DB非依存の純粋関数テスト（バリデーション、SQL文字列構造の検証=`builder.sql()`、`normalize_pagination`等）は`#[test]`のみで`cargo test -p mediavault-api`（無印）で実行される。
- 実DB必要な統合テストは`#[tokio::test]` + `#[ignore]`を付与し、`cargo test -- --ignored`で別途実行。`DATABASE_URL`環境変数からプール取得（`test_pool()`ヘルパー、item_repository.rs L1076-1082）。
- 統合テストはdocker-composeのPostgres（`docker compose up -d db`）を前提とし、テストごとにヘルパー関数（`insert_test_item`等）でシードデータを都度INSERT、クリーンアップ処理は明記されていない（テストDBは使い捨て前提）。
- DBエラー変換テストは`unreachable_pool()`で接続不能なPgPoolを構築し、`db_error`が`INTERNAL_ERROR`/500に変換することを確認するパターン（L1084-1089, L866-887）。
- SQL生成系のテストは実DB不要で`QueryBuilder.sql()`の文字列中身（`WHERE`/`AND`/カラム名/`EXISTS`等の有無）をassertする方針。PATCH用の動的UPDATE文も同様に文字列検証テストが書ける。
- 信頼性レベル絵文字（🔵🟡🔴）と日本語コメント（【テスト目的】【テスト内容】【期待される動作】【確認内容】等）を各テスト・各実装関数に付与する文書化規約がある。

### 注意点（TASK-0012固有）
- `title`を空文字に更新しようとした場合のみ`VALIDATION_ERROR`（他フィールドのバリデーションはタスク範囲外）。
- `media_type`, `source`, `external_id`は更新不可フィールドのためUpdateItemRequestに含まれず、SET句生成対象にもならない。
