//! MediaVault-api を呼び出す HTTP クライアント層
//!
//! TASK-0008: ApiClient層の実装

use std::time::Duration;

use reqwest::{Method, StatusCode, Url};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::api::envelope::{ApiEnvelope, ApiPagination};
use crate::api::error::ApiClientError;
use crate::config::SecretString;

/// `/api/v1/internal/` 配下のパスにのみ内部APIキーを付与する。
///
/// 🔵 Intent: NFR-103・`routes/internal.rs` のパス規約より。公開APIは無認証のため
///    キーを送る意味がなく、誤送信を防ぐためパスで厳密に判定する。
const INTERNAL_PATH_PREFIX: &str = "/api/v1/internal/";

/// 外部カタログ検索専用のタイムアウト。
///
/// 🟡 Intent: TASK-0015 タスクファイル「通常の15秒ではなく30秒」より。
///    `Config.external_timeout_secs` は起動時検証のみに使われ実際のHTTP呼び出しへは
///    まだ配線されていないため、ここでは固定値を用いる（既定値と一致）。
const EXTERNAL_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// `get` の戻り値。ページネーション情報を上位（TASK-0013）へ伝搬する。
///
/// 🔵 Intent: `items.md` の pagination 仕様に基づく。
#[derive(Debug, Clone)]
pub struct ApiResponse<T> {
    pub data: T,
    pub pagination: Option<ApiPagination>,
}

/// MediaVault-api 呼び出し用の HTTP クライアント。
///
/// 🔵 Intent: `reqwest::Client` はコネクションプールを内部に保持するため、
///    `Arc<ApiClient>` として単一インスタンスを共有する想定（呼び出し側の責務）。
pub struct ApiClient {
    http: reqwest::Client,
    base_url: Url,
    internal_api_key: SecretString,
}

impl ApiClient {
    /// 🔵 Intent: 接続タイムアウトと全体タイムアウトを分けて設定する（NFR-006）。
    ///    値は `Config` から渡す想定。
    pub fn new(
        base_url: Url,
        internal_api_key: SecretString,
        connect_timeout: Duration,
        request_timeout: Duration,
    ) -> Result<Self, ApiClientError> {
        let http = reqwest::Client::builder()
            .connect_timeout(connect_timeout)
            .timeout(request_timeout)
            .build()
            .map_err(|err| ApiClientError::Connection(err.to_string()))?;

        Ok(Self {
            http,
            base_url,
            internal_api_key,
        })
    }

    /// 🔵 Intent: REQ-141 を層レベルで担保する設計。`get`/`post`/`post_empty`/`patch` のみを
    ///    公開し、`delete` は実装しない（誤って削除系ツールが作られることを型レベルで防ぐ）。
    pub async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<ApiResponse<T>, ApiClientError> {
        let url = self.build_url(path, query)?;

        // 🟡 Intent: 冪等な GET のみ、接続エラー時に最大1回リトライする
        //    （architecture.md 可用性方針より）。
        match self
            .execute::<T>(Method::GET, url.clone(), path, None::<&()>, None)
            .await
        {
            Ok(response) => Ok(response),
            Err(ApiClientError::Connection(_)) => {
                self.execute::<T>(Method::GET, url, path, None::<&()>, None)
                    .await
            }
            Err(err) => Err(err),
        }
    }

    /// 外部プロバイダを経由するエンドポイント専用。通常より長い30秒タイムアウトを使う。
    ///
    /// 🔵 Intent: タスクファイル「外部カタログ検索のみ30秒」より。`get` と異なり
    ///    接続エラー時の自動リトライは行わない（30秒×2の待ちを避けるため）。
    pub async fn get_external<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<ApiResponse<T>, ApiClientError> {
        let url = self.build_url(path, query)?;
        self.execute::<T>(
            Method::GET,
            url,
            path,
            None::<&()>,
            Some(EXTERNAL_REQUEST_TIMEOUT),
        )
        .await
    }

    pub async fn post<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiClientError> {
        let url = self.build_url(path, &[])?;
        self.execute::<T>(Method::POST, url, path, Some(body), None)
            .await
            .map(|response| response.data)
    }

    pub async fn post_empty<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiClientError> {
        let url = self.build_url(path, &[])?;
        self.execute::<T>(Method::POST, url, path, None::<&()>, None)
            .await
            .map(|response| response.data)
    }

    pub async fn patch<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiClientError> {
        let url = self.build_url(path, &[])?;
        self.execute::<T>(Method::PATCH, url, path, Some(body), None)
            .await
            .map(|response| response.data)
    }

    /// ボディを持つが、成功時のレスポンスボディが空(`201`のみ)なエンドポイント用。
    ///
    /// 🟡 Intent: `POST /mylists/{id}/items` 等、成功時に `ApiEnvelope` を返さない
    ///    （`StatusCode::CREATED` のみ）エンドポイントがあるため（TASK-0018 で判明）、
    ///    `execute` とは別経路で成功/失敗を判定する。失敗時のボディは通常どおり
    ///    `ApiEnvelope::Err` として解釈する。
    pub async fn post_no_content<B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<(), ApiClientError> {
        let url = self.build_url(path, &[])?;
        self.execute_no_content(url, path, Some(body)).await
    }

    /// ボディを持たず、成功時のレスポンスボディも空なエンドポイント用。
    ///
    /// 🟡 Intent: `POST /items/{id}/tags/{tag_id}`（付与のみ、リクエストボディなし）用。
    pub async fn post_empty_no_content(&self, path: &str) -> Result<(), ApiClientError> {
        let url = self.build_url(path, &[])?;
        self.execute_no_content::<()>(url, path, None).await
    }

    async fn execute_no_content<B: Serialize>(
        &self,
        url: Url,
        path: &str,
        body: Option<&B>,
    ) -> Result<(), ApiClientError> {
        let mut request = self.http.request(Method::POST, url);

        if path.starts_with(INTERNAL_PATH_PREFIX) {
            request = request.bearer_auth(self.internal_api_key.expose());
        }
        if let Some(body) = body {
            request = request.json(body);
        }

        let response = request.send().await.map_err(classify_transport_error)?;
        let status = response.status();

        if status == StatusCode::UNAUTHORIZED && path.starts_with(INTERNAL_PATH_PREFIX) {
            return Err(ApiClientError::Auth);
        }

        if status.is_success() {
            return Ok(());
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|err| ApiClientError::Decode(err.to_string()))?;
        let envelope: ApiEnvelope<serde_json::Value> = serde_json::from_slice(&bytes)
            .map_err(|err| ApiClientError::Decode(err.to_string()))?;

        match envelope {
            ApiEnvelope::Err { error } => Err(to_api_error(error, status)),
            ApiEnvelope::Ok { .. } => Err(ApiClientError::Decode(
                "空ボディを期待したが success envelope が返った".to_string(),
            )),
        }
    }

    fn build_url(&self, path: &str, query: &[(&str, String)]) -> Result<Url, ApiClientError> {
        let mut url = self
            .base_url
            .join(path)
            .map_err(|err| ApiClientError::Connection(err.to_string()))?;
        if !query.is_empty() {
            url.query_pairs_mut().extend_pairs(query);
        }
        Ok(url)
    }

    /// 🔵 Intent: レスポンスを一箇所で処理し、401/デシリアライズ失敗/業務エラーの
    ///    分類ロジックを `get`/`post`/`post_empty`/`patch` で重複させない。
    async fn execute<T: DeserializeOwned>(
        &self,
        method: Method,
        url: Url,
        path: &str,
        body: Option<&impl Serialize>,
        timeout_override: Option<Duration>,
    ) -> Result<ApiResponse<T>, ApiClientError> {
        let mut request = self.http.request(method, url);

        if path.starts_with(INTERNAL_PATH_PREFIX) {
            request = request.bearer_auth(self.internal_api_key.expose());
        }

        if let Some(body) = body {
            request = request.json(body);
        }

        if let Some(timeout) = timeout_override {
            request = request.timeout(timeout);
        }

        let response = request.send().await.map_err(classify_transport_error)?;
        let status = response.status();

        if status == StatusCode::UNAUTHORIZED && path.starts_with(INTERNAL_PATH_PREFIX) {
            return Err(ApiClientError::Auth);
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|err| ApiClientError::Decode(err.to_string()))?;

        let envelope: ApiEnvelope<T> = serde_json::from_slice(&bytes)
            .map_err(|err| ApiClientError::Decode(err.to_string()))?;

        match envelope {
            ApiEnvelope::Ok { data, pagination } => Ok(ApiResponse { data, pagination }),
            ApiEnvelope::Err { error } => Err(to_api_error(error, status)),
        }
    }
}

fn to_api_error(error: crate::api::envelope::ApiErrorBody, status: StatusCode) -> ApiClientError {
    if error.code == "AMBIGUOUS_FILE" {
        return ApiClientError::AmbiguousFile {
            message: error.message,
            candidates: error
                .candidates
                .unwrap_or_else(|| serde_json::Value::Array(Vec::new())),
        };
    }
    ApiClientError::Api {
        code: error.code,
        message: error.message,
        status: status.as_u16(),
    }
}

/// 🔵 Intent: reqwest レベルの接続失敗・タイムアウト・DNS失敗を `Connection` に集約する。
fn classify_transport_error(err: reqwest::Error) -> ApiClientError {
    ApiClientError::Connection(err.to_string())
}
