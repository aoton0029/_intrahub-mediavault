pub mod models;
pub mod requests;

use std::collections::HashMap;
use std::time::Duration;

use crate::auth::AuthStrategy;
use crate::error::ApiError;
use crate::response::{ApiResponse, RawData, RequestResult};
use crate::traits::ApiClient;

use self::models::{
    AppDetailsEntry, GetOwnedGamesWrapper, SteamAppDetailsModel, SteamModel, SteamOwnedGamesModel,
    SteamStoreSearchResultModel, StoreSearchWrapper,
};
use self::requests::{
    GetOwnedGamesRequest, SteamAppDetailsRequest, SteamRequest, SteamStoreSearchRequest,
};

const STEAM_API_BASE_URL: &str = "https://api.steampowered.com";
const STEAM_STORE_BASE_URL: &str = "https://store.steampowered.com";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(3);

/// Steam Web API クライアント。
///
/// `GetOwnedGames` には API Key 必須。ストア系は認証不要。
/// レート制限プリセットなし。
pub struct SteamClient {
    http: reqwest::Client,
    auth: AuthStrategy,
    api_base_url: String,
    store_base_url: String,
}

impl SteamClient {
    /// 本番 URL を使ってクライアントを作成する。
    pub fn new(auth: AuthStrategy) -> Result<Self, ApiError> {
        Self::new_with_base_urls(auth, STEAM_API_BASE_URL, STEAM_STORE_BASE_URL)
    }

    /// カスタム URL を指定してクライアントを作成する（主にテスト用）。
    pub fn new_with_base_urls(
        auth: AuthStrategy,
        api_base_url: impl Into<String>,
        store_base_url: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let http = reqwest::Client::builder()
            .build()
            .map_err(|e| ApiError::Network(format!("HTTP client init failed: {e}")))?;
        Ok(Self {
            http,
            auth,
            api_base_url: api_base_url.into(),
            store_base_url: store_base_url.into(),
        })
    }

    fn api_key(&self) -> Option<&str> {
        match &self.auth {
            AuthStrategy::ApiKey(key) => Some(key.as_str()),
            _ => None,
        }
    }

    // ── 内部ヘルパー ────────────────────────────────────────────────────

    async fn get_json(&self, url: &str) -> Result<(u16, String, u64), ApiError> {
        tracing::debug!(url, "Steam request sending");
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
            tracing::warn!(status, body = %body, "Steam HTTP error");
            return Err(ApiError::Http { status, body });
        }

        let text = response
            .text()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        tracing::debug!(status, latency_ms, body = %text, "Steam response received");
        Ok((status, text, latency_ms))
    }

    // ── 公開メソッド ─────────────────────────────────────────────────────

    /// ユーザーの所持ゲーム一覧を取得する。
    pub async fn get_owned_games(
        &self,
        req: GetOwnedGamesRequest,
    ) -> Result<ApiResponse<SteamOwnedGamesModel>, ApiError> {
        let key = self.api_key().ok_or_else(|| {
            ApiError::Auth("Steam API key required for GetOwnedGames".to_string())
        })?;

        let mut url = reqwest::Url::parse(&format!(
            "{}/IPlayerService/GetOwnedGames/v0001/",
            self.api_base_url
        ))
        .map_err(|e| ApiError::Network(e.to_string()))?;

        {
            let mut params = url.query_pairs_mut();
            params.append_pair("key", key);
            params.append_pair("steamid", &req.steam_id.to_string());
            params.append_pair(
                "include_appinfo",
                if req.include_appinfo { "1" } else { "0" },
            );
            params.append_pair(
                "include_played_free_games",
                if req.include_played_free_games {
                    "1"
                } else {
                    "0"
                },
            );
            params.append_pair("format", "json");
        }

        let url_str = url.to_string();
        tracing::debug!(
            operation = "get_owned_games",
            steam_id = req.steam_id,
            "Steam request start"
        );
        let (status, text, latency_ms) = self.get_json(&url_str).await?;

        let parsed: GetOwnedGamesWrapper =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;

        let model = SteamOwnedGamesModel {
            game_count: parsed.response.game_count,
            games: parsed.response.games,
        };

        tracing::debug!(status, latency_ms, "Steam get_owned_games OK");

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

    /// アプリ詳細を取得する（store.steampowered.com/api/appdetails）。
    pub async fn get_app_details(
        &self,
        req: SteamAppDetailsRequest,
    ) -> Result<ApiResponse<SteamAppDetailsModel>, ApiError> {
        let url = format!(
            "{}/api/appdetails?appids={}",
            self.store_base_url, req.app_id
        );
        tracing::debug!(
            operation = "get_app_details",
            app_id = req.app_id,
            "Steam request start"
        );
        let (status, text, latency_ms) = self.get_json(&url).await?;

        // レスポンスは `{"<appid>": {"success": bool, "data": {...}}}` 形式
        let mut map: HashMap<String, AppDetailsEntry> =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;

        let entry = map
            .remove(&req.app_id.to_string())
            .ok_or_else(|| ApiError::Parse("App ID not found in response".to_string()))?;

        if !entry.success {
            return Err(ApiError::Http {
                status: 404,
                body: format!("App {} not found", req.app_id),
            });
        }

        let model = entry
            .data
            .ok_or_else(|| ApiError::Parse("Missing 'data' in app details response".to_string()))?;

        tracing::debug!(
            status,
            latency_ms,
            app_id = req.app_id,
            "Steam get_app_details OK"
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

    /// ストアでゲームを検索する（store.steampowered.com/api/storesearch）。
    pub async fn store_search(
        &self,
        req: SteamStoreSearchRequest,
    ) -> Result<ApiResponse<Vec<SteamStoreSearchResultModel>>, ApiError> {
        let mut url = reqwest::Url::parse(&format!("{}/api/storesearch/", self.store_base_url))
            .map_err(|e| ApiError::Network(e.to_string()))?;

        {
            let mut params = url.query_pairs_mut();
            params.append_pair("term", &req.term);
            params.append_pair("cc", "JP");
            params.append_pair("l", "english");
            if let Some(page) = req.page {
                params.append_pair("page", &page.to_string());
            }
        }

        let url_str = url.to_string();
        tracing::debug!(operation = "store_search", term = %req.term, "Steam request start");
        let (status, text, latency_ms) = self.get_json(&url_str).await?;

        let parsed: StoreSearchWrapper =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;

        tracing::debug!(
            status,
            latency_ms,
            result_count = parsed.results.len(),
            "Steam store_search OK"
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
}

// ── ApiClient trait impl ──────────────────────────────────────────────────

impl ApiClient for SteamClient {
    type Request = SteamRequest;
    type Model = SteamModel;

    async fn execute(&self, request: SteamRequest) -> Result<ApiResponse<SteamModel>, ApiError> {
        match request {
            SteamRequest::GetOwnedGames(req) => {
                let resp = self.get_owned_games(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: SteamModel::OwnedGames(resp.model),
                })
            }
            SteamRequest::GetAppDetails(req) => {
                let resp = self.get_app_details(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: SteamModel::AppDetails(resp.model),
                })
            }
            SteamRequest::StoreSearch(req) => {
                let resp = self.store_search(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: SteamModel::StoreResults(resp.model),
                })
            }
        }
    }
}
