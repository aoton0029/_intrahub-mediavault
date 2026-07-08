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

use self::models::{
    AnnictModel, CastModel, CastsResponse, CharacterModel, CharactersResponse, EpisodeModel,
    EpisodesResponse, OrganizationModel, OrganizationsResponse, PeopleResponse, PersonModel,
    SeriesModel, SeriesResponse, StaffModel, StaffsResponse, WorkModel, WorksResponse,
};
use self::requests::{
    AnnictRequest, ListCastsRequest, ListCharactersRequest, ListEpisodesRequest,
    ListOrganizationsRequest, ListPeopleRequest, ListSeriesRequest, ListStaffsRequest,
    ListWorksRequest,
};

const ANNICT_BASE_URL: &str = "https://api.annict.com";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(3);

/// Annict API クライアント。
///
/// アクセストークン（クエリパラメータ `access_token` または Bearer トークン）で認証する。
/// レート制限: 保守的に 60 req / 60 秒として扱う（公式のレート上限値は非公開）。
pub struct AnnictClient {
    http: reqwest::Client,
    auth: AuthStrategy,
    rate_limit: Arc<Mutex<RateLimitState>>,
    base_url: String,
}

impl AnnictClient {
    /// 本番 URL を使ってクライアントを作成する。
    pub fn new(auth: AuthStrategy) -> Result<Self, ApiError> {
        Self::new_with_base_url(auth, ANNICT_BASE_URL)
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
            rate_limit: Arc::new(Mutex::new(RateLimitState::new(60, 60))),
            base_url: base_url.into(),
        })
    }

    // ── 内部ヘルパー ────────────────────────────────────────────────────

    fn apply_auth(&self, mut url: reqwest::Url) -> (reqwest::Url, Option<String>) {
        match &self.auth {
            AuthStrategy::ApiKey(token) => {
                url.query_pairs_mut().append_pair("access_token", token);
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
        tracing::debug!(url = %url_str, "Annict request sending");

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
            tracing::warn!(status, body = %body, "Annict HTTP error");
            return Err(ApiError::Http { status, body });
        }

        let text = response
            .text()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        tracing::debug!(status, latency_ms, body = %text, "Annict response received");
        Ok((status, text, latency_ms))
    }

    fn build_url(&self, path: &str) -> Result<reqwest::Url, ApiError> {
        reqwest::Url::parse(&format!("{}{}", self.base_url, path))
            .map_err(|e| ApiError::Network(e.to_string()))
    }

    // ── 公開メソッド ─────────────────────────────────────────────────────

    /// 作品一覧を取得する（GET /v1/works）。
    pub async fn list_works(
        &self,
        req: ListWorksRequest,
    ) -> Result<ApiResponse<Vec<WorkModel>>, ApiError> {
        let mut url = self.build_url("/v1/works")?;
        {
            let mut p = url.query_pairs_mut();
            if let Some(v) = &req.fields {
                p.append_pair("fields", v);
            }
            if let Some(v) = req.page {
                p.append_pair("page", &v.to_string());
            }
            if let Some(v) = req.per_page {
                p.append_pair("per_page", &v.to_string());
            }
            if let Some(v) = &req.sort_id {
                p.append_pair("sort_id", v);
            }
            if let Some(v) = &req.filter_ids {
                p.append_pair("filter_ids", v);
            }
            if let Some(v) = &req.filter_season {
                p.append_pair("filter_season", v);
            }
            if let Some(v) = &req.filter_title {
                p.append_pair("filter_title", v);
            }
            if let Some(v) = &req.sort_season {
                p.append_pair("sort_season", v);
            }
            if let Some(v) = &req.sort_watchers_count {
                p.append_pair("sort_watchers_count", v);
            }
        }

        let url_str = url.to_string();
        tracing::debug!(operation = "list_works", "Annict request start");
        let (status, text, latency_ms) = self.get_json(url).await?;

        let parsed: WorksResponse =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;

        tracing::debug!(
            status,
            latency_ms,
            result_count = parsed.works.len(),
            "Annict list_works OK"
        );

        Ok(ApiResponse {
            request: RequestResult {
                status,
                url: url_str,
                latency_ms,
            },
            raw: RawData::Json(text),
            model: parsed.works,
        })
    }

    /// エピソード一覧を取得する（GET /v1/episodes）。
    pub async fn list_episodes(
        &self,
        req: ListEpisodesRequest,
    ) -> Result<ApiResponse<Vec<EpisodeModel>>, ApiError> {
        let mut url = self.build_url("/v1/episodes")?;
        {
            let mut p = url.query_pairs_mut();
            if let Some(v) = &req.fields {
                p.append_pair("fields", v);
            }
            if let Some(v) = req.page {
                p.append_pair("page", &v.to_string());
            }
            if let Some(v) = req.per_page {
                p.append_pair("per_page", &v.to_string());
            }
            if let Some(v) = &req.sort_id {
                p.append_pair("sort_id", v);
            }
            if let Some(v) = &req.filter_ids {
                p.append_pair("filter_ids", v);
            }
            if let Some(v) = req.filter_work_id {
                p.append_pair("filter_work_id", &v.to_string());
            }
            if let Some(v) = &req.sort_sort_number {
                p.append_pair("sort_sort_number", v);
            }
        }

        let url_str = url.to_string();
        tracing::debug!(operation = "list_episodes", "Annict request start");
        let (status, text, latency_ms) = self.get_json(url).await?;

        let parsed: EpisodesResponse =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;

        tracing::debug!(
            status,
            latency_ms,
            result_count = parsed.episodes.len(),
            "Annict list_episodes OK"
        );

        Ok(ApiResponse {
            request: RequestResult {
                status,
                url: url_str,
                latency_ms,
            },
            raw: RawData::Json(text),
            model: parsed.episodes,
        })
    }

    /// シリーズ一覧を取得する（GET /v1/series）。
    pub async fn list_series(
        &self,
        req: ListSeriesRequest,
    ) -> Result<ApiResponse<Vec<SeriesModel>>, ApiError> {
        let mut url = self.build_url("/v1/series")?;
        {
            let mut p = url.query_pairs_mut();
            if let Some(v) = &req.fields {
                p.append_pair("fields", v);
            }
            if let Some(v) = req.page {
                p.append_pair("page", &v.to_string());
            }
            if let Some(v) = req.per_page {
                p.append_pair("per_page", &v.to_string());
            }
            if let Some(v) = &req.sort_id {
                p.append_pair("sort_id", v);
            }
            if let Some(v) = &req.filter_ids {
                p.append_pair("filter_ids", v);
            }
            if let Some(v) = &req.filter_name {
                p.append_pair("filter_name", v);
            }
        }

        let url_str = url.to_string();
        tracing::debug!(operation = "list_series", "Annict request start");
        let (status, text, latency_ms) = self.get_json(url).await?;

        let parsed: SeriesResponse =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;

        tracing::debug!(
            status,
            latency_ms,
            result_count = parsed.series.len(),
            "Annict list_series OK"
        );

        Ok(ApiResponse {
            request: RequestResult {
                status,
                url: url_str,
                latency_ms,
            },
            raw: RawData::Json(text),
            model: parsed.series,
        })
    }

    /// キャラクター一覧を取得する（GET /v1/characters）。
    pub async fn list_characters(
        &self,
        req: ListCharactersRequest,
    ) -> Result<ApiResponse<Vec<CharacterModel>>, ApiError> {
        let mut url = self.build_url("/v1/characters")?;
        {
            let mut p = url.query_pairs_mut();
            if let Some(v) = &req.fields {
                p.append_pair("fields", v);
            }
            if let Some(v) = req.page {
                p.append_pair("page", &v.to_string());
            }
            if let Some(v) = req.per_page {
                p.append_pair("per_page", &v.to_string());
            }
            if let Some(v) = &req.sort_id {
                p.append_pair("sort_id", v);
            }
            if let Some(v) = &req.filter_ids {
                p.append_pair("filter_ids", v);
            }
            if let Some(v) = &req.filter_name {
                p.append_pair("filter_name", v);
            }
        }

        let url_str = url.to_string();
        tracing::debug!(operation = "list_characters", "Annict request start");
        let (status, text, latency_ms) = self.get_json(url).await?;

        let parsed: CharactersResponse =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;

        tracing::debug!(
            status,
            latency_ms,
            result_count = parsed.characters.len(),
            "Annict list_characters OK"
        );

        Ok(ApiResponse {
            request: RequestResult {
                status,
                url: url_str,
                latency_ms,
            },
            raw: RawData::Json(text),
            model: parsed.characters,
        })
    }

    /// 人物一覧を取得する（GET /v1/people）。
    pub async fn list_people(
        &self,
        req: ListPeopleRequest,
    ) -> Result<ApiResponse<Vec<PersonModel>>, ApiError> {
        let mut url = self.build_url("/v1/people")?;
        {
            let mut p = url.query_pairs_mut();
            if let Some(v) = &req.fields {
                p.append_pair("fields", v);
            }
            if let Some(v) = req.page {
                p.append_pair("page", &v.to_string());
            }
            if let Some(v) = req.per_page {
                p.append_pair("per_page", &v.to_string());
            }
            if let Some(v) = &req.sort_id {
                p.append_pair("sort_id", v);
            }
            if let Some(v) = &req.filter_ids {
                p.append_pair("filter_ids", v);
            }
            if let Some(v) = &req.filter_name {
                p.append_pair("filter_name", v);
            }
        }

        let url_str = url.to_string();
        tracing::debug!(operation = "list_people", "Annict request start");
        let (status, text, latency_ms) = self.get_json(url).await?;

        let parsed: PeopleResponse =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;

        tracing::debug!(
            status,
            latency_ms,
            result_count = parsed.people.len(),
            "Annict list_people OK"
        );

        Ok(ApiResponse {
            request: RequestResult {
                status,
                url: url_str,
                latency_ms,
            },
            raw: RawData::Json(text),
            model: parsed.people,
        })
    }

    /// 団体一覧を取得する（GET /v1/organizations）。
    pub async fn list_organizations(
        &self,
        req: ListOrganizationsRequest,
    ) -> Result<ApiResponse<Vec<OrganizationModel>>, ApiError> {
        let mut url = self.build_url("/v1/organizations")?;
        {
            let mut p = url.query_pairs_mut();
            if let Some(v) = &req.fields {
                p.append_pair("fields", v);
            }
            if let Some(v) = req.page {
                p.append_pair("page", &v.to_string());
            }
            if let Some(v) = req.per_page {
                p.append_pair("per_page", &v.to_string());
            }
            if let Some(v) = &req.sort_id {
                p.append_pair("sort_id", v);
            }
            if let Some(v) = &req.filter_ids {
                p.append_pair("filter_ids", v);
            }
            if let Some(v) = &req.filter_name {
                p.append_pair("filter_name", v);
            }
        }

        let url_str = url.to_string();
        tracing::debug!(operation = "list_organizations", "Annict request start");
        let (status, text, latency_ms) = self.get_json(url).await?;

        let parsed: OrganizationsResponse =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;

        tracing::debug!(
            status,
            latency_ms,
            result_count = parsed.organizations.len(),
            "Annict list_organizations OK"
        );

        Ok(ApiResponse {
            request: RequestResult {
                status,
                url: url_str,
                latency_ms,
            },
            raw: RawData::Json(text),
            model: parsed.organizations,
        })
    }

    /// キャスト一覧を取得する（GET /v1/casts）。
    pub async fn list_casts(
        &self,
        req: ListCastsRequest,
    ) -> Result<ApiResponse<Vec<CastModel>>, ApiError> {
        let mut url = self.build_url("/v1/casts")?;
        {
            let mut p = url.query_pairs_mut();
            if let Some(v) = &req.fields {
                p.append_pair("fields", v);
            }
            if let Some(v) = req.page {
                p.append_pair("page", &v.to_string());
            }
            if let Some(v) = req.per_page {
                p.append_pair("per_page", &v.to_string());
            }
            if let Some(v) = &req.sort_id {
                p.append_pair("sort_id", v);
            }
            if let Some(v) = &req.filter_ids {
                p.append_pair("filter_ids", v);
            }
            if let Some(v) = req.filter_work_id {
                p.append_pair("filter_work_id", &v.to_string());
            }
            if let Some(v) = &req.sort_sort_number {
                p.append_pair("sort_sort_number", v);
            }
        }

        let url_str = url.to_string();
        tracing::debug!(operation = "list_casts", "Annict request start");
        let (status, text, latency_ms) = self.get_json(url).await?;

        let parsed: CastsResponse =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;

        tracing::debug!(
            status,
            latency_ms,
            result_count = parsed.casts.len(),
            "Annict list_casts OK"
        );

        Ok(ApiResponse {
            request: RequestResult {
                status,
                url: url_str,
                latency_ms,
            },
            raw: RawData::Json(text),
            model: parsed.casts,
        })
    }

    /// スタッフ一覧を取得する（GET /v1/staffs）。
    pub async fn list_staffs(
        &self,
        req: ListStaffsRequest,
    ) -> Result<ApiResponse<Vec<StaffModel>>, ApiError> {
        let mut url = self.build_url("/v1/staffs")?;
        {
            let mut p = url.query_pairs_mut();
            if let Some(v) = &req.fields {
                p.append_pair("fields", v);
            }
            if let Some(v) = req.page {
                p.append_pair("page", &v.to_string());
            }
            if let Some(v) = req.per_page {
                p.append_pair("per_page", &v.to_string());
            }
            if let Some(v) = &req.sort_id {
                p.append_pair("sort_id", v);
            }
            if let Some(v) = &req.filter_ids {
                p.append_pair("filter_ids", v);
            }
            if let Some(v) = req.filter_work_id {
                p.append_pair("filter_work_id", &v.to_string());
            }
            if let Some(v) = &req.sort_sort_number {
                p.append_pair("sort_sort_number", v);
            }
        }

        let url_str = url.to_string();
        tracing::debug!(operation = "list_staffs", "Annict request start");
        let (status, text, latency_ms) = self.get_json(url).await?;

        let parsed: StaffsResponse =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;

        tracing::debug!(
            status,
            latency_ms,
            result_count = parsed.staffs.len(),
            "Annict list_staffs OK"
        );

        Ok(ApiResponse {
            request: RequestResult {
                status,
                url: url_str,
                latency_ms,
            },
            raw: RawData::Json(text),
            model: parsed.staffs,
        })
    }
}

// ── ApiClient trait impl ──────────────────────────────────────────────────

impl ApiClient for AnnictClient {
    type Request = AnnictRequest;
    type Model = AnnictModel;

    async fn execute(&self, request: AnnictRequest) -> Result<ApiResponse<AnnictModel>, ApiError> {
        match request {
            AnnictRequest::ListWorks(req) => {
                let resp = self.list_works(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: AnnictModel::Works(resp.model),
                })
            }
            AnnictRequest::ListEpisodes(req) => {
                let resp = self.list_episodes(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: AnnictModel::Episodes(resp.model),
                })
            }
            AnnictRequest::ListSeries(req) => {
                let resp = self.list_series(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: AnnictModel::Series(resp.model),
                })
            }
            AnnictRequest::ListCharacters(req) => {
                let resp = self.list_characters(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: AnnictModel::Characters(resp.model),
                })
            }
            AnnictRequest::ListPeople(req) => {
                let resp = self.list_people(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: AnnictModel::People(resp.model),
                })
            }
            AnnictRequest::ListOrganizations(req) => {
                let resp = self.list_organizations(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: AnnictModel::Organizations(resp.model),
                })
            }
            AnnictRequest::ListCasts(req) => {
                let resp = self.list_casts(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: AnnictModel::Casts(resp.model),
                })
            }
            AnnictRequest::ListStaffs(req) => {
                let resp = self.list_staffs(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: AnnictModel::Staffs(resp.model),
                })
            }
        }
    }
}
