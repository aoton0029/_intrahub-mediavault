pub mod models;
pub mod requests;

use std::time::Duration;

use crate::error::ApiError;
use crate::response::{ApiResponse, RawData, RequestResult};
use crate::traits::ApiClient;

use self::models::{parse_ndl_xml, NdlItemModel, NdlModel};
use self::requests::{NdlRequest, NdlSearchRequest};

const NDL_BASE_URL: &str = "https://ndlsearch.ndl.go.jp/api";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(3);

/// NDL OpenSearch API クライアント。
///
/// 認証不要の公開 API。レート制限プリセットなし。
pub struct NdlClient {
    http: reqwest::Client,
    base_url: String,
}

impl NdlClient {
    /// 本番 URL を使ってクライアントを作成する。
    pub fn new() -> Result<Self, ApiError> {
        Self::new_with_base_url(NDL_BASE_URL)
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

    /// NDL OpenSearch で書誌を検索する（GET /opensearch）。
    pub async fn search(
        &self,
        req: NdlSearchRequest,
    ) -> Result<ApiResponse<Vec<NdlItemModel>>, ApiError> {
        let mut url = reqwest::Url::parse(&format!("{}/opensearch", self.base_url))
            .map_err(|e| ApiError::Network(e.to_string()))?;

        {
            let mut params = url.query_pairs_mut();
            if let Some(t) = &req.title {
                params.append_pair("title", t);
            }
            if let Some(i) = &req.isbn {
                params.append_pair("isbn", i);
            }
            if let Some(c) = &req.creator {
                params.append_pair("creator", c);
            }
            if let Some(p) = &req.publisher {
                params.append_pair("publisher", p);
            }
            if let Some(a) = &req.any {
                params.append_pair("any", a);
            }
            if let Some(cnt) = req.cnt {
                params.append_pair("cnt", &cnt.to_string());
            }
            if let Some(d) = &req.dpid {
                params.append_pair("dpid", d);
            }
        }

        let url_str = url.to_string();
        tracing::debug!(operation = "search", "NDL request start");
        tracing::trace!(url = %url_str, "NDL search request sending");
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
            tracing::warn!(status, "NDL HTTP error");
            return Err(ApiError::Http { status, body });
        }

        let xml_text = response
            .text()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        tracing::trace!(response_len = xml_text.len(), "NDL parsing XML");
        let model = parse_ndl_xml(&xml_text)?;

        tracing::debug!(status, latency_ms, count = model.len(), "NDL search OK");

        Ok(ApiResponse {
            request: RequestResult {
                status,
                url: url_str,
                latency_ms,
            },
            raw: RawData::Xml(xml_text),
            model,
        })
    }
}

// ── ApiClient trait impl ──────────────────────────────────────────────────

impl ApiClient for NdlClient {
    type Request = NdlRequest;
    type Model = NdlModel;

    async fn execute(&self, request: NdlRequest) -> Result<ApiResponse<NdlModel>, ApiError> {
        match request {
            NdlRequest::Search(req) => {
                let resp = self.search(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: NdlModel::Items(resp.model),
                })
            }
        }
    }
}
