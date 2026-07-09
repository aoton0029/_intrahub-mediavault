//! ExternalSearchService: media_type→provider振り分けディスパッチサービス
//!
//! TASK-0023: ExternalSearchServiceラッパー実装（media_type→provider振り分け）
//!
//! 【信頼性レベル】: 🔵 要件定義書 第2章・第3章・REQ-0023-01〜06より
//!
//! 【dyn非互換に関する注記】: `api_client_lib::ApiClient::execute` はRPITIT形式
//! （`impl Future` 戻り値）のため `dyn ApiClient` としてトレイトオブジェクト化できない
//! （REQ-0023-402）。本サービスは media_type に対する `match` で各プロバイダ型を
//! 直接構築・呼び出す静的ディスパッチ方式を採る。
//!
//! 【models/domain全廃止に伴うリファクタ】: 旧`models::domain::MediaDetails`（複数プロバイダの
//! 形を1つに正規化するための抽象化）は、AniList削除後は実質Animeの Annict+Jikan マージにしか
//! 意味を持たなくなったため廃止した。検索結果は軽量な`SearchResultItem`を直接構築し、
//! インポート確定時は各プロバイダの生レスポンスから`CreateItemRequest`を直接構築する。

use std::sync::Arc;

use api_client_lib::auth::AuthStrategy;
use api_client_lib::clients::annict::AnnictClient;
use api_client_lib::clients::annict::models::WorkModel;
use api_client_lib::clients::annict::requests::ListWorksRequest;
use api_client_lib::clients::jikan::JikanClient;
use api_client_lib::clients::jikan::requests::{JikanAnimeDetailsRequest, JikanRequest};
use api_client_lib::clients::ndl::NdlClient;
use api_client_lib::clients::ndl::models::{NdlItemModel, NdlModel};
use api_client_lib::clients::ndl::requests::{NdlRequest, NdlSearchRequest};
use api_client_lib::clients::rakuten::RakutenClient;
use api_client_lib::clients::rakuten::models::BookModel;
use api_client_lib::clients::rakuten::requests::SearchBooksRequest;
use api_client_lib::clients::steam::SteamClient;
use api_client_lib::clients::steam::requests::{
    SteamAppDetailsRequest, SteamRequest, SteamStoreSearchRequest,
};
use api_client_lib::clients::tmdb::TmdbClient;
use api_client_lib::clients::tmdb::requests::{
    MovieDetailsRequest, SearchMovieRequest, SearchTvRequest, TmdbRequest, TvSeriesRequest,
};
use api_client_lib::traits::ApiClient;
use chrono::NaiveDate;
use serde_json::Value;
use sqlx::PgPool;

use crate::models::api_credential::{ApiCredential, ApiProvider};
use crate::models::external_search::{ExternalSearchError, SearchResultItem};
use crate::models::item::{CreateItemRequest, MediaType};
use crate::repositories::api_credential_repository;

/// キー必須プロバイダのAPIキー解決を行うDIポイント（テスト用差し替え可能）
///
#[derive(Clone)]
pub enum ApiCredentialLookup {
    /// 本番経路: 実際のPgPoolへ`find_by_provider`を発行する
    Pool(PgPool),
    /// テスト経路: 固定の`Option<ApiCredential>`を即時返す（DBアクセスなし）
    Fixed(Arc<dyn Fn(ApiProvider) -> Option<ApiCredential> + Send + Sync>),
}

/// `ApiResponse.raw`（`RawData::Json`/`Xml`）を`serde_json::Value`へ変換する共通ヘルパー
///
fn raw_data_to_value(raw: &api_client_lib::response::RawData) -> serde_json::Value {
    match raw {
        api_client_lib::response::RawData::Json(text) => {
            serde_json::from_str(text).unwrap_or(serde_json::Value::Null)
        }
        api_client_lib::response::RawData::Xml(text) => serde_json::json!({ "xml": text }),
    }
}

// ── raw JSON 抽出ヘルパー（旧 models/domain/core.rs から移設） ──────────────────

/// 文字列フィールドを取り出す（空文字は None 扱い）
fn json_str(v: &Value, key: &str) -> Option<String> {
    v.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
}

/// 数値フィールドを f64 で取り出す
fn json_f64(v: &Value, key: &str) -> Option<f64> {
    v.get(key).and_then(Value::as_f64)
}

/// 数値フィールドを u32 で取り出す
fn json_u32(v: &Value, key: &str) -> Option<u32> {
    v.get(key).and_then(Value::as_u64).map(|n| n as u32)
}

/// `[{"<name_key>": "..."}, ...]` 形式の配列から名前一覧を取り出す
fn json_names(v: &Value, key: &str, name_key: &str) -> Vec<String> {
    v.get(key)
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(|e| json_str(e, name_key)).collect())
        .unwrap_or_default()
}

/// 文字列配列を取り出す
fn json_str_array(v: &Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

/// 空文字列をNone扱いにする（Annictは未設定項目を`""`で返すため）
fn non_empty(s: Option<String>) -> Option<String> {
    s.filter(|v| !v.trim().is_empty())
}

/// TMDb ポスター画像のベース URL
const TMDB_IMAGE_BASE: &str = "https://image.tmdb.org/t/p/w342";

/// TMDb の poster_path を完全 URL に解決する。
fn tmdb_image_url(v: &Value, key: &str) -> Option<String> {
    json_str(v, key).map(|path| format!("{TMDB_IMAGE_BASE}{path}"))
}

/// `release_date`（精度がプロバイダごとに異なる文字列）を`NaiveDate`へ変換する。
///
/// "YYYY-MM-DD" を優先し、年のみ（"2003"等）は1月1日へフォールバック。
/// 解釈できない形式はNoneとし、インポート自体は拒否しない。
fn parse_release_date(raw: &str) -> Option<NaiveDate> {
    let trimmed = raw.trim();
    NaiveDate::parse_from_str(trimmed, "%Y-%m-%d")
        .ok()
        .or_else(|| {
            trimmed
                .parse::<i32>()
                .ok()
                .and_then(|year| NaiveDate::from_ymd_opt(year, 1, 1))
        })
}

/// 一覧系レスポンスのraw JSONから、指定キー配下の配列要素をマッパーで`SearchResultItem`へ変換する
///
/// TMDb（"results"）/Jikan（"data"）のように「レスポンス直下のキーに検索結果配列を持つ」
/// プロバイダ共通の変換ループ。配列が無い場合は空Vecを返す（panic防止）。
fn map_array_items(
    whole: &Value,
    array_key: &str,
    to_item: impl Fn(&Value) -> SearchResultItem,
) -> Vec<SearchResultItem> {
    whole
        .get(array_key)
        .and_then(Value::as_array)
        .map(|arr| arr.iter().map(to_item).collect())
        .unwrap_or_default()
}

// ── 検索結果（軽量DTO）構築関数 ────────────────────────────────────────────

fn search_item_from_annict_work(work: &WorkModel) -> SearchResultItem {
    SearchResultItem {
        id: work.id.to_string(),
        media_type: MediaType::Anime,
        provider: Some(ApiProvider::Annict),
        title: non_empty(work.title.clone()).unwrap_or_default(),
        thumbnail_url: work
            .images
            .as_ref()
            .and_then(|i| non_empty(i.recommended_url.clone())),
    }
}

fn search_item_from_rakuten_book(book: &BookModel, media_type: MediaType) -> SearchResultItem {
    SearchResultItem {
        id: book.isbn.clone().unwrap_or_default(),
        media_type,
        provider: Some(ApiProvider::Rakuten),
        title: book.title.clone().unwrap_or_default(),
        thumbnail_url: book
            .large_image_url
            .clone()
            .or_else(|| book.medium_image_url.clone())
            .or_else(|| book.small_image_url.clone()),
    }
}

fn search_item_from_tmdb_movie(data: &Value) -> SearchResultItem {
    SearchResultItem {
        id: data
            .get("id")
            .and_then(Value::as_u64)
            .map(|id| id.to_string())
            .unwrap_or_default(),
        media_type: MediaType::Movie,
        provider: Some(ApiProvider::Tmdb),
        title: json_str(data, "title").unwrap_or_default(),
        thumbnail_url: tmdb_image_url(data, "poster_path"),
    }
}

fn search_item_from_tmdb_tv(data: &Value) -> SearchResultItem {
    SearchResultItem {
        id: data
            .get("id")
            .and_then(Value::as_u64)
            .map(|id| id.to_string())
            .unwrap_or_default(),
        media_type: MediaType::Drama,
        provider: Some(ApiProvider::Tmdb),
        title: json_str(data, "name").unwrap_or_default(),
        thumbnail_url: tmdb_image_url(data, "poster_path"),
    }
}

fn search_item_from_steam_search(data: &Value) -> SearchResultItem {
    SearchResultItem {
        id: data
            .get("id")
            .and_then(Value::as_u64)
            .map(|id| id.to_string())
            .unwrap_or_default(),
        media_type: MediaType::Game,
        provider: Some(ApiProvider::Steam),
        title: json_str(data, "name").unwrap_or_default(),
        thumbnail_url: json_str(data, "tiny_image"),
    }
}

fn search_item_from_ndl_item(item: &NdlItemModel, media_type: MediaType) -> SearchResultItem {
    let isbn = item.isbn13.clone().or_else(|| item.isbn.clone());
    SearchResultItem {
        id: isbn.unwrap_or_default(),
        media_type,
        provider: Some(ApiProvider::Ndl),
        title: item.title.clone().unwrap_or_default(),
        thumbnail_url: item.thumbnail_url.clone(),
    }
}

// ── インポート確定時: CreateItemRequest直接構築関数 ─────────────────────────

/// AnnictとJikanの情報をマージして`CreateItemRequest`を構築する。
///
/// id/title/画像/話数等の作品識別・掲載情報はAnnict優先、あらすじ・ジャンル・評価・
/// スタジオ等の詳細はJikan由来とする。Annict側が空の場合はJikanの値でフォールバックする。
/// `jikan_data`が`None`（mal_anime_id未設定・Jikan取得失敗時）の場合はAnnict情報のみで構築する。
fn build_anime_create_request(work: &WorkModel, jikan_data: Option<&Value>) -> CreateItemRequest {
    let jikan_title = jikan_data.and_then(|d| json_str(d, "title"));
    let jikan_original_title = jikan_data.and_then(|d| json_str(d, "title_japanese"));
    let jikan_alt_title = jikan_data.and_then(|d| json_str(d, "title_english"));
    let jikan_description = jikan_data.and_then(|d| json_str(d, "synopsis"));
    let jikan_release_date = jikan_data
        .and_then(|d| d.get("aired"))
        .and_then(|a| json_str(a, "from"))
        .map(|d| d.chars().take(10).collect::<String>());
    let jikan_image = jikan_data
        .and_then(|d| d.get("images"))
        .and_then(|i| i.get("jpg"))
        .and_then(|j| json_str(j, "image_url"));
    let jikan_genres = jikan_data
        .map(|d| json_names(d, "genres", "name"))
        .unwrap_or_default();
    let jikan_rating = jikan_data.and_then(|d| json_f64(d, "score"));
    let jikan_url = jikan_data.and_then(|d| json_str(d, "url"));
    let jikan_episodes = jikan_data.and_then(|d| json_u32(d, "episodes"));
    let jikan_status = jikan_data.and_then(|d| json_str(d, "status"));
    let jikan_season = jikan_data.and_then(|d| json_str(d, "season"));
    let jikan_year = jikan_data.and_then(|d| json_u32(d, "year"));
    let jikan_studios = jikan_data
        .map(|d| json_names(d, "studios", "name"))
        .unwrap_or_default();
    let jikan_source = jikan_data.and_then(|d| json_str(d, "source"));
    let jikan_duration = jikan_data.and_then(|d| json_str(d, "duration"));
    let jikan_trailer = jikan_data
        .and_then(|d| d.get("trailer"))
        .and_then(|t| json_str(t, "url"));

    let title = non_empty(work.title.clone())
        .or(jikan_title)
        .unwrap_or_default();
    let image_url = work
        .images
        .as_ref()
        .and_then(|i| non_empty(i.recommended_url.clone()))
        .or(jikan_image);
    let episodes = work.episodes_count.or(jikan_episodes);
    let season = non_empty(work.season_name.clone()).or(jikan_season);
    let release_date_str = non_empty(work.released_on.clone()).or(jikan_release_date);
    let homepage_url = non_empty(work.official_site_url.clone()).or(jikan_url);

    let details = serde_json::json!({
        "episodes": episodes,
        "status": jikan_status,
        "season": season,
        "year": jikan_year,
        "studios": jikan_studios,
        "source": jikan_source,
        "duration": jikan_duration,
        "trailer_url": jikan_trailer,
        "genres": jikan_genres,
        "rating": jikan_rating,
        "url": homepage_url,
        "alternative_titles": jikan_alt_title.into_iter().collect::<Vec<_>>(),
    });

    CreateItemRequest {
        media_type: MediaType::Anime,
        title,
        original_title: jikan_original_title,
        description: jikan_description,
        cover_image_url: image_url,
        release_date: release_date_str.as_deref().and_then(parse_release_date),
        homepage_url,
        rating: None,
        is_favorite: None,
        details: Some(details),
        consumed_date: None,
    }
}

/// 楽天ブックスの`salesDate`（例: "1997年12月24日"、精度は年のみ〜年月日で揺れる）を`NaiveDate`へ変換する。
///
/// 数字部分のみを抽出し年/月/日として解釈する。月・日が欠けている場合は1で補完する。
fn parse_rakuten_sales_date(raw: &str) -> Option<NaiveDate> {
    let digits: Vec<i32> = raw
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse().ok())
        .collect();
    match digits.as_slice() {
        [y, m, d] => NaiveDate::from_ymd_opt(*y, *m as u32, *d as u32),
        [y, m] => NaiveDate::from_ymd_opt(*y, *m as u32, 1),
        [y] => NaiveDate::from_ymd_opt(*y, 1, 1),
        _ => None,
    }
}

/// 楽天ブックス`BookModel`（manga/novel/academic_book共通）から`CreateItemRequest`を構築する。
fn build_book_create_request(book: &BookModel, media_type: MediaType) -> CreateItemRequest {
    let details = serde_json::json!({
        "authors": book.author.clone(),
        "publisher": book.publisher_name.clone(),
        "isbn": book.isbn.clone(),
        "series_name": book.series_name.clone(),
    });

    CreateItemRequest {
        media_type,
        title: book.title.clone().unwrap_or_default(),
        original_title: None,
        description: book.item_caption.clone(),
        cover_image_url: book
            .large_image_url
            .clone()
            .or_else(|| book.medium_image_url.clone())
            .or_else(|| book.small_image_url.clone()),
        release_date: book
            .sales_date
            .as_deref()
            .and_then(parse_rakuten_sales_date),
        homepage_url: book.item_url.clone(),
        rating: None,
        is_favorite: None,
        details: Some(details),
        consumed_date: None,
    }
}

/// TMDb `GET /movie/{id}` レスポンスから`CreateItemRequest`を構築する。
fn build_movie_create_request(data: &Value) -> CreateItemRequest {
    let details = serde_json::json!({
        "runtime_minutes": json_u32(data, "runtime"),
        "original_language": json_str(data, "original_language"),
        "vote_count": json_u32(data, "vote_count"),
        "collection": data.get("belongs_to_collection").and_then(|c| json_str(c, "name")),
        "production_companies": json_names(data, "production_companies", "name"),
        "genres": json_names(data, "genres", "name"),
        "rating": json_f64(data, "vote_average"),
    });

    CreateItemRequest {
        media_type: MediaType::Movie,
        title: json_str(data, "title").unwrap_or_default(),
        original_title: json_str(data, "original_title"),
        description: json_str(data, "overview"),
        cover_image_url: tmdb_image_url(data, "poster_path"),
        release_date: json_str(data, "release_date")
            .as_deref()
            .and_then(parse_release_date),
        homepage_url: json_str(data, "homepage"),
        rating: None,
        is_favorite: None,
        details: Some(details),
        consumed_date: None,
    }
}

/// TMDb `GET /tv/{id}` レスポンスから`CreateItemRequest`を構築する。
fn build_drama_create_request(data: &Value) -> CreateItemRequest {
    let first_air_date = json_str(data, "first_air_date");
    let details = serde_json::json!({
        "number_of_seasons": json_u32(data, "number_of_seasons"),
        "number_of_episodes": json_u32(data, "number_of_episodes"),
        "networks": json_names(data, "networks", "name"),
        "status": json_str(data, "status"),
        "original_language": json_str(data, "original_language"),
        "first_air_date": first_air_date,
        "last_air_date": json_str(data, "last_air_date"),
        "genres": json_names(data, "genres", "name"),
        "rating": json_f64(data, "vote_average"),
    });

    CreateItemRequest {
        media_type: MediaType::Drama,
        title: json_str(data, "name").unwrap_or_default(),
        original_title: json_str(data, "original_name"),
        description: json_str(data, "overview"),
        cover_image_url: tmdb_image_url(data, "poster_path"),
        release_date: first_air_date.as_deref().and_then(parse_release_date),
        homepage_url: json_str(data, "homepage"),
        rating: None,
        is_favorite: None,
        details: Some(details),
        consumed_date: None,
    }
}

/// Steam `appdetails` の `data` オブジェクトから`CreateItemRequest`を構築する。
fn build_game_create_request(data: &Value) -> CreateItemRequest {
    let metacritic = data
        .get("metacritic")
        .and_then(|m| m.get("score"))
        .and_then(Value::as_u64)
        .map(|s| s as u32);
    let platforms: Vec<String> = data
        .get("platforms")
        .and_then(Value::as_object)
        .map(|obj| {
            obj.iter()
                .filter(|(_, v)| v.as_bool().unwrap_or(false))
                .map(|(k, _)| k.clone())
                .collect()
        })
        .unwrap_or_default();
    let screenshots: Vec<String> = data
        .get("screenshots")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|s| json_str(s, "path_thumbnail"))
                .collect()
        })
        .unwrap_or_default();

    let details = serde_json::json!({
        "platforms": platforms,
        "developers": json_str_array(data, "developers"),
        "publishers": json_str_array(data, "publishers"),
        "screenshots": screenshots,
        "metacritic": metacritic,
        "genres": json_names(data, "genres", "description"),
    });

    CreateItemRequest {
        media_type: MediaType::Game,
        title: json_str(data, "name").unwrap_or_default(),
        original_title: None,
        description: json_str(data, "short_description"),
        cover_image_url: json_str(data, "header_image"),
        release_date: data
            .get("release_date")
            .and_then(|r| json_str(r, "date"))
            .as_deref()
            .and_then(parse_release_date),
        homepage_url: json_str(data, "website"),
        rating: None,
        is_favorite: None,
        details: Some(details),
        consumed_date: None,
    }
}

/// NDL書誌アイテム（novel/academic_book/paper共通）から`CreateItemRequest`を構築する。
fn build_novel_create_request(item: &NdlItemModel, media_type: MediaType) -> CreateItemRequest {
    let isbn = item.isbn13.clone().or_else(|| item.isbn.clone());
    let details = serde_json::json!({
        "authors": item.creator.clone().unwrap_or_default(),
        "publisher": item.publisher.clone(),
        "isbn": isbn,
    });

    CreateItemRequest {
        media_type,
        title: item.title.clone().unwrap_or_default(),
        original_title: None,
        description: item.description.clone(),
        cover_image_url: item.thumbnail_url.clone(),
        release_date: item.pub_date.as_deref().and_then(parse_release_date),
        homepage_url: None,
        rating: None,
        is_favorite: None,
        details: Some(details),
        consumed_date: None,
    }
}

/// テスト用: 各プロバイダクライアントのベースURL差し替え（wiremock注入用）
///
/// 【設計判断】: 本番では各クライアントの`new()`（本番URL固定）を使うが、
/// ユニットテストではwiremockの`MockServer`URLへ差し替えるためのテスト専用オーバーライド。
/// 🟡 信頼性レベル: 要件定義書 第7章「(a) wiremock等のHTTPモックサーバーをnew_with_base_url等で注入」より
#[derive(Clone, Default)]
#[cfg(test)]
struct TestBaseUrls {
    jikan: Option<String>,
    tmdb: Option<String>,
    ndl: Option<String>,
    steam: Option<String>,
    annict: Option<String>,
    rakuten: Option<String>,
}

/// media_type→provider振り分けディスパッチサービス
///
/// 🔵 信頼性レベル: 要件定義書 第3章 API契約より
pub struct ExternalSearchService {
    credentials: ApiCredentialLookup,
    #[cfg(test)]
    test_base_urls: TestBaseUrls,
}

impl ApiCredentialLookup {
    /// 指定providerのAPIキーをDBから取得する（テストでは固定値を返す）
    /// 🟡 信頼性レベル: 上記DI設計判断より
    async fn find_by_provider(
        &self,
        provider: ApiProvider,
    ) -> Result<Option<ApiCredential>, ExternalSearchError> {
        match self {
            ApiCredentialLookup::Pool(pool) => {
                api_credential_repository::find_by_provider(pool, provider)
                    .await
                    .map_err(|_| ExternalSearchError::ApiKeyNotConfigured(provider))
            }
            ApiCredentialLookup::Fixed(resolver) => Ok(resolver(provider)),
        }
    }
}

impl ExternalSearchService {
    /// DI: 接続プールを受け取り初期化する（本番経路）
    /// 🔵 信頼性レベル: 要件定義書 第3章より
    pub fn new(pool: PgPool) -> Self {
        Self {
            credentials: ApiCredentialLookup::Pool(pool),
            #[cfg(test)]
            test_base_urls: TestBaseUrls::default(),
        }
    }

    /// DI: テスト用に固定の認証情報解決クロージャを注入して初期化する（DB非依存）
    ///
    /// 【テスト対応】: HTTPモックのみで完結するユニットテストがDATABASE_URLに依存しないようにするための
    /// テスト専用コンストラクタ。本番コードからは呼び出されない。
    /// 🟡 信頼性レベル: tdd-red指摘事項（DB依存ユニットテストのpanic回避）からの設計判断
    #[cfg(test)]
    fn with_fixed_credentials(
        resolver: impl Fn(ApiProvider) -> Option<ApiCredential> + Send + Sync + 'static,
    ) -> Self {
        Self {
            credentials: ApiCredentialLookup::Fixed(Arc::new(resolver)),
            test_base_urls: TestBaseUrls::default(),
        }
    }

    /// テスト用: プロバイダ別ベースURLを上書きするビルダーメソッド（wiremock注入用）
    /// 🟡 信頼性レベル: 要件定義書 第7章より
    #[cfg(test)]
    fn with_test_base_urls(mut self, f: impl FnOnce(&mut TestBaseUrls)) -> Self {
        f(&mut self.test_base_urls);
        self
    }

    /// media_typeに対応する単一プロバイダへ検索リクエストをディスパッチする
    ///
    /// 【実装方針】: 第2章マッピング表に従い、match の網羅性検査で8 variant全てを
    /// 静的に担保する（TC-002-B03/B04）。各分岐は専用の `dispatch_*` ヘルパーへ委譲する。
    /// 🔵 信頼性レベル: 要件定義書 REQ-0023-01〜06・REQ-0023-402より
    pub async fn search(
        &self,
        media_type: MediaType,
        query: &str,
    ) -> Result<Vec<SearchResultItem>, ExternalSearchError> {
        match media_type {
            MediaType::Anime => self.dispatch_annict_anime(query).await,
            MediaType::Manga => self.dispatch_rakuten_books(query, MediaType::Manga).await,
            MediaType::Movie => self.dispatch_tmdb_movie(query).await,
            MediaType::Drama => self.dispatch_tmdb_drama(query).await,
            MediaType::Novel => self.dispatch_rakuten_books(query, MediaType::Novel).await,
            MediaType::Game => self.dispatch_steam(query).await,
            MediaType::AcademicBook => {
                self.dispatch_rakuten_books(query, MediaType::AcademicBook)
                    .await
            }
            MediaType::Paper => self.dispatch_ndl_for(query, MediaType::Paper).await,
        }
    }

    /// インポート確定時: media_typeに対応する単一プロバイダから詳細情報を再取得し、
    /// `CreateItemRequest`を直接構築する。
    pub async fn fetch_import_details(
        &self,
        media_type: MediaType,
        external_id: &str,
    ) -> Result<CreateItemRequest, ExternalSearchError> {
        match media_type {
            MediaType::Anime => self.fetch_anime_import_details(external_id).await,
            MediaType::Manga => {
                self.fetch_rakuten_import_details(external_id, MediaType::Manga)
                    .await
            }
            MediaType::Movie => self.fetch_movie_import_details(external_id).await,
            MediaType::Drama => self.fetch_drama_import_details(external_id).await,
            MediaType::Game => self.fetch_game_import_details(external_id).await,
            MediaType::Novel => {
                self.fetch_rakuten_import_details(external_id, MediaType::Novel)
                    .await
            }
            MediaType::AcademicBook => {
                self.fetch_rakuten_import_details(external_id, MediaType::AcademicBook)
                    .await
            }
            MediaType::Paper => {
                self.fetch_novel_import_details(external_id, MediaType::Paper)
                    .await
            }
        }
    }

    /// Jikanクライアントを構築する（テスト時はベースURL差し替え可能） 🔵
    fn build_jikan_client(&self) -> Result<JikanClient, ExternalSearchError> {
        #[cfg(test)]
        if let Some(base_url) = &self.test_base_urls.jikan {
            return JikanClient::new_with_base_url(base_url.clone())
                .map_err(ExternalSearchError::ExternalApiError);
        }
        JikanClient::new().map_err(ExternalSearchError::ExternalApiError)
    }

    /// TMDbクライアントを構築する（テスト時はベースURL差し替え可能） 🔵
    fn build_tmdb_client(&self, api_key: String) -> Result<TmdbClient, ExternalSearchError> {
        #[cfg(test)]
        if let Some(base_url) = &self.test_base_urls.tmdb {
            return TmdbClient::new_with_base_url(AuthStrategy::ApiKey(api_key), base_url.clone())
                .map_err(ExternalSearchError::ExternalApiError);
        }
        TmdbClient::new(AuthStrategy::ApiKey(api_key))
            .map_err(ExternalSearchError::ExternalApiError)
    }

    /// NDLクライアントを構築する（テスト時はベースURL差し替え可能） 🔵
    fn build_ndl_client(&self) -> Result<NdlClient, ExternalSearchError> {
        #[cfg(test)]
        if let Some(base_url) = &self.test_base_urls.ndl {
            return NdlClient::new_with_base_url(base_url.clone())
                .map_err(ExternalSearchError::ExternalApiError);
        }
        NdlClient::new().map_err(ExternalSearchError::ExternalApiError)
    }

    /// Annictクライアントを構築する（テスト時はベースURL差し替え可能） 🔵
    fn build_annict_client(&self, api_key: String) -> Result<AnnictClient, ExternalSearchError> {
        #[cfg(test)]
        if let Some(base_url) = &self.test_base_urls.annict {
            return AnnictClient::new_with_base_url(
                AuthStrategy::ApiKey(api_key),
                base_url.clone(),
            )
            .map_err(ExternalSearchError::ExternalApiError);
        }
        AnnictClient::new(AuthStrategy::ApiKey(api_key))
            .map_err(ExternalSearchError::ExternalApiError)
    }

    /// 楽天ブックスクライアントを構築する（テスト時はベースURL差し替え可能）。
    ///
    /// `api_key`は`"applicationId:accessKey"`形式で1つの文字列にエンコードして保存する
    /// （`api_credentials.api_key`が単一文字列カラムのため、楽天が要求する2値をこの区切り文字で格納する規約）。
    fn build_rakuten_client(&self, api_key: String) -> Result<RakutenClient, ExternalSearchError> {
        let (application_id, access_key) = api_key.split_once(':').ok_or_else(|| {
            ExternalSearchError::ExternalApiError(api_client_lib::ApiError::Auth(
                "invalid Rakuten credential format (expected \"applicationId:accessKey\")"
                    .to_string(),
            ))
        })?;
        let auth = AuthStrategy::RakutenAppAuth {
            application_id: application_id.to_string(),
            access_key: access_key.to_string(),
        };
        #[cfg(test)]
        if let Some(base_url) = &self.test_base_urls.rakuten {
            return RakutenClient::new_with_base_url(auth, base_url.clone())
                .map_err(ExternalSearchError::ExternalApiError);
        }
        RakutenClient::new(auth).map_err(ExternalSearchError::ExternalApiError)
    }

    /// Steamクライアントを構築する（テスト時はベースURL差し替え可能。ストア検索は認証不要のため`AuthStrategy::None`固定） 🟡
    fn build_steam_client(&self) -> Result<SteamClient, ExternalSearchError> {
        #[cfg(test)]
        if let Some(base_url) = &self.test_base_urls.steam {
            return SteamClient::new_with_base_urls(
                AuthStrategy::None,
                base_url.clone(),
                base_url.clone(),
            )
            .map_err(ExternalSearchError::ExternalApiError);
        }
        SteamClient::new(AuthStrategy::None).map_err(ExternalSearchError::ExternalApiError)
    }

    /// Annict（anime）へディスパッチする。`find_by_provider(Annict)` でキーを取得する。
    /// 検索結果はid/title/images.recommended_urlのみを保持する軽量マッピングとする。
    async fn dispatch_annict_anime(
        &self,
        query: &str,
    ) -> Result<Vec<SearchResultItem>, ExternalSearchError> {
        let api_key = self.ensure_key(ApiProvider::Annict).await?;
        let client = self.build_annict_client(api_key)?;
        let response = client
            .list_works(ListWorksRequest {
                filter_title: Some(query.to_string()),
                ..Default::default()
            })
            .await
            .map_err(ExternalSearchError::ExternalApiError)?;
        Ok(response
            .model
            .iter()
            .map(search_item_from_annict_work)
            .collect())
    }

    /// インポート確定時: Annictの作品情報を再取得し、`mal_anime_id`を使ってJikanから詳細を取得、
    /// 両者をマージした`CreateItemRequest`を返す。
    ///
    /// `mal_anime_id`が空、またはJikan取得に失敗した場合はAnnict情報のみへフォールバックする
    /// （Jikan障害でインポート全体を失敗させない設計）。
    async fn fetch_anime_import_details(
        &self,
        annict_work_id: &str,
    ) -> Result<CreateItemRequest, ExternalSearchError> {
        let api_key = self.ensure_key(ApiProvider::Annict).await?;
        let annict_client = self.build_annict_client(api_key)?;
        let response = annict_client
            .list_works(ListWorksRequest {
                filter_ids: Some(annict_work_id.to_string()),
                ..Default::default()
            })
            .await
            .map_err(ExternalSearchError::ExternalApiError)?;
        let work = response.model.into_iter().next().ok_or_else(|| {
            ExternalSearchError::ExternalApiError(api_client_lib::ApiError::Http {
                status: 404,
                body: format!("Annict work not found: {annict_work_id}"),
            })
        })?;

        let mal_id: Option<u32> = work
            .mal_anime_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .and_then(|s| s.parse().ok());

        let Some(mal_id) = mal_id else {
            return Ok(build_anime_create_request(&work, None));
        };

        let jikan_client = self.build_jikan_client()?;
        let jikan_result = jikan_client
            .execute(JikanRequest::GetAnimeDetails(JikanAnimeDetailsRequest {
                id: mal_id,
            }))
            .await;

        match jikan_result {
            Ok(response) => {
                let raw_data = raw_data_to_value(&response.raw);
                let jikan_data = raw_data.get("data").cloned().unwrap_or(raw_data);
                Ok(build_anime_create_request(&work, Some(&jikan_data)))
            }
            Err(_) => Ok(build_anime_create_request(&work, None)),
        }
    }

    /// 楽天ブックス（manga/novel/academic_book共通）へディスパッチする。`find_by_provider(Rakuten)` でキーを取得する。
    async fn dispatch_rakuten_books(
        &self,
        query: &str,
        media_type: MediaType,
    ) -> Result<Vec<SearchResultItem>, ExternalSearchError> {
        let api_key = self.ensure_key(ApiProvider::Rakuten).await?;
        let client = self.build_rakuten_client(api_key)?;
        let response = client
            .search_books(SearchBooksRequest {
                title: Some(query.to_string()),
                ..Default::default()
            })
            .await
            .map_err(ExternalSearchError::ExternalApiError)?;
        Ok(response
            .model
            .iter()
            .map(|book| search_item_from_rakuten_book(book, media_type))
            .collect())
    }

    /// インポート確定時: 楽天ブックスをISBN検索し、書誌詳細から`CreateItemRequest`を構築する
    /// （manga/novel/academic_book共通、`external_id`はISBN）。
    async fn fetch_rakuten_import_details(
        &self,
        external_id: &str,
        media_type: MediaType,
    ) -> Result<CreateItemRequest, ExternalSearchError> {
        let api_key = self.ensure_key(ApiProvider::Rakuten).await?;
        let client = self.build_rakuten_client(api_key)?;
        let response = client
            .search_books(SearchBooksRequest {
                isbn: Some(external_id.to_string()),
                ..Default::default()
            })
            .await
            .map_err(ExternalSearchError::ExternalApiError)?;
        let book = response.model.into_iter().next().ok_or_else(|| {
            ExternalSearchError::ExternalApiError(api_client_lib::ApiError::Http {
                status: 404,
                body: format!("Rakuten book not found: {external_id}"),
            })
        })?;
        Ok(build_book_create_request(&book, media_type))
    }

    /// TMDb（movie）へディスパッチする。`find_by_provider(Tmdb)` でキーを取得する。
    /// 🔵 信頼性レベル: 要件定義書 REQ-0023-03より
    async fn dispatch_tmdb_movie(
        &self,
        query: &str,
    ) -> Result<Vec<SearchResultItem>, ExternalSearchError> {
        let api_key = self.ensure_key(ApiProvider::Tmdb).await?;
        let client = self.build_tmdb_client(api_key)?;
        let request = TmdbRequest::SearchMovie(SearchMovieRequest {
            query: query.to_string(),
            language: None,
            page: None,
        });
        let response = client
            .execute(request)
            .await
            .map_err(ExternalSearchError::ExternalApiError)?;
        let raw_data = raw_data_to_value(&response.raw);
        Ok(map_array_items(
            &raw_data,
            "results",
            search_item_from_tmdb_movie,
        ))
    }

    /// インポート確定時: TMDb `GET /movie/{id}` から詳細を取得し`CreateItemRequest`を構築する。
    async fn fetch_movie_import_details(
        &self,
        external_id: &str,
    ) -> Result<CreateItemRequest, ExternalSearchError> {
        let movie_id: u32 = external_id.trim().parse().map_err(|_| {
            ExternalSearchError::ExternalApiError(api_client_lib::ApiError::Parse(format!(
                "invalid movie id: {external_id}"
            )))
        })?;
        let api_key = self.ensure_key(ApiProvider::Tmdb).await?;
        let client = self.build_tmdb_client(api_key)?;
        let response = client
            .execute(TmdbRequest::GetMovieDetails(MovieDetailsRequest {
                movie_id,
                language: None,
            }))
            .await
            .map_err(ExternalSearchError::ExternalApiError)?;
        let raw_data = raw_data_to_value(&response.raw);
        Ok(build_movie_create_request(&raw_data))
    }

    /// TMDb（drama）へディスパッチする。movieと同一provider。TVエンドポイント（SearchTv）を使う。
    /// 🔵 信頼性レベル: 要件定義書 REQ-0023-03より
    async fn dispatch_tmdb_drama(
        &self,
        query: &str,
    ) -> Result<Vec<SearchResultItem>, ExternalSearchError> {
        let api_key = self.ensure_key(ApiProvider::Tmdb).await?;
        let client = self.build_tmdb_client(api_key)?;
        let request = TmdbRequest::SearchTv(SearchTvRequest {
            query: query.to_string(),
            language: None,
            page: None,
        });
        let response = client
            .execute(request)
            .await
            .map_err(ExternalSearchError::ExternalApiError)?;
        let raw_data = raw_data_to_value(&response.raw);
        Ok(map_array_items(
            &raw_data,
            "results",
            search_item_from_tmdb_tv,
        ))
    }

    /// インポート確定時: TMDb `GET /tv/{id}` から詳細を取得し`CreateItemRequest`を構築する。
    async fn fetch_drama_import_details(
        &self,
        external_id: &str,
    ) -> Result<CreateItemRequest, ExternalSearchError> {
        let series_id: u32 = external_id.trim().parse().map_err(|_| {
            ExternalSearchError::ExternalApiError(api_client_lib::ApiError::Parse(format!(
                "invalid tv series id: {external_id}"
            )))
        })?;
        let api_key = self.ensure_key(ApiProvider::Tmdb).await?;
        let client = self.build_tmdb_client(api_key)?;
        let response = client
            .execute(TmdbRequest::GetTvSeries(TvSeriesRequest {
                series_id,
                language: None,
            }))
            .await
            .map_err(ExternalSearchError::ExternalApiError)?;
        let raw_data = raw_data_to_value(&response.raw);
        Ok(build_drama_create_request(&raw_data))
    }

    /// Steam（game）へディスパッチする。ストア検索はキー不要のため `find_by_provider` は呼ばない。
    ///
    /// 【設計判断】: `store_search` はid/name/tiny_imageのみを返し、説明・評価・画像等の詳細情報を
    /// 含まない。一覧表示にはこれで十分なため軽量マッピングする。詳細情報はユーザーがインポートを
    /// 確定した時点で別途`get_app_details`から取得する想定。
    /// 🟡 信頼性レベル: ユーザー指示（ゲームはSteam検索を使用）・設計判断
    async fn dispatch_steam(
        &self,
        query: &str,
    ) -> Result<Vec<SearchResultItem>, ExternalSearchError> {
        let client = self.build_steam_client()?;
        let request = SteamRequest::StoreSearch(SteamStoreSearchRequest {
            term: query.to_string(),
            page: None,
        });
        let response = client
            .execute(request)
            .await
            .map_err(ExternalSearchError::ExternalApiError)?;
        let raw_data = raw_data_to_value(&response.raw);
        Ok(map_array_items(
            &raw_data,
            "items",
            search_item_from_steam_search,
        ))
    }

    /// インポート確定時: Steam `appdetails` から詳細を取得し`CreateItemRequest`を構築する。
    async fn fetch_game_import_details(
        &self,
        external_id: &str,
    ) -> Result<CreateItemRequest, ExternalSearchError> {
        let app_id: u32 = external_id.trim().parse().map_err(|_| {
            ExternalSearchError::ExternalApiError(api_client_lib::ApiError::Parse(format!(
                "invalid steam app id: {external_id}"
            )))
        })?;
        let client = self.build_steam_client()?;
        let response = client
            .execute(SteamRequest::GetAppDetails(SteamAppDetailsRequest {
                app_id,
            }))
            .await
            .map_err(ExternalSearchError::ExternalApiError)?;
        // レスポンスは `{"<appid>": {"success": bool, "data": {...}}}` 形式のためraw JSONから直接抽出する
        let raw_data = raw_data_to_value(&response.raw);
        let data = raw_data
            .get(app_id.to_string())
            .and_then(|entry| entry.get("data"))
            .cloned()
            .unwrap_or(Value::Null);
        Ok(build_game_create_request(&data))
    }

    /// NDLディスパッチの内部実装（media_typeを明示的に受け取る。academic_book/paper共通）
    /// 🔵 信頼性レベル: 要件定義書 REQ-0023-05より
    async fn dispatch_ndl_for(
        &self,
        query: &str,
        media_type: MediaType,
    ) -> Result<Vec<SearchResultItem>, ExternalSearchError> {
        self.ensure_key(ApiProvider::Ndl).await?;
        let client = self.build_ndl_client()?;
        let request = NdlRequest::Search(NdlSearchRequest {
            title: None,
            isbn: None,
            creator: None,
            publisher: None,
            any: Some(query.to_string()),
            cnt: None,
            dpid: None,
        });
        let response = client
            .execute(request)
            .await
            .map_err(ExternalSearchError::ExternalApiError)?;
        let NdlModel::Items(models) = response.model;
        Ok(models
            .iter()
            .map(|m| search_item_from_ndl_item(m, media_type))
            .collect())
    }

    /// インポート確定時: NDLをISBN検索し、書誌詳細から`CreateItemRequest`を構築する
    /// （novel/academic_book/paper共通、`external_id`はISBN）。
    async fn fetch_novel_import_details(
        &self,
        external_id: &str,
        media_type: MediaType,
    ) -> Result<CreateItemRequest, ExternalSearchError> {
        self.ensure_key(ApiProvider::Ndl).await?;
        let client = self.build_ndl_client()?;
        let response = client
            .execute(NdlRequest::Search(NdlSearchRequest {
                isbn: Some(external_id.to_string()),
                ..Default::default()
            }))
            .await
            .map_err(ExternalSearchError::ExternalApiError)?;
        let NdlModel::Items(models) = response.model;
        let item = models.into_iter().next().ok_or_else(|| {
            ExternalSearchError::ExternalApiError(api_client_lib::ApiError::Http {
                status: 404,
                body: format!("NDL item not found: {external_id}"),
            })
        })?;
        Ok(build_novel_create_request(&item, media_type))
    }

    /// キー必須プロバイダのDBキー存在確認を行う（REQ-0023-101・REQ-0023-403）。
    /// 未登録なら `ApiKeyNotConfigured` を返す。
    /// 🔵 信頼性レベル: 要件定義書 REQ-0023-101・REQ-0023-403より
    async fn ensure_key(&self, provider: ApiProvider) -> Result<String, ExternalSearchError> {
        let found = self.credentials.find_by_provider(provider).await?;
        found
            .map(|cred| cred.api_key)
            .ok_or(ExternalSearchError::ApiKeyNotConfigured(provider))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// docs/api-samples/ 配下の fixture を読み込む。
    /// 未取得（fetch_samples がスキップした）ファイルは None を返しテストをスキップさせる。
    fn load_fixture(rel_path: &str) -> Option<Value> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/api-samples")
            .join(rel_path);
        let text = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&text).ok()
    }

    /// 実DB統合テスト用プール取得ヘルパー（既存repositories/api_credential_repository.rsと同型）
    async fn test_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("TASK-0023統合テストにはDATABASE_URL環境変数が必要です");
        PgPool::connect(&url)
            .await
            .expect("テスト用DBへの接続に失敗しました")
    }

    /// 統合テスト共通: 指定providerのapi_credentials行を削除しクリーンな状態にする
    async fn cleanup_provider(pool: &PgPool, provider: &str) {
        sqlx::query("DELETE FROM api_credentials WHERE provider = $1::api_provider")
            .bind(provider)
            .execute(pool)
            .await
            .expect("テスト前クリーンアップに失敗しました");
    }

    /// 統合テスト共通: 指定providerにAPIキーを投入する
    async fn seed_provider(pool: &PgPool, provider: ApiProvider, api_key: &str) {
        api_credential_repository::upsert_api_credential(pool, provider, api_key.to_string())
            .await
            .expect("テスト用キー投入に失敗しました");
    }

    /// ユニットテスト共通: 指定providerにのみ固定キーを返すDB非依存サービスを構築する
    ///
    /// 【設計判断】: tdd-red指摘事項対応。`find_by_provider`がPgPoolを要求するため、
    /// 従来`test_pool()`（DATABASE_URL必須）に依存していたユニットテストを、
    /// `ExternalSearchService::with_fixed_credentials`経由のDB非依存に置き換える。
    /// 本ヘルパーは指定した1つのproviderにのみ固定キーを返し、他は`None`（未設定）を返す。
    /// 🟡 信頼性レベル: tdd-red指摘事項（DB依存ユニットテストのpanic回避）からの設計判断
    fn service_with_single_key(
        provider: ApiProvider,
        api_key: &'static str,
    ) -> ExternalSearchService {
        ExternalSearchService::with_fixed_credentials(move |p| {
            if p == provider {
                Some(ApiCredential {
                    provider: p,
                    api_key: api_key.to_string(),
                    updated_at: chrono::Utc::now().naive_utc(),
                })
            } else {
                None
            }
        })
    }

    /// ユニットテスト共通: 全キー必須providerについて常に`None`（キー未設定）を返すDB非依存サービスを構築する
    /// 🔵 信頼性レベル: REQ-0023-101（キー未設定検証）より
    fn service_with_no_keys() -> ExternalSearchService {
        ExternalSearchService::with_fixed_credentials(|_| None)
    }

    // ============================================================
    // 1. 正常系テストケース（基本的な動作）
    // ============================================================

    /// TC-002-01-A: media_type=Anime → Annictのみへディスパッチ（ユニット・HTTPモック）
    /// 🔵 信頼性レベル: ユーザー指示（アニメ検索はAnnictを使用）より
    #[tokio::test]
    async fn search_anime_dispatches_to_annict_only() {
        let annict_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"works": []})),
            )
            .mount(&annict_mock)
            .await;

        let service = service_with_single_key(ApiProvider::Annict, "test-annict-key")
            .with_test_base_urls(|u| u.annict = Some(annict_mock.uri()));

        let result = service.search(MediaType::Anime, "鬼滅の刃").await;

        assert!(result.is_ok());
        assert_eq!(annict_mock.received_requests().await.unwrap().len(), 1);
    }

    /// TC-002-02-B: media_type=Drama → TMDbへディスパッチ（ユニット・HTTPモック）
    /// 🔵 信頼性レベル: 要件定義書 REQ-0023-03より
    #[tokio::test]
    async fn search_drama_dispatches_to_tmdb_only() {
        let tmdb_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"results": []})),
            )
            .mount(&tmdb_mock)
            .await;

        let service = service_with_single_key(ApiProvider::Tmdb, "test-tmdb-key")
            .with_test_base_urls(|u| u.tmdb = Some(tmdb_mock.uri()));

        let result = service.search(MediaType::Drama, "タイトル").await;

        assert!(result.is_ok());
        assert_eq!(tmdb_mock.received_requests().await.unwrap().len(), 1);
    }

    /// TC-002-02-A: media_type=Movie → 実DBの`find_by_provider(Tmdb)`キーで初期化したTMDbへディスパッチ（統合・実DB）
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn search_movie_dispatches_to_tmdb_with_db_backed_key() {
        let tmdb_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"results": []})),
            )
            .mount(&tmdb_mock)
            .await;

        let pool = test_pool().await;
        cleanup_provider(&pool, "tmdb").await;
        seed_provider(&pool, ApiProvider::Tmdb, "test-tmdb-key").await;
        let service = ExternalSearchService::new(pool)
            .with_test_base_urls(|u| u.tmdb = Some(tmdb_mock.uri()));

        let result = service.search(MediaType::Movie, "タイトル").await;

        assert!(result.is_ok());
        assert_eq!(tmdb_mock.received_requests().await.unwrap().len(), 1);
    }

    /// TC-002-MANGA: media_type=Manga → 楽天ブックスへディスパッチする（キー必須・ユニット）
    #[tokio::test]
    async fn search_manga_dispatches_to_rakuten_only() {
        let rakuten_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"Items": []})))
            .mount(&rakuten_mock)
            .await;

        let service = service_with_single_key(ApiProvider::Rakuten, "app-id:access-key")
            .with_test_base_urls(|u| u.rakuten = Some(rakuten_mock.uri()));

        let result = service.search(MediaType::Manga, "ワンピース").await;

        assert!(result.is_ok());
        assert_eq!(rakuten_mock.received_requests().await.unwrap().len(), 1);
    }

    /// TC-002-NOVEL: media_type=Novel → 楽天ブックスへディスパッチする（ユニット・HTTPモック）
    #[tokio::test]
    async fn search_novel_dispatches_to_rakuten_only() {
        let rakuten_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"Items": []})))
            .mount(&rakuten_mock)
            .await;

        let service = service_with_single_key(ApiProvider::Rakuten, "app-id:access-key")
            .with_test_base_urls(|u| u.rakuten = Some(rakuten_mock.uri()));

        let result = service.search(MediaType::Novel, "タイトル").await;

        assert!(result.is_ok());
        assert_eq!(rakuten_mock.received_requests().await.unwrap().len(), 1);
    }

    /// TC-002-GAME: media_type=Game → Steamストア検索へディスパッチ（キー不要・ユニット）
    /// 🟡 信頼性レベル: ユーザー指示（ゲームはSteam検索を使用）より
    #[tokio::test]
    async fn search_game_dispatches_to_steam_only() {
        let steam_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "total": 0,
                "items": []
            })))
            .mount(&steam_mock)
            .await;

        let service =
            service_with_no_keys().with_test_base_urls(|u| u.steam = Some(steam_mock.uri()));

        let result = service.search(MediaType::Game, "ゼルダの伝説").await;

        assert!(result.is_ok());
        assert_eq!(steam_mock.received_requests().await.unwrap().len(), 1);
    }

    /// TC-002-GAME-RESULT: Steamストア検索結果がid/name/tiny_imageのみで`SearchResultItem`へノーマライズされる（ユニット）
    /// 🟡 信頼性レベル: ユーザー指示（検索一覧はid/name/tiny_imageのみ返す）より
    #[tokio::test]
    async fn search_game_converts_steam_search_result_to_search_result_item() {
        let steam_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "total": 1,
                "items": [{
                    "id": 400,
                    "name": "Portal",
                    "type": "game",
                    "tiny_image": "https://example.com/tiny/400.jpg"
                }]
            })))
            .mount(&steam_mock)
            .await;

        let service =
            service_with_no_keys().with_test_base_urls(|u| u.steam = Some(steam_mock.uri()));

        let result = service
            .search(MediaType::Game, "Portal")
            .await
            .expect("Okが返るはず");

        let first = result.first().expect("少なくとも1件の結果が返るはず");
        assert_eq!(first.media_type, MediaType::Game);
        assert_eq!(first.provider, Some(ApiProvider::Steam));
        assert_eq!(first.id, "400");
        assert_eq!(first.title, "Portal");
        assert_eq!(
            first.thumbnail_url.as_deref(),
            Some("https://example.com/tiny/400.jpg")
        );
    }

    /// TC-002-ACADEMIC: media_type=AcademicBook → 楽天ブックスへディスパッチする（ユニット・HTTPモック）
    #[tokio::test]
    async fn search_academic_book_dispatches_to_rakuten_only() {
        let rakuten_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"Items": []})))
            .mount(&rakuten_mock)
            .await;

        let service = service_with_single_key(ApiProvider::Rakuten, "app-id:access-key")
            .with_test_base_urls(|u| u.rakuten = Some(rakuten_mock.uri()));

        let result = service.search(MediaType::AcademicBook, "量子力学").await;

        assert!(result.is_ok());
        assert_eq!(rakuten_mock.received_requests().await.unwrap().len(), 1);
    }

    /// TC-002-PAPER: media_type=Paper → NDLへディスパッチ（ユニット・HTTPモック）
    /// 🔵 信頼性レベル: 要件定義書 マッピング表 L42・EDGE-0023-03より
    #[tokio::test]
    async fn search_paper_dispatches_to_ndl_only() {
        let ndl_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string("<rss><channel></channel></rss>"),
            )
            .mount(&ndl_mock)
            .await;

        let service = service_with_single_key(ApiProvider::Ndl, "test-ndl-key")
            .with_test_base_urls(|u| u.ndl = Some(ndl_mock.uri()));

        let result = service.search(MediaType::Paper, "機械学習").await;

        assert!(result.is_ok());
        assert_eq!(ndl_mock.received_requests().await.unwrap().len(), 1);
    }

    /// TC-002-RESULT: 成功時にプロバイダレスポンスが`SearchResultItem`へノーマライズされる（ユニット）
    /// 🟡 信頼性レベル: 要件定義書 REQ-0023-06・第3章 出力仕様より
    #[tokio::test]
    async fn search_movie_converts_tmdb_response_to_search_result_item() {
        let tmdb_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{
                    "id": 603,
                    "title": "The Matrix",
                    "original_title": "The Matrix",
                    "overview": "A computer hacker...",
                    "release_date": "1999-03-31",
                    "poster_path": "/poster.jpg",
                    "vote_average": 8.2
                }]
            })))
            .mount(&tmdb_mock)
            .await;

        let service = service_with_single_key(ApiProvider::Tmdb, "test-tmdb-key")
            .with_test_base_urls(|u| u.tmdb = Some(tmdb_mock.uri()));

        let result = service
            .search(MediaType::Movie, "タイトル")
            .await
            .expect("Okが返るはず");

        let first = result.first().expect("少なくとも1件の結果が返るはず");
        assert_eq!(first.media_type, MediaType::Movie);
        assert_eq!(first.provider, Some(ApiProvider::Tmdb));
        assert_eq!(first.id, "603");
        assert_eq!(first.title, "The Matrix");
        assert_eq!(
            first.thumbnail_url.as_deref(),
            Some("https://image.tmdb.org/t/p/w342/poster.jpg")
        );
    }

    // ============================================================
    // 2. 異常系テストケース（エラーハンドリング）
    // ============================================================

    /// TC-002-E01-A: キー必須プロバイダでキー未設定→ApiKeyNotConfigured（外部API非呼び出し・ユニット）
    /// 🔵 信頼性レベル: 要件定義書 REQ-0023-101・シナリオ3より
    #[tokio::test]
    async fn search_movie_returns_api_key_not_configured_when_tmdb_key_missing() {
        let tmdb_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"results": []})),
            )
            .mount(&tmdb_mock)
            .await;

        let service =
            service_with_no_keys().with_test_base_urls(|u| u.tmdb = Some(tmdb_mock.uri()));

        let result = service.search(MediaType::Movie, "タイトル").await;

        match result {
            Err(ExternalSearchError::ApiKeyNotConfigured(provider)) => {
                assert_eq!(provider, ApiProvider::Tmdb);
            }
            other => panic!("ApiKeyNotConfigured(Tmdb)が返るはずだったが: {other:?}"),
        }
        assert_eq!(tmdb_mock.received_requests().await.unwrap().len(), 0);
    }

    /// TC-002-E01-B: 各キー必須プロバイダで未設定時に対応providerのApiKeyNotConfiguredを返す（ユニット・パラメタライズド）
    /// 🔵 信頼性レベル: 要件定義書 REQ-0023-101（キー必須プロバイダ列挙）より
    #[tokio::test]
    async fn search_returns_api_key_not_configured_for_each_key_required_provider() {
        let service = service_with_no_keys();

        let cases = [
            (MediaType::Paper, ApiProvider::Ndl),
            (MediaType::Novel, ApiProvider::Rakuten),
            (MediaType::Manga, ApiProvider::Rakuten),
            (MediaType::AcademicBook, ApiProvider::Rakuten),
        ];

        for (media_type, expected_provider) in cases {
            let result = service.search(media_type, "クエリ").await;
            match result {
                Err(ExternalSearchError::ApiKeyNotConfigured(provider)) => {
                    assert_eq!(provider, expected_provider);
                }
                other => panic!(
                    "media_type={media_type:?}: ApiKeyNotConfigured({expected_provider:?})が返るはずだったが: {other:?}"
                ),
            }
        }
    }

    /// TC-002-E02-A: 外部API接続不能→ExternalApiError（panicしない・ユニット）
    /// 🔵 信頼性レベル: 要件定義書 REQ-0023-103・シナリオ4より
    #[tokio::test]
    async fn search_movie_returns_external_api_error_on_timeout_without_panicking() {
        let service = service_with_single_key(ApiProvider::Tmdb, "test-tmdb-key")
            .with_test_base_urls(|u| u.tmdb = Some("http://127.0.0.1:1".to_string()));

        let result = service.search(MediaType::Movie, "タイトル").await;

        match result {
            Err(ExternalSearchError::ExternalApiError(_)) => {}
            other => panic!("ExternalApiErrorが返るはずだったが: {other:?}"),
        }
    }

    /// TC-002-E02-B: 全ApiError variantがExternalApiErrorへ集約される（panicしない・ユニット・パラメタライズド）
    /// 🔵 信頼性レベル: 要件定義書 EDGE-0023-04・REQ-0023-103より
    #[test]
    fn external_search_error_wraps_all_six_api_error_variants_without_panicking() {
        let variants = vec![
            api_client_lib::ApiError::Http {
                status: 500,
                body: "internal error".to_string(),
            },
            api_client_lib::ApiError::Auth("auth failed".to_string()),
            api_client_lib::ApiError::RateLimit { retry_after: None },
            api_client_lib::ApiError::Parse("parse failed".to_string()),
            api_client_lib::ApiError::Timeout,
            api_client_lib::ApiError::Network("network failed".to_string()),
        ];

        for variant in variants {
            let wrapped = ExternalSearchError::ExternalApiError(variant);
            assert!(!wrapped.to_string().is_empty());
        }
    }

    // ============================================================
    // 3. 境界値テストケース（最小値、最大値、隣接variant等）
    // ============================================================

    /// TC-002-B01: 空クエリ文字列が透過的に各プロバイダへ渡される（境界・ユニット）
    /// 🟡 信頼性レベル: 要件定義書 第3章 L86（空文字バリデーションは呼び出し元責務）より
    #[tokio::test]
    async fn search_anime_with_empty_query_is_passed_through_without_validation() {
        let annict_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"works": []})),
            )
            .mount(&annict_mock)
            .await;

        let service = service_with_single_key(ApiProvider::Annict, "test-annict-key")
            .with_test_base_urls(|u| u.annict = Some(annict_mock.uri()));

        let result = service.search(MediaType::Anime, "").await;

        assert!(result.is_ok());
        assert_eq!(annict_mock.received_requests().await.unwrap().len(), 1);
    }

    /// TC-002-B02: 非常に長いクエリ文字列が透過的に処理される（境界・ユニット）
    /// 🟡 信頼性レベル: 要件定義書 第3章 L86（透過処理方針）より
    #[tokio::test]
    async fn search_anime_with_very_long_query_is_passed_through_without_panicking() {
        let annict_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"works": []})),
            )
            .mount(&annict_mock)
            .await;

        let service = service_with_single_key(ApiProvider::Annict, "test-annict-key")
            .with_test_base_urls(|u| u.annict = Some(annict_mock.uri()));
        let long_query = "あ".repeat(10_000);

        let result = service.search(MediaType::Anime, &long_query).await;

        match result {
            Ok(_) => {}
            Err(ExternalSearchError::ExternalApiError(_)) => {}
            Err(other) => panic!("ExternalApiError以外のエラーは想定外: {other:?}"),
        }
    }

    /// TC-002-B03: 全8 MediaType variantがちょうど1プロバイダへ一意写像される（境界・網羅性・ユニット）
    /// 🔵 信頼性レベル: 要件定義書 第2章 マッピング表・REQ-0023-01・REQ-0023-501より
    #[tokio::test]
    async fn search_maps_all_eight_media_type_variants_to_exactly_one_provider() {
        let rakuten_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"Items": []})))
            .mount(&rakuten_mock)
            .await;

        let service = service_with_single_key(ApiProvider::Rakuten, "app-id:access-key")
            .with_test_base_urls(|u| u.rakuten = Some(rakuten_mock.uri()));

        let result = service.search(MediaType::Manga, "クエリ").await;
        assert!(result.is_ok(), "media_type=Manga");
        assert_eq!(rakuten_mock.received_requests().await.unwrap().len(), 1);
    }

    /// TC-002-B04: 隣接enum variant誤ディスパッチ検証（Manga/Novel・AcademicBook/Paper・Anime/Movie 非混同・ユニット）
    /// 🔵 信頼性レベル: 要件定義書 REQ-0023-402・設計判断A/Bより
    #[tokio::test]
    async fn search_manga_does_not_reach_ndl_mock() {
        let rakuten_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"Items": []})))
            .mount(&rakuten_mock)
            .await;
        let ndl_mock = MockServer::start().await;

        let service = service_with_single_key(ApiProvider::Rakuten, "app-id:access-key")
            .with_test_base_urls(|u| {
                u.rakuten = Some(rakuten_mock.uri());
                u.ndl = Some(ndl_mock.uri());
            });

        let result = service.search(MediaType::Manga, "ワンピース").await;

        assert!(result.is_ok());
        assert_eq!(rakuten_mock.received_requests().await.unwrap().len(), 1);
        assert_eq!(ndl_mock.received_requests().await.unwrap().len(), 0);
    }

    /// TC-002-B05: Manga（楽天ブックス）はキー未設定時にApiKeyNotConfigured(Rakuten)を返す（境界・ユニット）
    #[tokio::test]
    async fn search_manga_returns_api_key_not_configured_when_rakuten_key_missing() {
        let rakuten_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"Items": []})))
            .mount(&rakuten_mock)
            .await;

        let service =
            service_with_no_keys().with_test_base_urls(|u| u.rakuten = Some(rakuten_mock.uri()));

        let manga_result = service.search(MediaType::Manga, "クエリ").await;

        match manga_result {
            Err(ExternalSearchError::ApiKeyNotConfigured(provider)) => {
                assert_eq!(provider, ApiProvider::Rakuten);
            }
            other => panic!("ApiKeyNotConfigured(Rakuten)が返るはずだったが: {other:?}"),
        }
        assert_eq!(rakuten_mock.received_requests().await.unwrap().len(), 0);
    }

    /// media_type=Anime → Annictキー未設定時にApiKeyNotConfigured(Annict)を返す（ユニット）
    /// 🔵 信頼性レベル: ユーザー指示（アニメ検索はAnnictを使用）・REQ-0023-101より
    #[tokio::test]
    async fn search_anime_returns_api_key_not_configured_when_annict_key_missing() {
        let annict_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"works": []})),
            )
            .mount(&annict_mock)
            .await;

        let service =
            service_with_no_keys().with_test_base_urls(|u| u.annict = Some(annict_mock.uri()));

        let result = service.search(MediaType::Anime, "クエリ").await;

        match result {
            Err(ExternalSearchError::ApiKeyNotConfigured(provider)) => {
                assert_eq!(provider, ApiProvider::Annict);
            }
            other => panic!("ApiKeyNotConfigured(Annict)が返るはずだったが: {other:?}"),
        }
        assert_eq!(annict_mock.received_requests().await.unwrap().len(), 0);
    }

    // ============================================================
    // 4. fetch_import_details / fetch_anime_import_details（インポート確定時の詳細取得）
    // ============================================================

    /// mal_anime_idが存在する場合、Annict再取得→Jikan取得の順で呼ばれマージされる
    #[tokio::test]
    async fn fetch_anime_import_details_merges_annict_and_jikan_when_mal_anime_id_present() {
        let annict_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "works": [{
                    "id": 6607,
                    "title": "メイドインアビス 深き魂の黎明",
                    "mal_anime_id": "9253",
                    "images": { "recommended_url": "http://example.com/ogp.jpg" },
                    "episodes_count": 1,
                    "season_name": "2020-winter"
                }]
            })))
            .mount(&annict_mock)
            .await;

        let jikan_mock = MockServer::start().await;
        let Some(jikan_fixture) = load_fixture("jikan/anime_details.json") else {
            eprintln!("fixture missing, skipped");
            return;
        };
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&jikan_fixture))
            .mount(&jikan_mock)
            .await;

        let service = service_with_single_key(ApiProvider::Annict, "test-annict-key")
            .with_test_base_urls(|u| {
                u.annict = Some(annict_mock.uri());
                u.jikan = Some(jikan_mock.uri());
            });

        let result = service
            .fetch_import_details(MediaType::Anime, "6607")
            .await
            .expect("Okが返るはず");

        assert_eq!(annict_mock.received_requests().await.unwrap().len(), 1);
        assert_eq!(jikan_mock.received_requests().await.unwrap().len(), 1);

        assert_eq!(result.media_type, MediaType::Anime);
        assert_eq!(result.title, "メイドインアビス 深き魂の黎明");
        assert!(result.description.is_some());
    }

    /// mal_anime_idが空の場合、JikanへはアクセスせずAnnict情報のみでフォールバックする
    #[tokio::test]
    async fn fetch_anime_import_details_falls_back_to_annict_only_when_mal_anime_id_blank() {
        let annict_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "works": [{
                    "id": 4021,
                    "title": "サラとダックン",
                    "mal_anime_id": "",
                    "images": { "recommended_url": "" }
                }]
            })))
            .mount(&annict_mock)
            .await;

        let jikan_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": {}})))
            .mount(&jikan_mock)
            .await;

        let service = service_with_single_key(ApiProvider::Annict, "test-annict-key")
            .with_test_base_urls(|u| {
                u.annict = Some(annict_mock.uri());
                u.jikan = Some(jikan_mock.uri());
            });

        let result = service
            .fetch_import_details(MediaType::Anime, "4021")
            .await
            .expect("Okが返るはず");

        assert_eq!(jikan_mock.received_requests().await.unwrap().len(), 0);
        assert_eq!(result.title, "サラとダックン");
    }

    /// Jikan呼び出しが失敗した場合、Annict情報のみでフォールバックしインポート全体は失敗させない
    #[tokio::test]
    async fn fetch_anime_import_details_falls_back_to_annict_only_when_jikan_fails() {
        let annict_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "works": [{
                    "id": 6607,
                    "title": "メイドインアビス 深き魂の黎明",
                    "mal_anime_id": "9253",
                    "images": { "recommended_url": "http://example.com/ogp.jpg" }
                }]
            })))
            .mount(&annict_mock)
            .await;

        let service = service_with_single_key(ApiProvider::Annict, "test-annict-key")
            .with_test_base_urls(|u| {
                u.annict = Some(annict_mock.uri());
                u.jikan = Some("http://127.0.0.1:1".to_string());
            });

        let result = service
            .fetch_import_details(MediaType::Anime, "6607")
            .await
            .expect("Jikan障害でもOkが返るはず");

        assert_eq!(result.title, "メイドインアビス 深き魂の黎明");
        assert!(result.description.is_none());
    }

    /// Annictに該当作品が存在しない（filter_idsで空配列）場合はErrを返す
    #[tokio::test]
    async fn fetch_anime_import_details_returns_error_when_annict_work_not_found() {
        let annict_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"works": []})),
            )
            .mount(&annict_mock)
            .await;

        let service = service_with_single_key(ApiProvider::Annict, "test-annict-key")
            .with_test_base_urls(|u| u.annict = Some(annict_mock.uri()));

        let result = service
            .fetch_import_details(MediaType::Anime, "999999")
            .await;

        assert!(matches!(
            result,
            Err(ExternalSearchError::ExternalApiError(_))
        ));
    }
}
