pub mod models;
pub mod requests;

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::error::ApiError;
use crate::rate_limit::RateLimitState;
use crate::response::{ApiResponse, RawData, RequestResult};
use crate::traits::ApiClient;

use self::models::{
    JikanAnimeModel, JikanCharacterListResponse, JikanEpisodeDetailModel,
    JikanEpisodeDetailResponse, JikanEpisodeListResponse, JikanEpisodeModel, JikanListResponse,
    JikanMangaListResponse, JikanMangaModel, JikanMangaSingleResponse, JikanModel,
    JikanPictureListResponse, JikanPictureModel, JikanProducerListResponse, JikanSingleResponse,
    JikanStaffListResponse, JikanStaffModel, JikanVideosModel, JikanVideosResponse,
};
use self::requests::{
    JikanAnimeDetailsRequest, JikanAnimeEpisodeRequest, JikanAnimeEpisodesRequest,
    JikanAnimePicturesRequest, JikanAnimeSearchRequest, JikanAnimeStaffRequest,
    JikanAnimeVideosRequest, JikanCharacterSearchRequest, JikanMangaDetailsRequest,
    JikanMangaSearchRequest, JikanProducerSearchRequest, JikanRequest, JikanSeasonRequest,
};

const JIKAN_BASE_URL: &str = "https://api.jikan.moe/v4";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Jikan API クライアント。
///
/// 認証不要の公開 API。
/// レート制限: 3 req / 秒（非公式）
pub struct JikanClient {
    http: reqwest::Client,
    rate_limit: Arc<Mutex<RateLimitState>>,
    base_url: String,
}

impl JikanClient {
    /// 本番 URL を使ってクライアントを作成する。
    pub fn new() -> Result<Self, ApiError> {
        Self::new_with_base_url(JIKAN_BASE_URL)
    }

    /// カスタムベース URL を指定してクライアントを作成する（主にテスト用）。
    pub fn new_with_base_url(base_url: impl Into<String>) -> Result<Self, ApiError> {
        let http = reqwest::Client::builder()
            .build()
            .map_err(|e| ApiError::Network(format!("HTTP client init failed: {e}")))?;
        Ok(Self {
            http,
            rate_limit: Arc::new(Mutex::new(RateLimitState::new(3, 1))),
            base_url: base_url.into(),
        })
    }

    // ── 内部ヘルパー ────────────────────────────────────────────────────

    async fn get_json(&self, url: &str) -> Result<(u16, String, u64), ApiError> {
        tracing::debug!(url, "Jikan request sending");
        let start = std::time::Instant::now();
        let response = self
            .http
            .get(url)
            .timeout(REQUEST_TIMEOUT)
            .send()
            .await
            .map_err(|e| {
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
            tracing::warn!(status, body = %body, "Jikan HTTP error");
            return Err(ApiError::Http { status, body });
        }

        let text = response
            .text()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        tracing::debug!(status, latency_ms, body = %text, "Jikan response received");
        Ok((status, text, latency_ms))
    }

    async fn check_rate_limit(&self) -> Result<(), ApiError> {
        let mut rl = self.rate_limit.lock().await;
        rl.check_and_increment()
    }

    // ── 公開メソッド ─────────────────────────────────────────────────────

    /// アニメを検索する（GET /anime）。
    pub async fn search_anime(
        &self,
        req: JikanAnimeSearchRequest,
    ) -> Result<ApiResponse<Vec<JikanAnimeModel>>, ApiError> {
        self.check_rate_limit().await?;

        let mut url = reqwest::Url::parse(&format!("{}/anime", self.base_url))
            .map_err(|e| ApiError::Network(e.to_string()))?;
        {
            let mut params = url.query_pairs_mut();
            if let Some(q) = &req.q {
                params.append_pair("q", q);
            }
            if let Some(page) = req.page {
                params.append_pair("page", &page.to_string());
            }
            if let Some(limit) = req.limit {
                params.append_pair("limit", &limit.to_string());
            }
            if let Some(t) = &req.anime_type {
                params.append_pair("type", t);
            }
            if let Some(s) = &req.status {
                params.append_pair("status", s);
            }
        }

        let url_str = url.to_string();
        tracing::debug!(operation = "search_anime", "Jikan request start");
        let (status, text, latency_ms) = self.get_json(&url_str).await?;

        let parsed: JikanListResponse =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;
        let model: Vec<JikanAnimeModel> =
            parsed.data.into_iter().map(JikanAnimeModel::from).collect();

        tracing::debug!(
            status,
            latency_ms,
            result_count = model.len(),
            "Jikan search_anime OK"
        );
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

    /// 指定 ID のアニメ詳細を取得する（GET /anime/{id}/full）。
    pub async fn get_anime_details(
        &self,
        req: JikanAnimeDetailsRequest,
    ) -> Result<ApiResponse<JikanAnimeModel>, ApiError> {
        self.check_rate_limit().await?;

        let url = format!("{}/anime/{}/full", self.base_url, req.id);
        tracing::debug!(
            operation = "get_anime_details",
            id = req.id,
            "Jikan request start"
        );
        let (status, text, latency_ms) = self.get_json(&url).await?;

        let parsed: JikanSingleResponse =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;
        let model = JikanAnimeModel::from(parsed.data);

        tracing::debug!(
            status,
            latency_ms,
            id = req.id,
            "Jikan get_anime_details OK"
        );
        Ok(ApiResponse {
            request: RequestResult {
                status,
                url,
                latency_ms,
            },
            raw: RawData::Json(text),
            model,
        })
    }

    /// 指定アニメのエピソード一覧を取得する（GET /anime/{id}/episodes）。
    pub async fn get_anime_episodes(
        &self,
        req: JikanAnimeEpisodesRequest,
    ) -> Result<ApiResponse<Vec<JikanEpisodeModel>>, ApiError> {
        self.check_rate_limit().await?;

        let mut url = reqwest::Url::parse(&format!("{}/anime/{}/episodes", self.base_url, req.id))
            .map_err(|e| ApiError::Network(e.to_string()))?;
        if let Some(page) = req.page {
            url.query_pairs_mut().append_pair("page", &page.to_string());
        }

        let url_str = url.to_string();
        tracing::debug!(
            operation = "get_anime_episodes",
            id = req.id,
            "Jikan request start"
        );
        let (status, text, latency_ms) = self.get_json(&url_str).await?;

        let parsed: JikanEpisodeListResponse =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;
        let model: Vec<JikanEpisodeModel> = parsed
            .data
            .into_iter()
            .map(JikanEpisodeModel::from)
            .collect();

        tracing::debug!(
            status,
            latency_ms,
            result_count = model.len(),
            "Jikan get_anime_episodes OK"
        );
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

    /// 指定アニメの特定エピソードを取得する（GET /anime/{id}/episodes/{episode}）。
    pub async fn get_anime_episode(
        &self,
        req: JikanAnimeEpisodeRequest,
    ) -> Result<ApiResponse<JikanEpisodeDetailModel>, ApiError> {
        self.check_rate_limit().await?;

        let url = format!(
            "{}/anime/{}/episodes/{}",
            self.base_url, req.id, req.episode
        );
        tracing::debug!(
            operation = "get_anime_episode",
            id = req.id,
            episode = req.episode,
            "Jikan request start"
        );
        let (status, text, latency_ms) = self.get_json(&url).await?;

        let parsed: JikanEpisodeDetailResponse =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;
        let model = JikanEpisodeDetailModel::from(parsed.data);

        tracing::debug!(status, latency_ms, "Jikan get_anime_episode OK");
        Ok(ApiResponse {
            request: RequestResult {
                status,
                url,
                latency_ms,
            },
            raw: RawData::Json(text),
            model,
        })
    }

    /// 指定アニメのスタッフ情報を取得する（GET /anime/{id}/staff）。
    pub async fn get_anime_staff(
        &self,
        req: JikanAnimeStaffRequest,
    ) -> Result<ApiResponse<Vec<JikanStaffModel>>, ApiError> {
        self.check_rate_limit().await?;

        let url = format!("{}/anime/{}/staff", self.base_url, req.id);
        tracing::debug!(
            operation = "get_anime_staff",
            id = req.id,
            "Jikan request start"
        );
        let (status, text, latency_ms) = self.get_json(&url).await?;

        let parsed: JikanStaffListResponse =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;
        let model: Vec<JikanStaffModel> =
            parsed.data.into_iter().map(JikanStaffModel::from).collect();

        tracing::debug!(
            status,
            latency_ms,
            result_count = model.len(),
            "Jikan get_anime_staff OK"
        );
        Ok(ApiResponse {
            request: RequestResult {
                status,
                url,
                latency_ms,
            },
            raw: RawData::Json(text),
            model,
        })
    }

    /// 指定アニメの画像一覧を取得する（GET /anime/{id}/pictures）。
    pub async fn get_anime_pictures(
        &self,
        req: JikanAnimePicturesRequest,
    ) -> Result<ApiResponse<Vec<JikanPictureModel>>, ApiError> {
        self.check_rate_limit().await?;

        let url = format!("{}/anime/{}/pictures", self.base_url, req.id);
        tracing::debug!(
            operation = "get_anime_pictures",
            id = req.id,
            "Jikan request start"
        );
        let (status, text, latency_ms) = self.get_json(&url).await?;

        let parsed: JikanPictureListResponse =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;
        let model: Vec<JikanPictureModel> = parsed
            .data
            .into_iter()
            .map(JikanPictureModel::from)
            .collect();

        tracing::debug!(
            status,
            latency_ms,
            result_count = model.len(),
            "Jikan get_anime_pictures OK"
        );
        Ok(ApiResponse {
            request: RequestResult {
                status,
                url,
                latency_ms,
            },
            raw: RawData::Json(text),
            model,
        })
    }

    /// 指定アニメの動画を取得する（GET /anime/{id}/videos）。
    pub async fn get_anime_videos(
        &self,
        req: JikanAnimeVideosRequest,
    ) -> Result<ApiResponse<JikanVideosModel>, ApiError> {
        self.check_rate_limit().await?;

        let url = format!("{}/anime/{}/videos", self.base_url, req.id);
        tracing::debug!(
            operation = "get_anime_videos",
            id = req.id,
            "Jikan request start"
        );
        let (status, text, latency_ms) = self.get_json(&url).await?;

        let parsed: JikanVideosResponse =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;
        let model = JikanVideosModel::from(parsed.data);

        tracing::debug!(status, latency_ms, "Jikan get_anime_videos OK");
        Ok(ApiResponse {
            request: RequestResult {
                status,
                url,
                latency_ms,
            },
            raw: RawData::Json(text),
            model,
        })
    }

    /// 漫画を検索する（GET /manga）。
    pub async fn search_manga(
        &self,
        req: JikanMangaSearchRequest,
    ) -> Result<ApiResponse<Vec<JikanMangaModel>>, ApiError> {
        self.check_rate_limit().await?;

        let mut url = reqwest::Url::parse(&format!("{}/manga", self.base_url))
            .map_err(|e| ApiError::Network(e.to_string()))?;
        {
            let mut params = url.query_pairs_mut();
            if let Some(q) = &req.q {
                params.append_pair("q", q);
            }
            if let Some(page) = req.page {
                params.append_pair("page", &page.to_string());
            }
            if let Some(limit) = req.limit {
                params.append_pair("limit", &limit.to_string());
            }
            if let Some(t) = &req.manga_type {
                params.append_pair("type", t);
            }
        }

        let url_str = url.to_string();
        tracing::debug!(operation = "search_manga", "Jikan request start");
        let (status, text, latency_ms) = self.get_json(&url_str).await?;

        let parsed: JikanMangaListResponse =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;
        let model: Vec<JikanMangaModel> =
            parsed.data.into_iter().map(JikanMangaModel::from).collect();

        tracing::debug!(
            status,
            latency_ms,
            result_count = model.len(),
            "Jikan search_manga OK"
        );
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

    /// 指定 ID の漫画詳細を取得する（GET /manga/{id}/full）。
    pub async fn get_manga_details(
        &self,
        req: JikanMangaDetailsRequest,
    ) -> Result<ApiResponse<JikanMangaModel>, ApiError> {
        self.check_rate_limit().await?;

        let url = format!("{}/manga/{}/full", self.base_url, req.id);
        tracing::debug!(
            operation = "get_manga_details",
            id = req.id,
            "Jikan request start"
        );
        let (status, text, latency_ms) = self.get_json(&url).await?;

        let parsed: JikanMangaSingleResponse =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;
        let model = JikanMangaModel::from(parsed.data);

        tracing::debug!(
            status,
            latency_ms,
            id = req.id,
            "Jikan get_manga_details OK"
        );
        Ok(ApiResponse {
            request: RequestResult {
                status,
                url,
                latency_ms,
            },
            raw: RawData::Json(text),
            model,
        })
    }

    /// キャラクターを検索する（GET /characters）。
    pub async fn search_characters(
        &self,
        req: JikanCharacterSearchRequest,
    ) -> Result<ApiResponse<Vec<models::JikanCharacterModel>>, ApiError> {
        self.check_rate_limit().await?;

        let mut url = reqwest::Url::parse(&format!("{}/characters", self.base_url))
            .map_err(|e| ApiError::Network(e.to_string()))?;
        {
            let mut params = url.query_pairs_mut();
            if let Some(q) = &req.q {
                params.append_pair("q", q);
            }
            if let Some(page) = req.page {
                params.append_pair("page", &page.to_string());
            }
        }

        let url_str = url.to_string();
        tracing::debug!(operation = "search_characters", "Jikan request start");
        let (status, text, latency_ms) = self.get_json(&url_str).await?;

        let parsed: JikanCharacterListResponse =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;
        let model: Vec<models::JikanCharacterModel> = parsed
            .data
            .into_iter()
            .map(models::JikanCharacterModel::from)
            .collect();

        tracing::debug!(
            status,
            latency_ms,
            result_count = model.len(),
            "Jikan search_characters OK"
        );
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

    /// プロデューサーを検索する（GET /producers）。
    pub async fn search_producers(
        &self,
        req: JikanProducerSearchRequest,
    ) -> Result<ApiResponse<Vec<models::JikanProducerModel>>, ApiError> {
        self.check_rate_limit().await?;

        let mut url = reqwest::Url::parse(&format!("{}/producers", self.base_url))
            .map_err(|e| ApiError::Network(e.to_string()))?;
        {
            let mut params = url.query_pairs_mut();
            if let Some(q) = &req.q {
                params.append_pair("q", q);
            }
            if let Some(page) = req.page {
                params.append_pair("page", &page.to_string());
            }
        }

        let url_str = url.to_string();
        tracing::debug!(operation = "search_producers", "Jikan request start");
        let (status, text, latency_ms) = self.get_json(&url_str).await?;

        let parsed: JikanProducerListResponse =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;
        let model: Vec<models::JikanProducerModel> = parsed
            .data
            .into_iter()
            .map(models::JikanProducerModel::from)
            .collect();

        tracing::debug!(
            status,
            latency_ms,
            result_count = model.len(),
            "Jikan search_producers OK"
        );
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

    /// 指定年・季節のアニメ一覧を取得する（GET /seasons/{year}/{season}）。
    pub async fn get_season_anime(
        &self,
        req: JikanSeasonRequest,
    ) -> Result<ApiResponse<Vec<JikanAnimeModel>>, ApiError> {
        self.check_rate_limit().await?;

        let mut url = reqwest::Url::parse(&format!(
            "{}/seasons/{}/{}",
            self.base_url, req.year, req.season
        ))
        .map_err(|e| ApiError::Network(e.to_string()))?;
        if let Some(page) = req.page {
            url.query_pairs_mut().append_pair("page", &page.to_string());
        }

        let url_str = url.to_string();
        tracing::debug!(operation = "get_season_anime", year = req.year, season = %req.season, "Jikan request start");
        let (status, text, latency_ms) = self.get_json(&url_str).await?;

        let parsed: JikanListResponse =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;
        let model: Vec<JikanAnimeModel> =
            parsed.data.into_iter().map(JikanAnimeModel::from).collect();

        tracing::debug!(
            status,
            latency_ms,
            result_count = model.len(),
            "Jikan get_season_anime OK"
        );
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

impl ApiClient for JikanClient {
    type Request = JikanRequest;
    type Model = JikanModel;

    async fn execute(&self, request: JikanRequest) -> Result<ApiResponse<JikanModel>, ApiError> {
        match request {
            JikanRequest::SearchAnime(req) => {
                let resp = self.search_anime(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: JikanModel::SearchResults(resp.model),
                })
            }
            JikanRequest::GetAnimeDetails(req) => {
                let resp = self.get_anime_details(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: JikanModel::AnimeDetails(resp.model),
                })
            }
            JikanRequest::GetAnimeEpisodes(req) => {
                let resp = self.get_anime_episodes(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: JikanModel::EpisodeList(resp.model),
                })
            }
            JikanRequest::GetAnimeEpisode(req) => {
                let resp = self.get_anime_episode(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: JikanModel::EpisodeDetail(resp.model),
                })
            }
            JikanRequest::GetAnimeStaff(req) => {
                let resp = self.get_anime_staff(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: JikanModel::StaffList(resp.model),
                })
            }
            JikanRequest::GetAnimePictures(req) => {
                let resp = self.get_anime_pictures(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: JikanModel::PictureList(resp.model),
                })
            }
            JikanRequest::GetAnimeVideos(req) => {
                let resp = self.get_anime_videos(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: JikanModel::Videos(resp.model),
                })
            }
            JikanRequest::SearchManga(req) => {
                let resp = self.search_manga(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: JikanModel::MangaSearchResults(resp.model),
                })
            }
            JikanRequest::GetMangaDetails(req) => {
                let resp = self.get_manga_details(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: JikanModel::MangaDetails(resp.model),
                })
            }
            JikanRequest::SearchCharacters(req) => {
                let resp = self.search_characters(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: JikanModel::CharacterSearchResults(resp.model),
                })
            }
            JikanRequest::SearchProducers(req) => {
                let resp = self.search_producers(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: JikanModel::ProducerSearchResults(resp.model),
                })
            }
            JikanRequest::GetSeasonAnime(req) => {
                let resp = self.get_season_anime(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: JikanModel::SeasonAnime(resp.model),
                })
            }
        }
    }
}
