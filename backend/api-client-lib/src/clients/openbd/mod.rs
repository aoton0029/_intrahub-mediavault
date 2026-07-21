pub mod models;
pub mod requests;

use std::time::Duration;

use crate::error::ApiError;
use crate::response::{ApiResponse, RawData, RequestResult};
use crate::traits::ApiClient;

use self::models::{parse_openbd_response, OpenBdItemModel, OpenBdModel};
use self::requests::{OpenBdGetRequest, OpenBdRequest};

const OPENBD_BASE_URL: &str = "https://api.openbd.jp/v1";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// OpenBD API クライアント。
///
/// 認証不要の公開API。1リクエストで複数ISBNをカンマ区切りでまとめて問い合わせられるため、
/// 大量件数のISBNもチャンク化して少ないリクエスト数で処理できる。
pub struct OpenBdClient {
    http: reqwest::Client,
    base_url: String,
}

impl OpenBdClient {
    /// 本番 URL を使ってクライアントを作成する。
    pub fn new() -> Result<Self, ApiError> {
        Self::new_with_base_url(OPENBD_BASE_URL)
    }

    /// カスタムベース URL を指定してクライアントを作成する（主にテスト用）。
    pub fn new_with_base_url(base_url: impl Into<String>) -> Result<Self, ApiError> {
        let http = reqwest::Client::builder()
            .build()
            .map_err(|e| ApiError::Network(format!("HTTP client init failed: {e}")))?;
        Ok(Self {
            http,
            base_url: base_url.into(),
        })
    }

    // ── 公開メソッド ─────────────────────────────────────────────────────

    /// ISBNをまとめて書誌取得する（GET /get?isbn=1,2,3,...）。
    pub async fn get(
        &self,
        req: OpenBdGetRequest,
    ) -> Result<ApiResponse<Vec<Option<OpenBdItemModel>>>, ApiError> {
        let url_str = format!("{}/get?isbn={}", self.base_url, req.isbns.join(","));

        tracing::debug!(operation = "get", url = %url_str, "OpenBD request sending");
        let start = std::time::Instant::now();
        let response = self
            .http
            .get(&url_str)
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
            tracing::warn!(status, body = %body, "OpenBD HTTP error");
            return Err(ApiError::Http { status, body });
        }

        let text = response
            .text()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        let model = parse_openbd_response(&text)?;

        tracing::debug!(status, latency_ms, count = model.len(), "OpenBD get OK");

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

impl ApiClient for OpenBdClient {
    type Request = OpenBdRequest;
    type Model = OpenBdModel;

    async fn execute(&self, request: OpenBdRequest) -> Result<ApiResponse<OpenBdModel>, ApiError> {
        match request {
            OpenBdRequest::Get(req) => {
                let resp = self.get(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: OpenBdModel::Items(resp.model),
                })
            }
        }
    }
}
