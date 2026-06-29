pub mod models;
pub mod requests;

use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::Mutex;

use crate::auth::AuthStrategy;
use crate::error::ApiError;
use crate::rate_limit::RateLimitState;
use crate::response::{ApiResponse, RawData, RequestResult};
use crate::traits::ApiClient;

use self::models::{
    AniListModel, DetailsResponse, GraphQlError, MediaListEntryModel, MediaListResponse,
    MediaModel, SearchResponse,
};
use self::requests::{
    AniListRequest, MediaDetailsRequest, MediaListExportRequest, MediaSearchRequest,
};

const ANILIST_BASE_URL: &str = "https://graphql.anilist.co";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(3);

// ── GraphQL クエリ定数 ──────────────────────────────────────────────────────

const SEARCH_QUERY: &str = r#"
query ($search: String, $type: MediaType, $page: Int, $perPage: Int, $genre_in: [String], $season: MediaSeason, $seasonYear: Int) {
  Page(page: $page, perPage: $perPage) {
    media(search: $search, type: $type, genre_in: $genre_in, season: $season, seasonYear: $seasonYear) {
      id type status episodes chapters genres averageScore season seasonYear
      title { romaji native english }
      coverImage { large }
    }
  }
}"#;

const DETAILS_QUERY: &str = r#"
query ($id: Int) {
  Media(id: $id) {
    id type status episodes chapters genres averageScore season seasonYear
    title { romaji native english }
    coverImage { large }
  }
}"#;

const MEDIA_LIST_QUERY: &str = r#"
query ($userName: String) {
  MediaListCollection(userName: $userName, type: ANIME) {
    lists {
      entries {
        id status progress
        score(format: POINT_10_DECIMAL)
        media {
          id type status episodes chapters genres averageScore season seasonYear
          title { romaji native english }
          coverImage { large }
        }
      }
    }
  }
}"#;

// ── クライアント構造体 ────────────────────────────────────────────────────────

/// AniList API クライアント。
///
/// レート制限: 90 req / 60 秒
pub struct AniListClient {
    http: reqwest::Client,
    auth: AuthStrategy,
    rate_limit: Arc<Mutex<RateLimitState>>,
    base_url: String,
}

impl AniListClient {
    /// 本番 URL を使ってクライアントを作成する。
    pub fn new(auth: AuthStrategy) -> Result<Self, ApiError> {
        Self::new_with_base_url(auth, ANILIST_BASE_URL)
    }

    /// カスタムベース URL を指定してクライアントを作成する（主にテスト用）。
    pub fn new_with_base_url(
        auth: AuthStrategy,
        base_url: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let http = reqwest::Client::builder()
            .build()
            .map_err(|e| ApiError::Network(format!("HTTP client init failed: {e}")))?;
        Ok(Self {
            http,
            auth,
            rate_limit: Arc::new(Mutex::new(RateLimitState::new(90, 60))),
            base_url: base_url.into(),
        })
    }

    // ── 内部ヘルパー ────────────────────────────────────────────────────

    fn apply_auth_header(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.auth {
            AuthStrategy::Bearer(token) => {
                builder.header("Authorization", format!("Bearer {token}"))
            }
            _ => builder,
        }
    }

    async fn post_graphql_raw(
        &self,
        body: &serde_json::Value,
    ) -> Result<(u16, String, u64), ApiError> {
        tracing::trace!(url = %self.base_url, "AniList GraphQL request sending");

        let mut builder = self
            .http
            .post(&self.base_url)
            .timeout(REQUEST_TIMEOUT)
            .header("Content-Type", "application/json")
            .json(body);

        builder = self.apply_auth_header(builder);

        let start = Instant::now();
        let response = builder.send().await.map_err(|e| {
            if e.is_timeout() {
                ApiError::Timeout
            } else {
                ApiError::Network(e.to_string())
            }
        })?;

        let status = response.status().as_u16();
        let latency_ms = start.elapsed().as_millis() as u64;

        // AniList は GraphQL エラーでも 200 を返すことがある
        // 4xx/5xx は直接エラー
        if status >= 400 {
            let body_text = response.text().await.unwrap_or_default();
            tracing::warn!(status, "AniList HTTP error");
            return Err(ApiError::Http {
                status,
                body: body_text,
            });
        }

        let text = response
            .text()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        tracing::trace!(
            status,
            latency_ms,
            response_len = text.len(),
            "AniList response received"
        );

        Ok((status, text, latency_ms))
    }

    fn check_graphql_errors(errors: Option<Vec<GraphQlError>>) -> Result<(), ApiError> {
        if let Some(errs) = errors {
            if let Some(first) = errs.into_iter().next() {
                tracing::warn!(message = %first.message, "AniList GraphQL error in response");
                return Err(ApiError::Http {
                    status: first.status.unwrap_or(400),
                    body: first.message,
                });
            }
        }
        Ok(())
    }

    // ── 公開メソッド ─────────────────────────────────────────────────────

    /// アニメ・マンガを検索する。
    pub async fn search_media(
        &self,
        req: MediaSearchRequest,
    ) -> Result<ApiResponse<Vec<MediaModel>>, ApiError> {
        tracing::debug!(operation = "search_media", "AniList request start");
        {
            let mut rl = self.rate_limit.lock().await;
            rl.check_and_increment()?;
        }

        let variables = serde_json::json!({
            "search": req.search,
            "type": req.media_type.map(|t| t.as_str()),
            "page": req.page,
            "perPage": req.per_page,
            "genre_in": req.genre_in,
            "season": req.season.map(|s| s.as_str()),
            "seasonYear": req.season_year,
        });
        let body = serde_json::json!({ "query": SEARCH_QUERY, "variables": variables });

        let (status, text, latency_ms) = self.post_graphql_raw(&body).await?;

        let parsed: SearchResponse =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;

        Self::check_graphql_errors(parsed.errors)?;

        let media: Vec<MediaModel> = parsed
            .data
            .map(|d| d.page.media.into_iter().map(MediaModel::from).collect())
            .unwrap_or_default();

        tracing::debug!(
            status,
            latency_ms,
            result_count = media.len(),
            "AniList search_media OK"
        );

        Ok(ApiResponse {
            request: RequestResult {
                status,
                url: self.base_url.clone(),
                latency_ms,
            },
            raw: RawData::Json(text),
            model: media,
        })
    }

    /// 指定 ID のアニメ・マンガ詳細を取得する。
    pub async fn get_media_details(
        &self,
        req: MediaDetailsRequest,
    ) -> Result<ApiResponse<MediaModel>, ApiError> {
        tracing::debug!(
            operation = "get_media_details",
            id = req.id,
            "AniList request start"
        );
        {
            let mut rl = self.rate_limit.lock().await;
            rl.check_and_increment()?;
        }

        let variables = serde_json::json!({ "id": req.id });
        let body = serde_json::json!({ "query": DETAILS_QUERY, "variables": variables });

        let (status, text, latency_ms) = self.post_graphql_raw(&body).await?;

        let parsed: DetailsResponse =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;

        Self::check_graphql_errors(parsed.errors)?;

        let media = parsed
            .data
            .map(|d| MediaModel::from(d.media))
            .ok_or_else(|| ApiError::Parse("Missing 'data.Media' in response".to_string()))?;

        tracing::debug!(status, latency_ms, "AniList get_media_details OK");

        Ok(ApiResponse {
            request: RequestResult {
                status,
                url: self.base_url.clone(),
                latency_ms,
            },
            raw: RawData::Json(text),
            model: media,
        })
    }

    /// ユーザーのメディアリスト（視聴履歴）を取得する（Bearer 認証必須）。
    pub async fn export_media_list(
        &self,
        req: MediaListExportRequest,
    ) -> Result<ApiResponse<Vec<MediaListEntryModel>>, ApiError> {
        tracing::debug!(operation = "export_media_list", user_name = %req.user_name, "AniList request start");
        {
            let mut rl = self.rate_limit.lock().await;
            rl.check_and_increment()?;
        }

        let variables = serde_json::json!({ "userName": req.user_name });
        let body = serde_json::json!({ "query": MEDIA_LIST_QUERY, "variables": variables });

        let (status, text, latency_ms) = self.post_graphql_raw(&body).await?;

        let parsed: MediaListResponse =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;

        Self::check_graphql_errors(parsed.errors)?;

        let entries: Vec<MediaListEntryModel> = parsed
            .data
            .map(|d| {
                d.collection
                    .lists
                    .into_iter()
                    .flat_map(|g| g.entries)
                    .map(|e| MediaListEntryModel {
                        id: e.id,
                        status: e.status,
                        score: e.score,
                        progress: e.progress,
                        media: MediaModel::from(e.media),
                    })
                    .collect()
            })
            .unwrap_or_default();

        tracing::debug!(
            status,
            latency_ms,
            result_count = entries.len(),
            "AniList export_media_list OK"
        );

        Ok(ApiResponse {
            request: RequestResult {
                status,
                url: self.base_url.clone(),
                latency_ms,
            },
            raw: RawData::Json(text),
            model: entries,
        })
    }
}

// ── ApiClient trait impl ──────────────────────────────────────────────────

impl ApiClient for AniListClient {
    type Request = AniListRequest;
    type Model = AniListModel;

    async fn execute(
        &self,
        request: AniListRequest,
    ) -> Result<ApiResponse<AniListModel>, ApiError> {
        match request {
            AniListRequest::SearchMedia(req) => {
                let resp = self.search_media(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: AniListModel::MediaList(resp.model),
                })
            }
            AniListRequest::GetMediaDetails(req) => {
                let resp = self.get_media_details(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: AniListModel::MediaDetail(Box::new(resp.model)),
                })
            }
            AniListRequest::ExportMediaList(req) => {
                let resp = self.export_media_list(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: AniListModel::MediaListEntries(resp.model),
                })
            }
        }
    }
}
