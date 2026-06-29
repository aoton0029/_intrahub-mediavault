pub mod models;
pub mod requests;

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use crate::auth::TwitchTokenState;
use crate::error::ApiError;
use crate::rate_limit::RateLimitState;
use crate::response::{ApiResponse, RawData, RequestResult};
use crate::traits::ApiClient;

use self::models::{GameModel, IgdbModel, TwitchTokenResponse};
use self::requests::{GamesQueryRequest, IgdbEndpointRequest, IgdbRequest, IgdbSearchRequest};

const IGDB_BASE_URL: &str = "https://api.igdb.com/v4";
const TWITCH_TOKEN_URL: &str = "https://id.twitch.tv/oauth2/token";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(3);

/// IGDB API クライアント。
///
/// Twitch OAuth2 クライアント資格情報フローで自動トークン管理を行う。
/// レート制限: 4 req / 秒（非公式）
pub struct IgdbClient {
    http: reqwest::Client,
    client_id: String,
    client_secret: String,
    token_state: Arc<Mutex<Option<TwitchTokenState>>>,
    rate_limit: Arc<Mutex<RateLimitState>>,
    base_url: String,
    twitch_token_url: String,
}

impl IgdbClient {
    /// 本番 URL を使ってクライアントを作成する。
    pub fn new(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
    ) -> Result<Self, ApiError> {
        Self::new_with_urls(client_id, client_secret, IGDB_BASE_URL, TWITCH_TOKEN_URL)
    }

    /// カスタム URL を指定してクライアントを作成する（主にテスト用）。
    pub fn new_with_urls(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        base_url: impl Into<String>,
        twitch_token_url: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let http = reqwest::Client::builder()
            .build()
            .map_err(|e| ApiError::Network(format!("HTTP client init failed: {e}")))?;
        Ok(Self {
            http,
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            token_state: Arc::new(Mutex::new(None)),
            rate_limit: Arc::new(Mutex::new(RateLimitState::new(4, 1))),
            base_url: base_url.into(),
            twitch_token_url: twitch_token_url.into(),
        })
    }

    // ── Twitch トークン管理 ─────────────────────────────────────────────

    async fn fetch_twitch_token(&self) -> Result<TwitchTokenResponse, ApiError> {
        tracing::debug!("fetching Twitch OAuth2 token");
        let response = self
            .http
            .post(&self.twitch_token_url)
            .timeout(REQUEST_TIMEOUT)
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("grant_type", "client_credentials"),
            ])
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    ApiError::Timeout
                } else {
                    ApiError::Network(e.to_string())
                }
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            tracing::warn!(status, "Twitch token fetch failed");
            return Err(ApiError::Auth(format!(
                "Twitch token fetch failed: status={status}, body={body}"
            )));
        }

        let text = response
            .text()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        let token_resp = serde_json::from_str::<TwitchTokenResponse>(&text)
            .map_err(|e| ApiError::Auth(format!("Twitch token parse error: {e}")))?;
        tracing::debug!(expires_in = token_resp.expires_in, "Twitch token acquired");
        Ok(token_resp)
    }

    async fn ensure_token(&self) -> Result<String, ApiError> {
        let mut state = self.token_state.lock().await;

        if let Some(ref token) = *state {
            if token.is_valid() {
                tracing::trace!("Twitch token cache hit");
                return Ok(token.access_token.clone());
            }
        }

        tracing::debug!("Twitch token missing or expired, refreshing");
        // 取得（または再取得）
        let resp = self.fetch_twitch_token().await?;
        let access_token = resp.access_token.clone();
        // 60 秒のバッファを設ける
        let expires_in = resp.expires_in.saturating_sub(60);
        *state = Some(TwitchTokenState {
            access_token: resp.access_token,
            expires_at: Instant::now() + Duration::from_secs(expires_in),
        });

        tracing::debug!("Twitch token refreshed");
        Ok(access_token)
    }

    async fn invalidate_token(&self) {
        let mut state = self.token_state.lock().await;
        *state = None;
        tracing::debug!("Twitch token invalidated");
    }

    // ── Apicalypse リクエスト ────────────────────────────────────────────

    async fn try_apicalypse<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        query: &str,
    ) -> Result<(u16, String, u64, T), ApiError> {
        let url = format!("{}/{}", self.base_url, endpoint);
        tracing::trace!(url = %url, endpoint, "IGDB Apicalypse request sending");
        let access_token = self.ensure_token().await?;

        let start = Instant::now();
        let response = self
            .http
            .post(&url)
            .timeout(REQUEST_TIMEOUT)
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {access_token}"))
            .header("Accept", "application/json")
            .body(query.to_string())
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
            tracing::warn!(status, endpoint, "IGDB HTTP error");
            return Err(ApiError::Http { status, body });
        }

        let text = response
            .text()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        tracing::trace!(
            status,
            latency_ms,
            response_len = text.len(),
            "IGDB response received"
        );
        let model: T = serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;

        Ok((status, text, latency_ms, model))
    }

    async fn apicalypse<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        query: &str,
    ) -> Result<(u16, String, u64, T), ApiError> {
        match self.try_apicalypse(endpoint, query).await {
            // 401: トークン期限切れの可能性 → リフレッシュして 1 回再試行
            Err(ApiError::Http { status: 401, .. }) => {
                tracing::warn!(
                    endpoint,
                    "IGDB 401 received, invalidating token and retrying"
                );
                self.invalidate_token().await;
                self.try_apicalypse(endpoint, query).await
            }
            other => other,
        }
    }

    // ── 公開メソッド ─────────────────────────────────────────────────────

    /// `/games` エンドポイントに Apicalypse クエリを実行する。
    pub async fn query_games(
        &self,
        req: GamesQueryRequest,
    ) -> Result<ApiResponse<Vec<GameModel>>, ApiError> {
        {
            let mut rl = self.rate_limit.lock().await;
            rl.check_and_increment()?;
        }

        let (status, text, latency_ms, model) = self
            .apicalypse::<Vec<GameModel>>("games", &req.query)
            .await?;

        let url = format!("{}/games", self.base_url);
        tracing::debug!(status, latency_ms, "IGDB query_games OK");

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

    /// `/search` エンドポイントに横断検索を実行する。
    pub async fn search(
        &self,
        req: IgdbSearchRequest,
    ) -> Result<ApiResponse<Vec<serde_json::Value>>, ApiError> {
        {
            let mut rl = self.rate_limit.lock().await;
            rl.check_and_increment()?;
        }

        let (status, text, latency_ms, model) = self
            .apicalypse::<Vec<serde_json::Value>>("search", &req.query)
            .await?;

        let url = format!("{}/search", self.base_url);
        tracing::debug!(status, latency_ms, "IGDB search OK");

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

    /// 汎用エンドポイント（/companies, /platforms, /covers, /screenshots など）にクエリを実行する。
    pub async fn query_endpoint(
        &self,
        req: IgdbEndpointRequest,
    ) -> Result<ApiResponse<Vec<serde_json::Value>>, ApiError> {
        {
            let mut rl = self.rate_limit.lock().await;
            rl.check_and_increment()?;
        }

        let (status, text, latency_ms, model) = self
            .apicalypse::<Vec<serde_json::Value>>(&req.endpoint, &req.query)
            .await?;

        let url = format!("{}/{}", self.base_url, req.endpoint);
        tracing::debug!(status, latency_ms, endpoint = %req.endpoint, "IGDB query_endpoint OK");

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
}

// ── ApiClient trait impl ──────────────────────────────────────────────────

impl ApiClient for IgdbClient {
    type Request = IgdbRequest;
    type Model = IgdbModel;

    async fn execute(&self, request: IgdbRequest) -> Result<ApiResponse<IgdbModel>, ApiError> {
        match request {
            IgdbRequest::QueryGames(req) => {
                let resp = self.query_games(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: IgdbModel::Games(resp.model),
                })
            }
            IgdbRequest::Search(req) => {
                let resp = self.search(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: IgdbModel::SearchResults(resp.model),
                })
            }
            IgdbRequest::QueryEndpoint(req) => {
                let resp = self.query_endpoint(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: IgdbModel::EndpointResults(resp.model),
                })
            }
        }
    }
}
