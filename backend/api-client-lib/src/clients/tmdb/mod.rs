pub mod models;
pub mod requests;

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::auth::AuthStrategy;
use crate::error::ApiError;
use crate::rate_limit::RateLimitState;
use crate::response::{ApiResponse, RawData, RequestResult};
use crate::traits::ApiClient;

use self::models::{MovieModel, TmdbModel, TmdbPagedResults, TvModel, TvSeasonModel};
use self::requests::{
    MovieDetailsRequest, SearchMovieRequest, SearchTvRequest, TmdbRequest, TvSeasonRequest,
    TvSeriesRequest,
};

const TMDB_BASE_URL: &str = "https://api.themoviedb.org/3";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(3);

/// TMDb API クライアント。
///
/// ApiKey（クエリパラメータ）または Bearer トークンで認証する。
/// レート制限: 40 req / 10 秒
pub struct TmdbClient {
    http: reqwest::Client,
    auth: AuthStrategy,
    rate_limit: Arc<Mutex<RateLimitState>>,
    base_url: String,
}

impl TmdbClient {
    /// 本番 URL を使ってクライアントを作成する。
    pub fn new(auth: AuthStrategy) -> Result<Self, ApiError> {
        Self::new_with_base_url(auth, TMDB_BASE_URL)
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
            rate_limit: Arc::new(Mutex::new(RateLimitState::new(40, 10))),
            base_url: base_url.into(),
        })
    }

    // ── 内部ヘルパー ────────────────────────────────────────────────────

    fn apply_auth(&self, mut url: reqwest::Url) -> (reqwest::Url, Option<String>) {
        match &self.auth {
            AuthStrategy::ApiKey(key) => {
                url.query_pairs_mut().append_pair("api_key", key);
                (url, None)
            }
            AuthStrategy::Bearer(token) => (url, Some(format!("Bearer {token}"))),
            _ => (url, None),
        }
    }

    async fn get_json(&self, url: reqwest::Url) -> Result<(u16, String, u64), ApiError> {
        {
            let mut rl = self.rate_limit.lock().await;
            rl.check_and_increment()?;
        }

        let (url_with_auth, bearer) = self.apply_auth(url);
        let url_str = url_with_auth.to_string();
        tracing::trace!(url = %url_str, "TMDb GET request sending");

        let mut builder = self.http.get(&url_str).timeout(REQUEST_TIMEOUT);
        if let Some(token) = bearer {
            builder = builder.header("Authorization", token);
        }

        let start = std::time::Instant::now();
        let response = builder.send().await.map_err(|e| {
            if e.is_timeout() {
                ApiError::Timeout
            } else {
                ApiError::Network(e.to_string())
            }
        })?;

        let status = response.status().as_u16();
        let latency_ms = start.elapsed().as_millis() as u64;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            tracing::warn!(status, "TMDb HTTP error");
            return Err(ApiError::Http { status, body });
        }

        let text = response
            .text()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        tracing::trace!(status, latency_ms, "TMDb response received");
        Ok((status, text, latency_ms))
    }

    fn build_url(&self, path: &str) -> Result<reqwest::Url, ApiError> {
        reqwest::Url::parse(&format!("{}{}", self.base_url, path))
            .map_err(|e| ApiError::Network(e.to_string()))
    }

    // ── 公開メソッド ─────────────────────────────────────────────────────

    /// 映画を検索する（GET /search/movie）。
    pub async fn search_movie(
        &self,
        req: SearchMovieRequest,
    ) -> Result<ApiResponse<Vec<MovieModel>>, ApiError> {
        let mut url = self.build_url("/search/movie")?;
        {
            let mut p = url.query_pairs_mut();
            p.append_pair("query", &req.query);
            if let Some(lang) = &req.language {
                p.append_pair("language", lang);
            }
            if let Some(page) = req.page {
                p.append_pair("page", &page.to_string());
            }
        }

        let url_str = url.to_string();
        tracing::debug!(operation = "search_movie", "TMDb request start");
        let (status, text, latency_ms) = self.get_json(url).await?;

        let parsed: TmdbPagedResults<MovieModel> =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;

        tracing::debug!(
            status,
            latency_ms,
            result_count = parsed.results.len(),
            "TMDb search_movie OK"
        );

        Ok(ApiResponse {
            request: RequestResult {
                status,
                url: url_str,
                latency_ms,
            },
            raw: RawData::Json(text),
            model: parsed.results,
        })
    }

    /// 映画詳細を取得する（GET /movie/{movie_id}）。
    pub async fn get_movie_details(
        &self,
        req: MovieDetailsRequest,
    ) -> Result<ApiResponse<MovieModel>, ApiError> {
        let mut url = self.build_url(&format!("/movie/{}", req.movie_id))?;
        if let Some(lang) = &req.language {
            url.query_pairs_mut().append_pair("language", lang);
        }

        let url_str = url.to_string();
        tracing::debug!(
            operation = "get_movie_details",
            movie_id = req.movie_id,
            "TMDb request start"
        );
        let (status, text, latency_ms) = self.get_json(url).await?;

        let model: MovieModel =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;

        tracing::debug!(status, latency_ms, "TMDb get_movie_details OK");

        Ok(ApiResponse {
            request: RequestResult {
                status,
                url: url_str,
                latency_ms,
            },
            raw: RawData::Json(text),
            model,
        })
    }

    /// TV シリーズを検索する（GET /search/tv）。
    pub async fn search_tv(
        &self,
        req: SearchTvRequest,
    ) -> Result<ApiResponse<Vec<TvModel>>, ApiError> {
        let mut url = self.build_url("/search/tv")?;
        {
            let mut p = url.query_pairs_mut();
            p.append_pair("query", &req.query);
            if let Some(lang) = &req.language {
                p.append_pair("language", lang);
            }
            if let Some(page) = req.page {
                p.append_pair("page", &page.to_string());
            }
        }

        let url_str = url.to_string();
        tracing::debug!(operation = "search_tv", "TMDb request start");
        let (status, text, latency_ms) = self.get_json(url).await?;

        let parsed: TmdbPagedResults<TvModel> =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;

        tracing::debug!(
            status,
            latency_ms,
            result_count = parsed.results.len(),
            "TMDb search_tv OK"
        );

        Ok(ApiResponse {
            request: RequestResult {
                status,
                url: url_str,
                latency_ms,
            },
            raw: RawData::Json(text),
            model: parsed.results,
        })
    }

    /// TV シリーズ詳細を取得する（GET /tv/{series_id}）。
    pub async fn get_tv_series(
        &self,
        req: TvSeriesRequest,
    ) -> Result<ApiResponse<TvModel>, ApiError> {
        let mut url = self.build_url(&format!("/tv/{}", req.series_id))?;
        if let Some(lang) = &req.language {
            url.query_pairs_mut().append_pair("language", lang);
        }

        let url_str = url.to_string();
        tracing::debug!(
            operation = "get_tv_series",
            series_id = req.series_id,
            "TMDb request start"
        );
        let (status, text, latency_ms) = self.get_json(url).await?;

        let model: TvModel =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;

        tracing::debug!(status, latency_ms, "TMDb get_tv_series OK");

        Ok(ApiResponse {
            request: RequestResult {
                status,
                url: url_str,
                latency_ms,
            },
            raw: RawData::Json(text),
            model,
        })
    }

    /// TV シーズン詳細を取得する（GET /tv/{series_id}/season/{season_number}）。
    pub async fn get_tv_season(
        &self,
        req: TvSeasonRequest,
    ) -> Result<ApiResponse<TvSeasonModel>, ApiError> {
        let mut url = self.build_url(&format!(
            "/tv/{}/season/{}",
            req.series_id, req.season_number
        ))?;
        if let Some(lang) = &req.language {
            url.query_pairs_mut().append_pair("language", lang);
        }

        let url_str = url.to_string();
        tracing::debug!(
            operation = "get_tv_season",
            series_id = req.series_id,
            season_number = req.season_number,
            "TMDb request start"
        );
        let (status, text, latency_ms) = self.get_json(url).await?;

        let model: TvSeasonModel =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;

        tracing::debug!(status, latency_ms, "TMDb get_tv_season OK");

        Ok(ApiResponse {
            request: RequestResult {
                status,
                url: url_str,
                latency_ms,
            },
            raw: RawData::Json(text),
            model,
        })
    }
}

// ── ApiClient trait impl ──────────────────────────────────────────────────

impl ApiClient for TmdbClient {
    type Request = TmdbRequest;
    type Model = TmdbModel;

    async fn execute(&self, request: TmdbRequest) -> Result<ApiResponse<TmdbModel>, ApiError> {
        match request {
            TmdbRequest::SearchMovie(req) => {
                let resp = self.search_movie(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: TmdbModel::MovieList(resp.model),
                })
            }
            TmdbRequest::GetMovieDetails(req) => {
                let resp = self.get_movie_details(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: TmdbModel::MovieDetails(resp.model),
                })
            }
            TmdbRequest::SearchTv(req) => {
                let resp = self.search_tv(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: TmdbModel::TvList(resp.model),
                })
            }
            TmdbRequest::GetTvSeries(req) => {
                let resp = self.get_tv_series(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: TmdbModel::TvSeries(resp.model),
                })
            }
            TmdbRequest::GetTvSeason(req) => {
                let resp = self.get_tv_season(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: TmdbModel::TvSeason(resp.model),
                })
            }
        }
    }
}
