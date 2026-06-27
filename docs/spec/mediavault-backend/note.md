# mediavault-backend 開発ノート

## TASK-0029: 内部REST APIルート群実装（/internal/items等）

### Greenフェーズ実装サマリー
- `routes/internal.rs`の`build_internal_router(state) -> Router`を実装。`/internal/items`(POST)・
  `/internal/items/search`(GET)・`/internal/items/:id`(PATCH)・`/internal/items/:id/groups`(POST)・
  `/internal/groups/:group_id/episodes`(POST)・`/internal/items/:id/files`(POST)を`api_key_auth`
  ミドルウェア配下にマウント。`/internal/items/search`は`/internal/items/:id`より前に登録し誤マッチを防止。
- items本体のPOST/PATCH・検索は既存`handlers::items::{create_item_handler, update_item_handler,
  list_items_handler}`をそのまま再利用（新規ハンドラなし）。検索は`ListItemsQuery`の既存`title`
  フィールド（TASK-0024時点で追加済み）でフィルタするため、外部API検索用`search_items_handler`
  ではなく`list_items_handler`を流用した点が設計判断のポイント。
- files登録は既存`handlers::item_files::create_item_file_handler`をそのまま再利用。
- groups/episodesはRedフェーズで既に`item_group_repository::upsert_item_group`・
  `item_episode_repository::upsert_item_episode`（INSERT…ON CONFLICT DO UPDATE等）が実装済みだったため、
  Greenフェーズでは新規ハンドラ`handlers/internal_groups.rs::upsert_item_group_handler`・
  `handlers/internal_episodes.rs::upsert_item_episode_handler`を追加し、これらのupsert関数を呼び出す
  薄いラッパーとして実装した（既存`create_item_group_handler`/`create_item_episode_handler`とは
  upsert/insertの違いのみで対称構造）。
- `main.rs`では`routes::build_router(state.clone()).merge(routes::internal::build_internal_router(state))`
  でメインルーターと内部ルーターをmergeしてサーバーに登録。
- Docker未起動環境のため`#[ignore]`統合テスト（実DB必要）は未実行。コンパイル成功・DB非依存ユニットテスト
  164件全成功で確認済み（`cargo build -p mediavault-api` / `cargo test -p mediavault-api`）。

### 依存タスクの状態に関する補足
- TASK-0029着手時点でoverview.mdはTASK-0012（PATCH /items/:id）・TASK-0013（DELETE /items/:id）を
  未完了表示していたが、実際は`handlers/items.rs`に`update_item_handler`/`delete_item_handler`が
  実装済みだった（overview.mdの更新漏れ）。両タスクのチェックボックスを完了に更新した。

## TASK-0024: GET /items/search 実装

### タスク概要
`GET /items/search?media_type=&q=` を新設し、TASK-0023の`ExternalSearchService::search`をHTTP層へ公開する。エラーマッピング: `ApiKeyNotConfigured`→422 `API_KEY_NOT_CONFIGURED`、`ExternalApiError`→502 `EXTERNAL_API_TIMEOUT`、クエリパラメータ欠落/不正値→400 `VALIDATION_ERROR`。

### 🚨 AppStateはPgPoolのみ保持（ExternalSearchServiceは未注入、ハンドラ内で都度構築する設計）
- `backend/mediavault-api/src/main.rs` L17-20: `pub struct AppState { pub db: PgPool, pub internal_api_key: String }`。`ExternalSearchService`をフィールドとして持たない。
- TASK-0023の`ExternalSearchService::new(pool: PgPool) -> Self`（`backend/mediavault-api/src/services/external_search.rs` L163-169）はPgPoolを所有型で受け取る設計のため、ハンドラ内で`ExternalSearchService::new(state.db.clone())`のように都度構築して呼び出す必要がある（`PgPool`は内部`Arc`保持でclone安価、既存ハンドラに前例はないが`item_repository`関数群が`&state.db`を直接渡すパターンと対比的）。
- 既存ハンドラ（`handlers/items.rs`, `handlers/settings.rs`等）はいずれも`state.db`を直接`item_repository::*`へ渡す薄いパターンのみで、サービス層インスタンスをハンドラ内で構築する前例は本タスクが初。

### ルーティング登録: タスクファイル記載の`routes/items.rs`は実在しない、`routes/mod.rs`へ直接追記する
- `backend/mediavault-api/src/routes/`配下には`mod.rs`のみが存在し、`items.rs`等のサブモジュールファイルは無い（全エンドポイントが`build_router`関数内に`.route(...)`をフラットに列挙する単一ファイル構成）。タスクファイル「実装ファイル: `backend/mediavault-api/src/routes/items.rs`」という記載は実コード構成と不一致のため、実装時は既存規約に従い`backend/mediavault-api/src/routes/mod.rs`の`build_router`内に追記する。
- 動的パス競合に関する実害確認: 既存ルートは`/items`（GET一覧+POST作成）と`/items/:id`（GET詳細+PATCH+DELETE、`routes/mod.rs` L44-49）のみで、現状`/items/search`は未登録。Axum 0.8の`Router`はリテラルパス（`/items/search`）と動的パス（`/items/:id`）が同一階層に共存する場合、リテラル一致を優先してマッチする実装のため、登録順序自体は技術的には問題にならない可能性が高い。ただしタスクファイル注意事項（L106「`/items/search`は`/items/:id`より前に定義」）に従い、`.route("/items/search", get(search_items_handler))`を`.route("/items/:id", ...)`より前の行に置くことで将来のAxumバージョン変更や可読性の観点からも安全側に倣う。
- 追記イメージ（既存パターン踏襲、`.route("/items", ...)`の直後・`.route("/items/:id", ...)`の直前に挿入）:
  ```rust
  .route("/items", get(list_items_handler).post(create_item_handler))
  // 【TASK-0024】: GET /items/search（外部API検索）を/items/:idより前に登録
  .route("/items/search", get(search_items_handler))
  .route("/items/:id", get(get_item_handler).patch(update_item_handler).delete(delete_item_handler))
  ```

### models/response.rs の ApiErrorCode: 422/502用の新規variantが必要（現状は汎用ExternalApiErrorのみ存在）
- `backend/mediavault-api/src/models/response.rs` L49-90の`ApiErrorCode` enumには`API_KEY_NOT_CONFIGURED`（422）に対応するvariantが存在しない。`UnprocessableEntity`（L54, 422）は既存するが、コード文字列が`"UNPROCESSABLE_ENTITY"`固定（L99-101）であり、タスク要求の`"API_KEY_NOT_CONFIGURED"`という具体的コード文字列とは異なる。新規variant`ApiKeyNotConfigured`を追加し`code_and_status()`に`("API_KEY_NOT_CONFIGURED", StatusCode::UNPROCESSABLE_ENTITY)`を追記する必要がある（既存`InvalidProvider`等のTASK単位追加パターンを踏襲）。
- `ExternalApiError`（L56）は既存し`("EXTERNAL_API_ERROR", StatusCode::BAD_GATEWAY)`（L103）にマッピングされるが、コード文字列が`"EXTERNAL_API_ERROR"`であり、タスク要求の`"EXTERNAL_API_TIMEOUT"`とは異なる。本タスクでは既存`ExternalApiError`を流用せず、新規variant（例: `ExternalApiTimeout`）を追加し`("EXTERNAL_API_TIMEOUT", StatusCode::BAD_GATEWAY)`を割り当てるか、既存`ExternalApiError`のコード文字列自体を`EXTERNAL_API_TIMEOUT`へ変更するかの設計判断が必要（既存`ExternalApiError`を使う既存テスト`external_api_error_returns_502`（L246-252、response.rs）がコード文字列を直接assertしていないため、文字列変更によるリグレッションリスクは低い）。
- `VALIDATION_ERROR`（400）は既存の`ApiErrorCode::ValidationError`（L51, L96）がそのまま使える。クエリパラメータ必須欠落・`media_type`不正値はAxumの`Query<ItemSearchQuery>` extractor自体のデシリアライズ失敗時にAxumのデフォルト400レスポンス（`Rejection`）が返り、既存`ApiError`形式の統一レスポンスを返したい場合は`Query`のRejectionをカスタムハンドリングする実装（既存`GET /items`の`media_type=invalid`時の挙動、`routes/mod.rs` L156-179のテストでは素のAxum 400のみを確認しレスポンスボディ形式は検証していない点に注意）が必要か確認すること。

### `services/external_search.rs`からのエラー型・呼び出し方法（再掲・本タスクで直接使用）
- `ExternalSearchService::search(&self, media_type: MediaType, query: &str) -> Result<Vec<ExternalSearchResult>, ExternalSearchError>`（L199-214）。
- `ExternalSearchError`（`backend/mediavault-api/src/models/external_search.rs` L33-40）: `ApiKeyNotConfigured(ApiProvider)` / `ExternalApiError(api_client_lib::ApiError)`の2variant。`impl std::error::Error`実装済み（L52）。`From<ExternalSearchError> for ApiError`の変換実装をハンドラ層または`errors.rs`相当の場所に新規追加する（タスクファイルL52-53は`errors.rs`を想定しているが、当該ファイルは現状未確認——実コードに専用`errors.rs`が存在するか要確認。無い場合は`handlers/items.rs`内に`impl From<ExternalSearchError> for ApiError`を直接書くか、`models/response.rs`に追記する既存規約に倣う）。
- `ExternalSearchResult`（`models/external_search.rs` L17-26）: `media_type: MediaType, provider: Option<ApiProvider>, external_id: String, title: String, raw_data: serde_json::Value`。レスポンスは`ApiOk<Vec<ExternalSearchResult>>`でそのまま200返却可能（Serialize実装済み、L8）。

### クエリパラメータDTO（新規ファイル、既存パターン踏襲）
- タスクファイル指定の`ItemSearchQuery { media_type: MediaType, q: String }`は`backend/mediavault-api/src/models/item_search.rs`に新規作成想定（現状未作成）。既存`ListItemsQuery`（`models/item.rs`内、L81周辺で参照されている）と同様、Axumの`Query<T>`extractorで使うため`#[derive(Deserialize)]`必須。`MediaType`は既に`Deserialize`実装済み（`models/item.rs` L12-14）のためそのまま再利用可能。

### ハンドラ・テスト規約（既存方針を継続）
- `handlers/items.rs`内にインライン`#[cfg(test)] mod tests`で配置する既存パターン（L195以降）を継続。DB非依存ユニットテストは`ExternalSearchService`をモック化困難な構造（PgPoolで直接構築）のため、ハンドラ単体のロジック検証（エラー型→ApiErrorのマッピング部分のみ）は関数分離してテスト容易性を確保するか、TASK-0023同様`with_fixed_credentials`等のDI経路を活用したテスト用ヘルパーをハンドラ層にも用意する設計判断が要る。
- 既存ルーティング統合テストパターン（`routes/mod.rs`内`test_app_state()` + `#[ignore]` + `cargo test -- --ignored`、L144-154）に合流させる想定。
- `AGENTS.md`、`docs/rule/`ディレクトリは本リポジトリに存在しない（再確認済み、追加ルールなし）。

### 注意事項
- タスクファイルが指す`backend/mediavault-api/src/routes/items.rs`・`backend/mediavault-api/src/errors.rs`は実コードに存在しないため、実装時は実際のファイル構成（`routes/mod.rs`、`models/response.rs`または`handlers/items.rs`内）に読み替えること。
- `ApiErrorCode`への`API_KEY_NOT_CONFIGURED`(422)・`EXTERNAL_API_TIMEOUT`(502)追加は、既存`UnprocessableEntity`/`ExternalApiError`のコード文字列とは別物として新規定義する方針が要件のコード文字列要求に最も忠実（既存variant流用はコード文字列不一致を生む）。
- `/items/search`は`/items/:id`より前に登録すること（Axum 0.8ではリテラル優先のため実害は低いと推測されるが、タスクファイル注意事項に明記の安全策として踏襲）。

---

## TASK-0023: ExternalSearchServiceラッパー実装（media_type→provider振り分け）

### タスク概要
`media_type`（anime/movie/drama/manga/novel/game/paper/book等）に応じて`api-client-lib`の各プロバイダクライアントの`ApiClient::execute`を呼び出すディスパッチサービス`ExternalSearchService`を`backend/mediavault-api/src/services/external_search.rs`に新設する。キーが必要なプロバイダはTASK-0022の`find_by_provider`でDBからキーを取得し、未設定時は`ExternalSearchError::ApiKeyNotConfigured`を返す。

### 🚨 最重要確認事項: api-client-lib は実在し利用可能（ブロッカーなし）
- `backend/Cargo.toml`のワークスペースmembersに`api-client-lib`が含まれ、`backend/mediavault-api/Cargo.toml`は既に`api-client-lib = { path = "../api-client-lib" }`を依存に追加済み（TASK-0012ノートにも「未使用」と記載されていたが依存自体は既存）。
- クレート本体は`backend/api-client-lib/`に実装済み。7プロバイダ全てのクライアント構造体とモデル/リクエスト型が揃っている：
  - `backend/api-client-lib/src/clients/jikan/mod.rs`（`JikanClient`、キー不要、レート制限3req/秒）
  - `backend/api-client-lib/src/clients/tmdb/mod.rs`（`TmdbClient::new(AuthStrategy)`、ApiKey/Bearer認証、レート制限40req/10秒）
  - `backend/api-client-lib/src/clients/ndl/mod.rs`, `openlibrary/mod.rs`, `steam/mod.rs`, `igdb/mod.rs`, `anilist/mod.rs`（同様の構成、各`mod.rs`/`models.rs`/`requests.rs`）
- 共通トレイト`ApiClient`（`backend/api-client-lib/src/traits.rs`）:
  ```rust
  pub trait ApiClient {
      type Request;
      type Model;
      fn execute(&self, request: Self::Request)
          -> impl std::future::Future<Output = Result<ApiResponse<Self::Model>, ApiError>> + Send;
  }
  ```
  各クライアント型ごとに`Request`/`Model`のassociated typeが異なる（`JikanRequest`/`JikanModel`、`TmdbRequest`/`TmdbModel`等、プロバイダ間で型が統一されていない）。**重要**: `execute`は`impl Future`を返すRPITIT形式（`async fn` in trait相当）であり、`dyn ApiClient`としてトレイトオブジェクト化できない（dyn非互換）。ExternalSearchServiceでmedia_type→クライアントの動的ディスパッチを行う際は、enumによる手動分岐（各プロバイダ型を直接構築してmatchする）か、ジェネリクスで対応する設計が必要。
- 共通エラー型`ApiError`（`backend/api-client-lib/src/error.rs`）: `Http{status,body}` / `Auth(String)` / `RateLimit{retry_after}` / `Parse(String)` / `Timeout` / `Network(String)`の6variant。`ExternalSearchError::ExternalApiError`へのマッピング元となる。
- 共通レスポンス型`ApiResponse<T>`（`backend/api-client-lib/src/response.rs`）: `request: RequestResult{status,url,latency_ms}` / `raw: RawData(Json|Xml)` / `model: T`。
- `lib.rs`で`AuthStrategy`/`ApiError`/`ApiResponse`/`RawData`/`RequestResult`/`ApiClient`がクレートルートからre-export済み（`api_client_lib::ApiClient`等で参照可能）。
- 各クライアントは`new()`または`new_with_base_url()`（テスト用にベースURLを注入可能）、キー必要なものは`new(auth: AuthStrategy)`の形。`AuthStrategy`は`backend/api-client-lib/src/auth.rs`に定義（`ApiKey(String)`/`Bearer(String)`等のvariantを持つ想定、TmdbClientの`apply_auth`実装から確認）。

### 🚨 mockallは未導入（ワークスペース全体で依存に存在しない）
- `backend/mediavault-api/Cargo.toml`・`backend/api-client-lib/Cargo.toml`のいずれにも`mockall`記載なし。`api-client-lib`には`[dev-dependencies]`セクション自体が存在しない（プロバイダ別`tests/*_test.rs`統合テストのみで、ユニットテスト用モックの仕組みは未整備）。
- タスクファイル注意事項に明記の通り「モックには`mockall`等の利用を想定（既存依存に追加が必要な場合はTASK-0001の依存設定を更新する）」——本タスクで`mediavault-api/Cargo.toml`の`[dev-dependencies]`に`mockall`を新規追加する必要がある。
- 設計上の注意: `ApiClient::execute`がdyn非互換（RPITIT）のため、`mockall::automock`を素のトレイトに直接適用できない可能性がある（mockallはRPITITを部分的にサポートするが、async_trait形式の方が実績が多い）。ExternalSearchService内部では各プロバイダクライアントを直接構築する設計とし、HTTP層をモックする（例: `new_with_base_url`でテスト用モックサーバーのURLを注入、または`wiremock`crateの利用）方が、`ApiClient`トレイト自体をモックするより確実な可能性がある。テストケース1・2は「クライアントのexecuteのみが呼ばれる」という呼び出し検証を要求しているため、トレイトのモック化要否は設計判断が必要（mockallでのモック化が技術的に難しい場合、呼び出し検証ではなくHTTPモックサーバーへのリクエスト到達確認に置き換える等の代替案を検討すること）。

### MediaType / ApiProvider の実コード定義
- `MediaType`（`backend/mediavault-api/src/models/item.rs` L15-24）: `Anime, Movie, Drama, Manga, Novel, Game, AcademicBook, Paper`の8variant（`#[sqlx(type_name="media_type", rename_all="snake_case")]`、`Serialize`/`Deserialize`実装済み）。タスクファイルが言う「book」はDB上`academic_book`（`AcademicBook`）に対応。
- `ApiProvider`（`backend/mediavault-api/src/models/api_credential.rs` L22-32, TASK-0022実装）: `Tmdb, Igdb, Ndl, Steam, OpenLibrary, AniList`の6variant（Jikanはキー不要のため対象外）。`find_by_provider(pool, provider) -> Result<Option<ApiCredential>, ApiError>`（`backend/mediavault-api/src/repositories/api_credential_repository.rs` L62-77）がそのまま利用可能。

### 🟡 media_type→provider マッピングの未確定点（要設計判断、実装着手前に決定すること）
1. **Game → Steam or IGDB**: `MediaType::Game`に対し`api-client-lib`はSteam・IGDBの両クライアントを提供するが、要件上どちらを使うかmedia_type単独では一意に決まらない。タスクファイルが提示する2案: (a) クエリパラメータで明示的にプロバイダ指定させる、(b) 優先順位を固定（例: IGDB優先、Steam fallback）。本ノート作成時点では未決定。実装時にどちらを採用したかをコミットログまたは本ノートの追記として残すこと。
2. **Manga / Novel → OpenLibrary vs NDL**: タスクファイルは「manga/novel → OpenLibrary」「paper/book(academic_book) → NDL」と一旦割り振っているが、要件定義に明記が薄く妥当な推測との注記あり。日本語マンガ・ライトノベルの書誌情報としてはNDL（国立国会図書館）の方が適切な可能性もあり、要設計判断。
3. **Anime → Jikan + AniList併用方針**: AniListをJikanの補完として呼ぶか、anime用には常にJikanのみで良いか要件に明記なし。本タスクの完了条件（テストケース1）は「media_type=animeでJikanのみが呼ばれ他は呼ばれない」を明示しているため、現テストケース定義に従う限りAniListは本タスクのディスパッチには含めない実装が妥当（ただし将来拡張の余地を設計コメントに残すこと）。
4. 上記3点はテストケース1・2（🔵信頼性、anime→Jikan・movie/drama→TMDb）には影響しないため、まずテストケース1・2・3・4（タスクファイル記載の4ケース）を通す実装を優先し、game/manga/novelの分岐は要件確認後に追記する進め方も可能。

### エラー型設計
- `ExternalSearchError`は新規定義（`backend/mediavault-api/src/services/external_search.rs`内、または`models/external_search.rs`）。variant: `ApiKeyNotConfigured(ApiProvider)` / `ExternalApiError(ApiError)`（api-client-libの`ApiError`をラップ）。
- 既存`ApiErrorCode`（`backend/mediavault-api/src/models/response.rs`）には既に`ExternalApiError`（L56, EXTERNAL_API_ERROR→502）が存在するため、ハンドラ層（後続TASK-0024）でのマッピング先は流用可能。`ApiKeyNotConfigured`に対応する422マッピング用コードは未確認（要追加検討、後続タスクの範囲）。
- `services/mod.rs`は現状空ファイル（中身なし）。`pub mod external_search;`の追記が必要。

### レスポンス共通化（ExternalSearchResult）
- `media_type`・`provider`・`external_id`・`title`・`raw_data`(JSON)等を含む共通DTOを各プロバイダ用アダプタ関数で生成する設計（🟡推測、ラップ形式は実装詳細未確定）。`backend/mediavault-api/src/models/external_search.rs`は未作成（新規ファイル）。

### テスト規約（既存方針を継続、TASK-0022/0012ノート参照）
- インラインテスト（`#[cfg(test)] mod tests`を実装ファイル末尾）。DB非依存のディスパッチロジック単体テストが本タスクの主対象（モック使用、上記mockall課題に注意）。
- 実DB必要なテスト（`find_by_provider`経由のキー取得確認）は`#[tokio::test]` + `#[ignore]`、`DATABASE_URL`環境変数使用パターンを踏襲。
- 外部API実呼び出しを伴う統合テストはタスク範囲外（後続TASK-0024のハンドラレベルで結合確認）。

### 注意事項
- `ApiClient`トレイトの既存インターフェースは変更しないこと（architecture.md「互換性制約」）。
- game/manga/novelの複数候補プロバイダ問題は実装時に設計判断をコミットログ等に残すこと（上記🟡参照）。
- `AGENTS.md`、`docs/rule/`ディレクトリは本リポジトリに存在しない（追加ルールなし、TASK-0022ノートで確認済み、再確認済み）。

---

## TASK-0022: api_credentials（外部APIキー管理）CRUD実装

### タスク概要
`PUT /settings/api-keys/:provider` で外部APIキー（tmdb/igdb/ndl/steam/open_library/ani_list）を`api_credentials`テーブルにupsertする。Jikanはキー不要のためenum対象外、不正provider文字列は`400 INVALID_PROVIDER`。

### 技術スタック・モジュール構成（実コード現況）
- 実コードのDB層モジュール名は `db::api_credentials` ではなく `repositories/` 配下（`backend/mediavault-api/src/repositories/`）。既存ファイルは`category_repository.rs`, `item_repository.rs`, `staff_repository.rs`, `tag_repository.rs`等の`*_repository.rs`命名パターン。タスクファイル記載の`backend/mediavault-api/src/db/api_credentials.rs`はリポジトリ規約と不一致のため、実装時は`repositories/api_credential_repository.rs`に倣う命名を検討する（既存規約優先）。
- `backend/mediavault-api/src/db/mod.rs`は接続プール生成のみ（`create_pool`）。テーブル別のCRUDロジックはここには置かれていない。
- ハンドラ/ルートに`settings`関連は未実装（`handlers/settings.rs`, `routes/settings.rs`は新規作成）。`routes/mod.rs`の`build_router`に既存パターン（`.route("/path", method(handler))`）で追記する。

### スキーマ・型定義（設計書より）
- `database-schema.sql` L348-353: `api_credentials(provider api_provider PRIMARY KEY, api_key VARCHAR(500) NOT NULL, updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP)`。L375-376で`trg_api_credentials_updated_at`（共通の`update_updated_at_column()`関数）がBEFORE UPDATEで`updated_at`を自動更新するため、UPDATE文のSET句に`updated_at`を明示的に含める必要はない（含めても上書きされる、TASK-0012と同様の方針）。
- `types.rs` L86-94: `ApiProvider` enum（`Tmdb, Igdb, Ndl, Steam, OpenLibrary, AniList`）。L236-240: `ApiCredential { provider: ApiProvider, api_key: String, updated_at: NaiveDateTime }`。L419-421: `UpsertApiCredentialRequest { api_key: String }`（タスク内`UpdateApiKeyRequest`と同義、命名はタスク指示に従う）。
- `api-endpoints.md` L375-388: リクエスト例`{ "api_key": "xxxxx" }`、エラーコード`INVALID_PROVIDER`（400, TC-015-02）。

### エラーコード追加が必要
- `backend/mediavault-api/src/models/response.rs` L50-65の`ApiErrorCode` enumには現時点で`InvalidProvider`相当の値が存在しない（grep確認済み、`INVALID_PROVIDER`は設計書にのみ記載）。本タスクで新規バリアント追加が必須（既存`DuplicateTagName`/`TagNotFound`等と同様に400/404系を追記するパターンを踏襲）。

### 既存upsertパターンの参考
- `ON CONFLICT (provider) DO UPDATE SET api_key = $2, updated_at = CURRENT_TIMESTAMP`形式の`sqlx::query!`はタスク内で新規記述。類似のUPSERT実装が既存リポジトリにあるか確認しつつ、`db_error_utils.rs`（`repositories/db_error_utils.rs`）の共通DBエラー変換ヘルパーを利用し、sqlxエラーをクライアントに直接漏らさず`tracing::error!`＋`InternalError`へ変換する既存方針（TASK-0012ノート記載の`db_error`関数と同型）に従う。

### テスト規約（既存方針を継続）
- インラインテスト（`#[cfg(test)] mod tests`を実装ファイル末尾に配置）。
- 実DB不要なテスト（provider文字列→enum変換、DTOデシリアライズ）は`#[test]`のみ。
- 実DB必要な統合テスト（upsert確認、find_by_provider確認）は`#[tokio::test]` + `#[ignore]`、`DATABASE_URL`環境変数使用、`cargo test -- --ignored`で実行。
- ルーティング統合テストは`routes/mod.rs`の`tests`モジュール内、`test_app_state()`ヘルパー（`AppState { db, internal_api_key }`構築）パターンに合流させる想定。

### 注意事項
- 平文保存が本フェーズの仕様（暗号化は対象外、REQ-015/NFR-202）。
- `api_key`をレスポンスに含めるかはタスク判断に委ねられている（ログ出力時はマスキング検討）。
- 依存: 前提TASK-0004（マイグレーション）・TASK-0007（ルーター骨格）、後続TASK-0023（ExternalSearchServiceがDBからキー取得）。

### プロジェクト全体の補助情報
- `AGENTS.md`、`docs/rule/`ディレクトリは本リポジトリに存在しない（追加ルールなし、確認済み）。

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
