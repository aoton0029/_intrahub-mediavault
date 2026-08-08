//! wiremock を使う統合テストの共通フィクスチャ
//!
//! TASK-0008: ApiClient層の実装

use std::time::Duration;

use mediavault_mcp::api::client::ApiClient;
use mediavault_mcp::config::SecretString;
use url::Url;
use wiremock::MockServer;

/// 🟡 Intent: Phase 2・3 の全ツールテストの基盤として共通化する
///    （タスクファイル「注意事項」より）。接続3秒/全体15秒は本番既定値と揃える。
pub async fn start_mock_server() -> MockServer {
    MockServer::start().await
}

pub fn build_client(mock_server: &MockServer, internal_api_key: &str) -> ApiClient {
    let base_url = Url::parse(&mock_server.uri()).expect("mock server uri should be a valid url");
    ApiClient::new(
        base_url,
        SecretString::from(internal_api_key.to_string()),
        Duration::from_secs(3),
        Duration::from_secs(15),
    )
    .expect("client should build")
}

#[allow(dead_code, reason = "一部の統合テストバイナリでのみ使用する")]
pub fn build_client_with_timeout(
    mock_server: &MockServer,
    internal_api_key: &str,
    request_timeout: Duration,
) -> ApiClient {
    let base_url = Url::parse(&mock_server.uri()).expect("mock server uri should be a valid url");
    ApiClient::new(
        base_url,
        SecretString::from(internal_api_key.to_string()),
        Duration::from_secs(3),
        request_timeout,
    )
    .expect("client should build")
}
