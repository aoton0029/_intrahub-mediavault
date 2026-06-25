# TASK-0023 要件定義書: ExternalSearchServiceラッパー実装（media_type→provider振り分け）

**作成日**: 2026-06-25
**関連タスク**: [TASK-0023](../../tasks/mediavault-backend/TASK-0023.md)
**関連ノート**: [note.md](note.md) TASK-0023セクション
**親要件**: [requirements.md](requirements.md) REQ-002 ・ [acceptance-criteria.md](acceptance-criteria.md) TC-002-01 / TC-002-02 / TC-002-E01 / TC-002-E02
**設計文書**: [architecture.md](../../design/mediavault-backend/architecture.md)「外部APIクライアント」節 ・ [dataflow.md](../../design/mediavault-backend/dataflow.md)「機能1: 外部API検索→アイテム追加」・ [types.rs](../../design/mediavault-backend/types.rs)

**【信頼性レベル凡例】**:
- 🔵 **青信号**: タスク仕様・設計文書・既存コード（note.md記載）から確実な要件
- 🟡 **黄信号**: タスク仕様・設計文書から妥当な推測による要件
- 🔴 **赤信号**: 推測による要件（本ドキュメントには無し）

---

## 1. 機能の概要

`ExternalSearchService` は、利用者が指定する `media_type`（anime/movie/drama/manga/novel/game/academic_book/paper）に応じて、`api-client-lib` が提供する各プロバイダクライアント（jikan/tmdb/ndl/openlibrary/steam/igdb/anilist）の `ApiClient::execute(Request) -> Result<ApiResponse<Model>, ApiError>` を呼び分けるディスパッチサービスである。🔵 *タスク概要 L16-17・architecture.md L45-46より*

- **何をする機能か**: `search(media_type, query)` を受け、media_typeに対応する単一プロバイダのクライアントを選択し、必要に応じてDBからAPIキーを取得して外部APIへ検索リクエストを送り、共通DTO `ExternalSearchResult` の一覧を返す。🔵 *タスク概要・dataflow.md L52-61より*
- **解決する問題**: ハンドラ層・呼び出し元が、プロバイダ固有のクライアント型・認証方式・型の差異を意識せずに「media_typeとクエリ」だけで外部メタデータ検索を行えるようにする（外部API連携の抽象化）。🔵 *architecture.md L27/L46より*
- **想定ユーザー**: 本サービスを呼び出すハンドラ層（後続TASK-0024の `GET /items/search`）。🔵 *dataflow.md L52-54・依存タスク L21より*
- **システム内での位置づけ**: Phase3「外部API連携」の中核サービス。前提TASK-0022の `find_by_provider` でDBキーを取得し、後続TASK-0024のハンドラが本サービスを呼んで `ExternalSearchError` をHTTPステータス（422/502）へマッピングする。🔵 *依存タスク L19-21・note.md L45-46より*
- **対象外**: 実際の外部API実呼び出しを伴う統合テスト（後続TASK-0024で結合確認）、`/items/import` による登録、AniListによるanime補完（下記5.の設計判断参照）。🔵 *統合テスト要件 L100-101・タスク概要より*

**参照したEARS要件**: REQ-002
**参照した設計文書**: architecture.md L41-46（外部APIクライアント）, dataflow.md「機能1」, types.rs L23-32（MediaType）/ L86-93（ApiProvider）

## 2. MediaType → Provider マッピング（設計判断・本タスクの中核）

`types.rs` のメディア別詳細テーブルが各メディアの正規の外部ソースID列（`jikan_id` / `tmdb_id` / `openlibrary_id` / `ndl_id` / `steam_appid` / `igdb_id`）を保持しており、これがマッピングの一次根拠である。本タスクは **1 media_type → 1 provider（単一プロバイダ）** のディスパッチに範囲を限定する。🔵 *types.rs L246-335・完了条件 L24（「正しいプロバイダクライアントが呼び出される」単数）より*

| MediaType | 採用Provider | APIキー | 詳細テーブルの根拠列 | 信頼性 | 判断根拠 |
|---|---|---|---|---|---|
| `Anime` | **Jikan** | 不要 | `anime_details.jikan_id` | 🔵 | タスクL34・TC-002-01・types.rs L255で確定 |
| `Movie` | **TMDb** | 必要 | `movie_details.tmdb_id` | 🔵 | タスクL35・TC-002-02・types.rs L265で確定 |
| `Drama` | **TMDb** | 必要 | `drama_details.tmdb_id` | 🔵 | タスクL35・TC-002-02・types.rs L276で確定 |
| `Manga` | **Jikan** | 不要 | `manga_details.jikan_id` | 🟡 | **設計判断A**（タスクは「manga→OpenLibrary」だが、types.rs L288の正規列は `jikan_id`。詳細テーブルを正とする） |
| `Novel` | **OpenLibrary** | 必要 | `novel_details.openlibrary_id` | 🔵 | タスクL37・types.rs L299で確定 |
| `Game` | **IGDB** | 必要 | `game_details.igdb_id`（`steam_appid`も保持） | 🟡 | **設計判断B**（Steam/IGDB両対応列が存在。IGDBを唯一のプライマリに固定） |
| `AcademicBook` | **NDL** | 必要 | `academic_book_details.ndl_id` | 🔵 | タスクL38（paper/book→NDL）・types.rs L321で確定 |
| `Paper` | **NDL** | 必要 | `paper_details.ndl_id` | 🔵 | タスクL38・types.rs L334で確定 |

### 設計判断A: Manga → Jikan（OpenLibraryではない）

タスクファイル L37 は「manga / novel → OpenLibrary」と一括りにしているが、`types.rs` L280-289 の `manga_details` が保持する正規の外部ID列は `jikan_id` のみであり、`openlibrary_id` は持たない（一方 `novel_details` L292-301 は `openlibrary_id` を保持）。Jikan(MyAnimeList)は漫画の書誌・話数・掲載誌情報を網羅しており、かつ**キー不要**でDB初期化前から検索可能であるため、manga は **Jikan** を採用する。novel のみ OpenLibrary とする。🟡 *types.rs L288/L299の詳細テーブル定義からの妥当な推測（タスク本文の一括記載より、データモデルの個別列定義を優先）*

### 設計判断B: Game → IGDB をプライマリに固定（Steamは本タスク対象外）

`game_details`（types.rs L304-312）は `steam_appid` と `igdb_id` の双方を保持し、media_type単独ではどちらを使うか一意に決まらない（タスクL36・note.md L39で🟡明示の真の曖昧点）。本タスクでは **IGDB を唯一のプライマリプロバイダ**として固定する。

- **採用理由**: IGDBは「ゲームタイトルの汎用メタデータ検索DB」であり、任意のクエリ文字列でのタイトル検索という本機能の用途に合致する。一方 Steam Web API はストア/ライブラリ（app ownership・storefront）コンテキストに依存し、汎用的なタイトル名検索のプライマリ手段として不適である（Steamの主用途はTASK-0017のSteamライブラリインポート＝steam_id起点であり、検索とは別系統）。🟡 *note.md L39（2案: IGDB優先/Steam fallback or クエリ指定）からの設計判断。Steamの用途はdataflow.md「機能5」のライブラリインポートに分離されている点を根拠とする*
- **本タスクでの結論**: `MediaType::Game` は常にIGDBへディスパッチする。Steamへの切替やfallbackは本タスクの単一プロバイダ方針の対象外とし、将来クエリパラメータ（例 `?provider=steam`）による明示指定の拡張余地を実装コメントとして残す。🟡

### 設計判断C: Anime は Jikan のみ（AniList併用は対象外）

完了条件 L24・テストケース1（L75-79）は「media_type=anime で **Jikanのみ** が呼ばれ、他プロバイダは呼ばれない」を明示している。よって本タスクでは anime → Jikan の単一ディスパッチとし、AniListによる関連情報拡張（タスクL39の🟡項目）は **本タスクのスコープ外（将来拡張）** とする。`ApiProvider::AniList` はマッピング表に登場しない。実装には将来のenrichment追加余地をコメントで残す。🔵 *完了条件 L24・テストケース1 L75-79・note.md L41より*

**参照した設計文書**: types.rs L246-335（メディア別詳細テーブル）, タスクL31-39（マッピング定義）, note.md L38-42（未確定点）

## 3. 入力・出力の仕様（API契約）

### ExternalSearchService API契約 🔵 *タスク実装詳細2 L44-55より*

```rust
pub struct ExternalSearchService {
    pool: PgPool,
}

impl ExternalSearchService {
    /// DI: 接続プールを受け取り初期化
    pub fn new(pool: PgPool) -> Self;

    /// media_typeに対応する単一プロバイダへ検索リクエストをディスパッチする。
    pub async fn search(
        &self,
        media_type: MediaType,
        query: &str,
    ) -> Result<Vec<ExternalSearchResult>, ExternalSearchError>;
}
```

### 入力 🔵 *types.rs L368-373（ExternalSearchQuery）・タスク L52-53より*

- `media_type: MediaType`: 8 variant（`Anime`/`Movie`/`Drama`/`Manga`/`Novel`/`Game`/`AcademicBook`/`Paper`）。第2章の表に従い必ず1プロバイダへ写像される。
- `query: &str`: 検索クエリ文字列（タイトル等）。本タスクでは空文字バリデーションは要件未指定のため呼び出し元責務とし、サービス層では透過的に各プロバイダのRequestへ渡す。🟡 *タスクにバリデーション言及なしからの妥当な推測*

### 出力（共通DTO ExternalSearchResult） 🟡 *タスク実装詳細5 L68-71・api-endpoints.md「外部API検索結果一覧」より、ラップ形式は実装詳細未確定のため妥当な推測*

プロバイダ固有のModel型を、各プロバイダ用アダプタ関数で以下の共通DTOへ変換する。`backend/mediavault-api/src/models/external_search.rs`（新規）に定義する。

```rust
#[derive(Debug, Clone, Serialize)]
pub struct ExternalSearchResult {
    pub media_type: MediaType,
    pub provider: ApiProvider,        // 採用したプロバイダ（Jikanは下記注記参照）
    pub external_id: String,          // プロバイダ固有ID（jikan_id/tmdb_id等の元値）
    pub title: String,
    pub raw_data: serde_json::Value,  // プロバイダ固有の生データ（ApiResponse.raw 由来）
}
```

- **注記**: `provider` フィールドの表現について、`ApiProvider` enum（types.rs L86-93）には **Jikan が含まれない**（キー不要のため）。`Anime`/`Manga`（→Jikan）の結果で `provider` をどう表すかは2案あり実装判断とする: (a) `ApiProvider` を `Option` 化し Jikan時は `None`、(b) DTO専用の別enum（Jikanを含む）を定義。🟡 *types.rs L86-93（ApiProvider に Jikan 無し）と note.md L36 からの設計上の論点。tdd-red着手前に確定すること*

### データフロー 🔵 *dataflow.md L44-61より*

`handlers（TASK-0024）` → `ExternalSearchService::search(media_type, query)` → 第2章でprovider決定 → （キー必要なら）`repositories::api_credential_repository::find_by_provider(pool, provider)` → クライアント初期化（`AuthStrategy` 注入）→ `client.execute(Request)` → `ApiResponse<Model>` → アダプタで `ExternalSearchResult` へ変換 → `Vec<ExternalSearchResult>` を返す。

**参照したEARS要件**: REQ-002
**参照した設計文書**: types.rs L368-373（ExternalSearchQuery）, dataflow.md「機能1」, api-endpoints.md（外部API検索結果一覧）

## 4. ExternalSearchError 仕様

`ExternalSearchError` を新規定義する（`backend/mediavault-api/src/services/external_search.rs` 内、または `models/external_search.rs`）。🔵 *note.md L44-46・タスク完了条件 L25-26より*

```rust
#[derive(Debug)]
pub enum ExternalSearchError {
    /// キー必須プロバイダでDBにキー未登録（後続TASK-0024で422へマッピング）
    ApiKeyNotConfigured(ApiProvider),
    /// api-client-lib の ApiError をラップ（後続TASK-0024で502へマッピング）
    ExternalApiError(api_client_lib::ApiError),
}
```

| variant | 発生条件 | 後続マッピング（TASK-0024範囲） | 信頼性 |
|---|---|---|---|
| `ApiKeyNotConfigured(ApiProvider)` | キー必須プロバイダで `find_by_provider` が `None` を返した | 422（`ApiKeyNotConfigured` 用コードは要新規追加・後続範囲） | 🔵 *タスク L25/L60より* |
| `ExternalApiError(ApiError)` | `client.execute` が `ApiError`（`Timeout`/`Http`/`Auth`/`RateLimit`/`Parse`/`Network`）を返した | 502（既存 `ApiErrorCode::ExternalApiError`=EXTERNAL_API_ERROR を流用） | 🔵 *タスク L26/L63-65・note.md L24/L46より* |

- `api-client-lib` の `ApiError` 6 variant（`Http{status,body}` / `Auth` / `RateLimit{retry_after}` / `Parse` / `Timeout` / `Network`）はすべて `ExternalApiError` へ集約する。🔵 *note.md L24より*
- `?` 演算子で `Result` を呼び出し元へ伝播させ、**panicさせない**こと。🔵 *タスク L26/L65より*

**参照した設計文書**: note.md L24（ApiError）/ L44-46（ExternalSearchError）, response.rs L56（既存ExternalApiError→502）

## 5. 機能要件（EARS記法）

### 通常要件

- REQ-0023-01: システムは `search(media_type, query)` を受理し、第2章の表に従って media_type を一意の単一プロバイダへ写像しなければならない。🔵 *完了条件 L24・タスク L31-39より*
- REQ-0023-02: `media_type=Anime` の場合、システムは Jikan クライアントの `execute` のみを呼び出し、他プロバイダのクライアントを呼び出してはならない。🔵 *完了条件 L24・TC-002-01・テストケース1 L75-79より*
- REQ-0023-03: `media_type=Movie` または `Drama` の場合、システムはDBから取得したキーで初期化したTMDbクライアントの `execute` を呼び出さなければならない。🔵 *TC-002-02・テストケース2 L81-85より*
- REQ-0023-04: `media_type=Manga` の場合、システムは（キー不要の）Jikanクライアントへディスパッチしなければならない（設計判断A）。🟡 *types.rs L288より妥当な推測*
- REQ-0023-05: `media_type=Novel` の場合は OpenLibrary、`Game` の場合は IGDB（設計判断B）、`AcademicBook`/`Paper` の場合は NDL クライアントへディスパッチしなければならない。🔵🟡 *タスク L37-38・types.rs L299/L311/L321/L334より（GameのみIGDB固定が🟡）*
- REQ-0023-06: システムは成功時、プロバイダ固有Modelをアダプタで `ExternalSearchResult` へ変換し `Vec<ExternalSearchResult>` を返さなければならない。🟡 *タスク実装詳細5 L68-71より妥当な推測*

### 条件付き要件

- REQ-0023-101: キー必須プロバイダ（TMDb/IGDB/NDL/Steam/OpenLibrary/AniList）について、`find_by_provider` が `None` を返した場合、システムは `ExternalSearchError::ApiKeyNotConfigured(provider)` を返し、外部API呼び出しを一切行ってはならない。🔵 *完了条件 L25・タスク L58-60・テストケース3 L87-91より*
- REQ-0023-102: Jikan（anime/manga）の場合、システムはキー取得（`find_by_provider`）をスキップしなければならない。🔵 *タスク L60より*
- REQ-0023-103: `client.execute` が `ApiError` を返した場合、システムは `ExternalSearchError::ExternalApiError(..)` を返し、panicしてはならない。🔵 *完了条件 L26・テストケース4 L93-97より*

### 制約要件

- REQ-0023-401: システムは `api-client-lib` の `ApiClient` トレイトのインターフェースを変更してはならない。🔵 *タスク注意事項 L113・architecture.md L161より*
- REQ-0023-402: `ApiClient::execute` は `impl Future` を返すRPITIT形式で **dyn非互換**のため、システムは動的ディスパッチに `dyn ApiClient` を用いず、enum/matchによる各プロバイダ型の直接構築（または静的ジェネリクス）で実装しなければならない。🔵 *note.md L23より*
- REQ-0023-403: システムはキー必須プロバイダのDBキー取得に既存 `repositories::api_credential_repository::find_by_provider`（TASK-0022実装）を利用しなければならない。🔵 *タスク L58-60・note.md L36より*
- REQ-0023-404: `services/mod.rs` に `pub mod external_search;` を追記し、サービスを公開しなければならない。🔵 *note.md L47より*

### 単一プロバイダ・スコープ制約

- REQ-0023-501: 本タスクは 1 media_type → 1 provider のディスパッチに限定し、AniListによるanime補完・Steamへのgame切替・複数プロバイダ併用は実装しない（将来拡張としてコメントを残す）。🔵 *完了条件 L24（単数）・note.md L41-42・設計判断B/Cより*

## 6. 非機能要件

- NFR-0023-01: 本サービスは独自のレスポンス形式を新設せず、エラーは `ExternalSearchError` として返し、HTTPステータス変換はハンドラ層（TASK-0024）に委ねなければならない。🔵 *タスク L17/L26・note.md L46より*
- NFR-0023-02: DBキー取得失敗（接続不能・SQLエラー）は `find_by_provider` 既存の `ApiError` 変換方針（内部情報非漏洩・`tracing::error!`）を踏襲しなければならない。🟡 *note.md TASK-0022 L83の `db_error` 方針からの妥当な推測*
- NFR-0023-03: APIキーをログ出力する場合はマスキングを検討する。🟡 *TASK-0022 注意事項 L92踏襲の妥当な推測*
- NFR-0023-04: テストはDB非依存のディスパッチロジック単体テストを主とし、実DB必要なキー取得確認は `#[tokio::test]` + `#[ignore]`（`DATABASE_URL`）で分離する。🔵 *note.md L52-55より*

## 7. テスト設計上の制約（mockall課題） 🟡

`ApiClient::execute` がRPITIT（dyn非互換）のため、`mockall::automock` を素のトレイトに直接適用できない可能性がある。テストケース1・2の「executeのみが呼ばれる」呼び出し検証を満たす手段として、以下のいずれかを設計判断する（tdd-red着手前に確定）: 🟡 *note.md L29-32より妥当な推測*

- (a) 各クライアントの `new_with_base_url` でテスト用モックサーバー（`wiremock` 等）のURLを注入し、HTTPレベルでリクエスト到達と非到達を検証する。
- (b) `mockall` を `mediavault-api/Cargo.toml` の `[dev-dependencies]` に新規追加し、RPITIT対応可否を確認のうえトレイトをモック化する。

技術的にトレイトモック化が困難な場合、(a) のHTTPモックサーバーへの「対象プロバイダURLにのみリクエストが到達し、他には到達しない」検証へ置き換える。🟡 *note.md L32より*

## 8. 想定される使用例（Given/When/Then・Edgeケース）

### シナリオ1: anime → Jikanのみ呼び出し（TC-002-01） 🔵
- **Given**: `media_type=Anime`、各プロバイダクライアントがモック化されている
- **When**: `search(MediaType::Anime, "鬼滅の刃")` を呼ぶ
- **Then**: Jikanクライアントの `execute` のみが呼ばれ、他プロバイダは呼ばれない。キー取得（`find_by_provider`）も発生しない

### シナリオ2: movie/drama → DBキーで初期化したTMDb呼び出し（TC-002-02） 🔵
- **Given**: `media_type=Movie`（または `Drama`）、`api_credentials` にTMDbキーが登録済み
- **When**: `search(MediaType::Movie, "タイトル")` を呼ぶ
- **Then**: `find_by_provider(Tmdb)` で取得したキーで初期化されたTMDbクライアントの `execute` が呼ばれる

### シナリオ3: キー未設定で ApiKeyNotConfigured（TC-002-E01） 🟡
- **Given**: `media_type=Movie`、`api_credentials` にTMDbキーが存在しない
- **When**: `search(MediaType::Movie, "タイトル")` を呼ぶ
- **Then**: `Err(ExternalSearchError::ApiKeyNotConfigured(ApiProvider::Tmdb))` が返り、外部API呼び出しは発生しない

### シナリオ4: 外部APIタイムアウトで ExternalApiError（TC-002-E02） 🟡
- **Given**: モックTMDbクライアントが `ApiError::Timeout` を返すよう設定
- **When**: `search(MediaType::Movie, "タイトル")` を呼ぶ
- **Then**: `Err(ExternalSearchError::ExternalApiError(..))` が返り、panicしない

### 追加Edgeケース
- EDGE-0023-01: `media_type=Manga` → Jikanへディスパッチ（設計判断A）。キー取得スキップ。🟡 *types.rs L288より*
- EDGE-0023-02: `media_type=Game` → IGDBへディスパッチ（設計判断B）。Steamには到達しない。🟡 *note.md L39・設計判断Bより*
- EDGE-0023-03: `media_type=AcademicBook`/`Paper` → NDLへディスパッチ。🔵 *types.rs L321/L334より*
- EDGE-0023-04: `ApiError::Http{status}`/`Auth`/`RateLimit`/`Parse`/`Network` のいずれも `ExternalApiError` へ集約され、panicしない。🔵 *note.md L24より*

**参照したEARS要件**: REQ-002, TC-002-01 / TC-002-02 / TC-002-E01 / TC-002-E02
**参照した設計文書**: dataflow.md「機能1」, types.rs（詳細テーブル）

## 9. 完了基準

- [ ] `ExternalSearchService::search(media_type, query)` が実装され、第2章の表どおりに正しい単一プロバイダクライアントが呼び出される。🔵 *完了条件 L24*
- [ ] キー必須プロバイダでキー未設定時に `ExternalSearchError::ApiKeyNotConfigured(provider)` を返し、外部API呼び出しが発生しない。🔵 *完了条件 L25*
- [ ] `client.execute` のタイムアウト/エラー時に `ExternalSearchError::ExternalApiError` を返し、panicしない。🔵 *完了条件 L26*
- [ ] media_type→provider ディスパッチを検証する単体テスト（モックまたはHTTPモックサーバー）がすべて成功する。🔵 *完了条件 L27*
- [ ] anime/mangaはJikan（キー取得スキップ）、game はIGDB、academic_book/paper はNDL、novel はOpenLibrary、movie/drama はTMDbへ写像される（設計判断A/B/C反映）。🟡
- [ ] `services/mod.rs` に `pub mod external_search;` が追記され、`models/external_search.rs`（または同等箇所）に `ExternalSearchResult` / `ExternalSearchError` が定義されている。🔵 *note.md L47/L50*
- [ ] `ApiClient` トレイトのインターフェースは変更されていない。🔵 *タスク注意事項 L113*

## 10. EARS要件・設計文書との対応関係

- **参照した機能要件**: REQ-002
- **参照した非機能要件**: （本サービス層に直接対応するNFRは無し。エラー変換/ログ方針はTASK-0022踏襲）
- **参照したEdgeケース**: TC-002-E01 / TC-002-E02（本ドキュメントで EDGE-0023-01〜04 を新設）
- **参照した受け入れ基準**: TC-002-01 / TC-002-02 / TC-002-E01 / TC-002-E02
- **参照した設計文書**:
  - **アーキテクチャ**: architecture.md L41-46（外部APIクライアント・ExternalSearchService新設方針）/ L158-161（互換性制約）
  - **データフロー**: dataflow.md「機能1: 外部API検索→アイテム追加」L44-76
  - **型定義**: types.rs L23-32（MediaType）/ L86-93（ApiProvider）/ L246-335（メディア別詳細テーブル＝マッピング一次根拠）/ L368-373（ExternalSearchQuery）
  - **既存コード現況**: note.md TASK-0023セクション（api-client-lib API surface・RPITIT/dyn非互換・mockall未導入・MediaType/ApiProvider実定義）

## 11. 信頼性レベルサマリー

| カテゴリ | 🔵 | 🟡 | 🔴 | 合計 |
|---|---|---|---|---|
| マッピング表（8 media_type） | 6 | 2 | 0 | 8 |
| 機能要件（通常） | 4 | 2 | 0 | 6 |
| 機能要件（条件付き） | 3 | 0 | 0 | 3 |
| 機能要件（制約） | 4 | 0 | 0 | 4 |
| スコープ制約 | 1 | 0 | 0 | 1 |
| 非機能要件 | 1 | 3 | 0 | 4 |
| Edgeケース/シナリオ | 4 | 4 | 0 | 8 |

**全体評価**: 高品質（赤信号なし）。黄信号は (1) 設計判断A: manga→Jikan、(2) 設計判断B: game→IGDB固定、(3) `ExternalSearchResult`/`provider`(Jikan)のラップ形式、(4) mockall/HTTPモックのテスト手段 に集中。いずれも本ドキュメントで設計判断を明文化済み、または tdd-red着手前の確定事項として下記に引き継ぐ。

---

## 次フェーズへの引き渡し事項

- `tdd-testcases` フェーズでは、シナリオ1〜4を中核とし、EDGE-0023-01（manga→Jikan）・EDGE-0023-02（game→IGDB、Steam非到達）・EDGE-0023-03（academic_book/paper→NDL）・EDGE-0023-04（全ApiError集約）を追加洗い出しすること。
- `tdd-red` 着手前に以下を確定すること:
  1. **テスト手段**: `mockall`（RPITIT対応可否）か `wiremock`等のHTTPモックサーバー（`new_with_base_url`注入）か（第7章）。困難なら呼び出し検証→URL到達検証へ置換。
  2. **ディスパッチ実装方式**: enum/matchによる各プロバイダ型の直接構築（dyn非互換のため `dyn ApiClient` 不可、REQ-0023-402）。
  3. **`ExternalSearchResult.provider` の Jikan表現**: `Option<ApiProvider>` か DTO専用enか（第3章注記）。
  4. **設計判断A/Bのコミットログ記載**: manga→Jikan・game→IGDB固定の根拠を実装コミットメッセージまたはnote.md追記として残すこと（タスク注意事項 L114）。
- 依存追加が必要な場合（`mockall`/`wiremock`）は `mediavault-api/Cargo.toml` の `[dev-dependencies]` を更新する（note.md L31）。
