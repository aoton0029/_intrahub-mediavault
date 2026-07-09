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

use self::models::{BookModel, BooksSearchResponse, RakutenModel};
use self::requests::{RakutenRequest, SearchBooksRequest};

const RAKUTEN_BASE_URL: &str = "https://openapi.rakuten.co.jp/services/api";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(3);

/// 楽天ブックス書籍検索API クライアント。
///
/// `applicationId` + `accessKey`（[`AuthStrategy::RakutenAppAuth`]）の2値で認証する。
/// Rakuten Developers側で「Allowed IP Addresses」を設定している場合、
/// 許可されていないIPからは `403 CLIENT_IP_NOT_ALLOWED` が返る。
/// レート制限: 保守的に 1 req / 1 秒として扱う（公式のレート上限値は非公開）。
pub struct RakutenClient {
    http: reqwest::Client,
    auth: AuthStrategy,
    rate_limit: Arc<Mutex<RateLimitState>>,
    base_url: String,
}

impl RakutenClient {
    /// 本番 URL を使ってクライアントを作成する。
    pub fn new(auth: AuthStrategy) -> Result<Self, ApiError> {
        Self::new_with_base_url(auth, RAKUTEN_BASE_URL)
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
            rate_limit: Arc::new(Mutex::new(RateLimitState::new(1, 1))),
            base_url: base_url.into(),
        })
    }

    // ── 内部ヘルパー ────────────────────────────────────────────────────

    fn apply_auth(&self, mut url: reqwest::Url) -> reqwest::Url {
        if let AuthStrategy::RakutenAppAuth {
            application_id,
            access_key,
        } = &self.auth
        {
            let mut p = url.query_pairs_mut();
            p.append_pair("applicationId", application_id);
            p.append_pair("accessKey", access_key);
        }
        url
    }

    async fn get_json(&self, url: reqwest::Url) -> Result<(u16, String, u64), ApiError> {
        {
            let mut rl = self.rate_limit.lock().await;
            rl.check_and_increment()?;
        }

        let url_with_auth = self.apply_auth(url);
        let url_str = url_with_auth.to_string();
        tracing::debug!(url = %url_str, "Rakuten request sending");

        let builder = self.http.get(&url_str).timeout(REQUEST_TIMEOUT);

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
            tracing::warn!(status, body = %body, "Rakuten HTTP error");
            return Err(ApiError::Http { status, body });
        }

        let text = response
            .text()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        tracing::debug!(status, latency_ms, body = %text, "Rakuten response received");
        Ok((status, text, latency_ms))
    }

    fn build_url(&self, path: &str) -> Result<reqwest::Url, ApiError> {
        reqwest::Url::parse(&format!("{}{}", self.base_url, path))
            .map_err(|e| ApiError::Network(e.to_string()))
    }

    // ── 公開メソッド ─────────────────────────────────────────────────────

    /// 書籍を検索する（GET /BooksBook/Search/20170404）。
    pub async fn search_books(
        &self,
        req: SearchBooksRequest,
    ) -> Result<ApiResponse<Vec<BookModel>>, ApiError> {
        let mut url = self.build_url("/BooksBook/Search/20170404")?;
        {
            let mut p = url.query_pairs_mut();
            p.append_pair("format", "json");
            if let Some(v) = &req.title {
                p.append_pair("title", v);
            }
            if let Some(v) = &req.author {
                p.append_pair("author", v);
            }
            if let Some(v) = &req.publisher_name {
                p.append_pair("publisherName", v);
            }
            if let Some(v) = &req.isbn {
                p.append_pair("isbn", v);
            }
            if let Some(v) = &req.books_genre_id {
                p.append_pair("booksGenreId", v);
            }
            if let Some(v) = &req.size {
                p.append_pair("size", v);
            }
            if let Some(v) = &req.sort {
                p.append_pair("sort", v);
            }
            if let Some(v) = req.page {
                p.append_pair("page", &v.to_string());
            }
            if let Some(v) = req.hits {
                p.append_pair("hits", &v.to_string());
            }
        }

        let url_str = url.to_string();
        tracing::debug!(operation = "search_books", "Rakuten request start");
        let (status, text, latency_ms) = self.get_json(url).await?;

        let parsed: BooksSearchResponse =
            serde_json::from_str(&text).map_err(|e| ApiError::Parse(e.to_string()))?;
        let books = parsed.items.into_iter().map(|w| w.item).collect::<Vec<_>>();

        tracing::debug!(
            status,
            latency_ms,
            result_count = books.len(),
            "Rakuten search_books OK"
        );

        Ok(ApiResponse {
            request: RequestResult {
                status,
                url: url_str,
                latency_ms,
            },
            raw: RawData::Json(text),
            model: books,
        })
    }
}

// ── ApiClient trait impl ──────────────────────────────────────────────────

impl ApiClient for RakutenClient {
    type Request = RakutenRequest;
    type Model = RakutenModel;

    async fn execute(
        &self,
        request: RakutenRequest,
    ) -> Result<ApiResponse<RakutenModel>, ApiError> {
        match request {
            RakutenRequest::SearchBooks(req) => {
                let resp = self.search_books(req).await?;
                Ok(ApiResponse {
                    request: resp.request,
                    raw: resp.raw,
                    model: RakutenModel::Books(resp.model),
                })
            }
        }
    }
}
