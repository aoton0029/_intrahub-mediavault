pub mod models;
pub mod requests;

use std::time::Duration;

use crate::error::ApiError;
use crate::response::{ApiResponse, RawData, RequestResult};
use crate::traits::ApiClient;

use self::models::{OlEditionModel, OlModel, OlSearchResponse, OlSearchResultModel, OlWorkModel};
use self::requests::{OlIsbnRequest, OlRequest, OlSearchRequest, OlWorksRequest};

const OL_BASE_URL: &str = "https://openlibrary.org";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(3);

/// Open Library API クライアント。
///
/// 認証不要の公開 API。レート制限プリセットなし。
pub struct OpenLibraryClient {
    http: reqwest::Client,
    base_url: String,
}

impl OpenLibraryClient {
    /// 本番 URL を使ってクライアントを作成する。
    pub fn new() -> Result<Self, ApiError> {
        Self::new_with_base_url(OL_BASE_URL)
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

    // ── 内部ヘルパー ────────────────────────────────────────────────────

    async fn get_json(&self, url: &str) -> Result<(u16, String, u64), ApiError> {
        tracing::trace!(url, "OpenLibrary GET request sending");
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
            tracing::warn!(status, "OpenLibrary HTTP error");
            return Err(ApiError::Http { status, body });
        }

        let text = response
            .text()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        tracing::trace!(status, latency_ms, "OpenLibrary response received");
        Ok((status, text, latency_ms))
    }

    // ── 公開メソッド ─────────────────────────────────────────────────────

    /// 書籍を全文検索する（GET /search.json）。
    pub async fn search(
        &self,
        req: OlSearchRequest,
    ) -> Result<ApiResponse<Vec<OlSearchResultModel>>, ApiError> {
        let mut url = reqwest::Url::parse(&format!("{}/search.json", self.base_url))
            .map_err(|e| ApiError::Network(e.to_string()))?;

        {
            let mut params = url.query_pairs_mut();
            params.append_pair("q", &req.q);
            if let Some(page) = req.page {
                params.append_pair("page", &page.to_string());
            }
            if let Some(limit) = req.limit {
                params.append_pair("limit", &limit.to_string());
            }
        }

        let url_str = url.to_string();
        tracing::debug!(operation = "search", "OpenLibrary request start");
        let (status, text, latency_ms) = self.get_json(&url_str).await?;

        let parsed: OlSearchResponse =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;

        tracing::debug!(status, latency_ms, result_count = parsed.docs.len(), "Open Library search OK");

        Ok(ApiResponse {
            request: RequestResult {
                status,
                url: url_str,
                latency_ms,
            },
            raw: RawData::Json(text),
            model: parsed.docs,
        })
    }

    /// ISBN から書籍 Edition を取得する（GET /isbn/{isbn}.json）。
    pub async fn get_by_isbn(
        &self,
        req: OlIsbnRequest,
    ) -> Result<ApiResponse<OlEditionModel>, ApiError> {
        let url = format!("{}/isbn/{}.json", self.base_url, req.isbn);
        tracing::debug!(operation = "get_by_isbn", isbn = %req.isbn, "OpenLibrary request start");
        let (status, text, latency_ms) = self.get_json(&url).await?;

        let model: OlEditionModel =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;

        tracing::debug!(status, latency_ms, "Open Library get_by_isbn OK");

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

    /// Works を取得する（GET /works/{olid}.json）。
    pub async fn get_works(
        &self,
        req: OlWorksRequest,
    ) -> Result<ApiResponse<OlWorkModel>, ApiError> {
        let url = format!("{}/works/{}.json", self.base_url, req.olid);
        tracing::debug!(operation = "get_works", olid = %req.olid, "OpenLibrary request start");
        let (status, text, latency_ms) = self.get_json(&url).await?;

        let model: OlWorkModel =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;

        tracing::debug!(status, latency_ms, "Open Library get_works OK");

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

impl ApiClient for OpenLibraryClient {
    type Request = OlRequest;
    type Model = OlModel;

    async fn execute(&self, request: OlRequest) -> Result<ApiResponse<OlModel>, ApiError> {
        match request {
            OlRequest::Search(req) => {
                let resp = self.search(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: OlModel::SearchResults(resp.model),
                })
            }
            OlRequest::GetByIsbn(req) => {
                let resp = self.get_by_isbn(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: OlModel::Edition(resp.model),
                })
            }
            OlRequest::GetWorks(req) => {
                let resp = self.get_works(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: OlModel::Works(resp.model),
                })
            }
        }
    }
}
