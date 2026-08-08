//! `health` ツールの統合テスト（wiremock）
//!
//! TASK-0010: health ツールの実装

use std::sync::Arc;

use axum::body::Body;
use axum::http::Request as HttpRequest;
use axum::{Router, middleware};
use mediavault_mcp::auth;
use mediavault_mcp::config::Config;
use mediavault_mcp::server::MediaVaultServer;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use serde_json::{Value, json};
use tokio_util::sync::CancellationToken;
use tower::ServiceExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

mod common;

const TOKEN: &str = "test-auth-token-with-sufficient-length-000000";

fn test_config() -> Arc<Config> {
    let mut env = std::collections::HashMap::new();
    env.insert("MCP_AUTH_TOKEN".to_string(), TOKEN.to_string());
    env.insert(
        "MEDIAVAULT_API_BASE_URL".to_string(),
        "http://mediavault-api:8080".to_string(),
    );
    Arc::new(Config::from_map_for_test(&env))
}

fn extract_json_rpc_result(body: &str) -> Option<Value> {
    if let Ok(value) = serde_json::from_str::<Value>(body) {
        return Some(value);
    }
    body.lines()
        .filter_map(|line| line.strip_prefix("data: "))
        .filter_map(|data| serde_json::from_str::<Value>(data).ok())
        .find(|value| value.get("result").is_some())
}

fn init_body() -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {"name": "test-client", "version": "1.0.0"}
        }
    })
}

async fn spawn_mediavault_service(
    api: Arc<mediavault_mcp::api::client::ApiClient>,
) -> (reqwest::Client, String, CancellationToken) {
    let ct = CancellationToken::new();
    let config = StreamableHttpServerConfig::default()
        .with_json_response(true)
        .with_sse_keep_alive(None)
        .with_cancellation_token(ct.child_token());

    let service = StreamableHttpService::new(
        move || Ok(MediaVaultServer::new(api.clone())),
        Arc::new(LocalSessionManager::default()),
        config,
    );
    let router = Router::new().nest_service("/mcp", service);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn({
        let ct = ct.clone();
        async move {
            let _ = axum::serve(listener, router)
                .with_graceful_shutdown(async move { ct.cancelled_owned().await })
                .await;
        }
    });

    (reqwest::Client::new(), format!("http://{addr}/mcp"), ct)
}

async fn call_health(client: &reqwest::Client, url: &str) -> Value {
    let init_response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&init_body())
        .send()
        .await
        .expect("initialize should succeed");
    let session_id = init_response
        .headers()
        .get("Mcp-Session-Id")
        .map(|v| v.to_str().unwrap().to_string());

    let mut call_request = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {"name": "health", "arguments": {}}
        }));
    if let Some(session_id) = &session_id {
        call_request = call_request.header("Mcp-Session-Id", session_id);
    }
    let response = call_request.send().await.unwrap();
    assert_eq!(response.status(), 200);
    let text = response.text().await.unwrap();
    extract_json_rpc_result(&text).expect("response should contain a JSON-RPC result")
}

/// 統合テスト1: api 到達時
#[tokio::test]
async fn health_reports_reachable_true_with_latency_when_api_is_up() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": {"status": "ok"}
        })))
        .mount(&mock_server)
        .await;
    let api = Arc::new(common::build_client(&mock_server, "internal-key"));
    let (client, url, ct) = spawn_mediavault_service(api).await;

    let body = call_health(&client, &url).await;
    let structured = &body["result"]["structuredContent"];

    assert_eq!(structured["outcome"], "success");
    assert_eq!(structured["api"]["reachable"], true);
    assert!(structured["api"]["latency_ms"].is_number());
    assert!(structured["api"]["error"].is_null());

    ct.cancel();
}

/// 統合テスト2: api 停止時（到達不能アドレスを設定）
#[tokio::test]
async fn health_reports_reachable_false_with_mcp_api_unreachable_when_api_is_down() {
    // 到達不能アドレス: 一度も listen していないポートへ接続させ、接続拒否を発生させる
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let down_addr = listener.local_addr().unwrap();
    drop(listener);

    let base_url = url::Url::parse(&format!("http://{down_addr}")).unwrap();
    let api = Arc::new(
        mediavault_mcp::api::client::ApiClient::new(
            base_url,
            mediavault_mcp::config::SecretString::from("internal-key".to_string()),
            std::time::Duration::from_secs(3),
            std::time::Duration::from_secs(15),
        )
        .unwrap(),
    );
    let (client, url, ct) = spawn_mediavault_service(api).await;

    let body = call_health(&client, &url).await;
    let structured = &body["result"]["structuredContent"];

    assert_eq!(structured["outcome"], "success");
    assert_eq!(structured["api"]["reachable"], false);
    assert!(structured["api"]["latency_ms"].is_null());
    assert_eq!(structured["api"]["error"]["code"], "MCP_API_UNREACHABLE");
    assert_eq!(structured["api"]["error"]["retriable"], true);

    ct.cancel();
}

/// 統合テスト3: api が 500 を返す
#[tokio::test]
async fn health_distinguishes_internal_error_from_unreachable() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/health"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "success": false,
            "error": {"code": "INTERNAL_ERROR", "message": "サーバーエラー"}
        })))
        .mount(&mock_server)
        .await;
    let api = Arc::new(common::build_client(&mock_server, "internal-key"));
    let (client, url, ct) = spawn_mediavault_service(api).await;

    let body = call_health(&client, &url).await;
    let structured = &body["result"]["structuredContent"];

    assert_eq!(structured["api"]["reachable"], false);
    assert_eq!(structured["api"]["error"]["code"], "INTERNAL_ERROR");
    assert_ne!(structured["api"]["error"]["code"], "MCP_API_UNREACHABLE");

    ct.cancel();
}

/// 統合テスト4: tools/list への登場と annotation
#[tokio::test]
async fn health_appears_in_tools_list_as_read_only() {
    let mock_server = common::start_mock_server().await;
    let api = Arc::new(common::build_client(&mock_server, "internal-key"));
    let (client, url, ct) = spawn_mediavault_service(api).await;

    let init_response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&init_body())
        .send()
        .await
        .expect("initialize should succeed");
    let session_id = init_response
        .headers()
        .get("Mcp-Session-Id")
        .map(|v| v.to_str().unwrap().to_string());

    let mut list_request = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}));
    if let Some(session_id) = &session_id {
        list_request = list_request.header("Mcp-Session-Id", session_id);
    }
    let response = list_request.send().await.unwrap();
    let text = response.text().await.unwrap();
    let body = extract_json_rpc_result(&text).expect("response should contain a JSON-RPC result");

    let tools = body["result"]["tools"].as_array().unwrap();
    let health_tool = tools
        .iter()
        .find(|t| t["name"] == "health")
        .expect("health tool should be listed");
    assert_eq!(health_tool["annotations"]["readOnlyHint"], true);

    ct.cancel();
}

/// 統合テスト5: 秘密情報の非露出
#[tokio::test]
async fn health_result_does_not_expose_secrets() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": {"status": "ok"}
        })))
        .mount(&mock_server)
        .await;
    let api = Arc::new(common::build_client(
        &mock_server,
        "super-secret-internal-key",
    ));
    let (client, url, ct) = spawn_mediavault_service(api).await;

    let body = call_health(&client, &url).await;
    let text = serde_json::to_string(&body).unwrap();

    assert!(!text.contains("super-secret-internal-key"));
    assert!(!text.contains(TOKEN));

    ct.cancel();
}

/// 統合テスト6: 認証必須
#[tokio::test]
async fn health_requires_authentication() {
    let mock_server = common::start_mock_server().await;
    let config = test_config();
    let api = Arc::new(common::build_client(&mock_server, "internal-key"));

    let mcp_service = StreamableHttpService::new(
        move || Ok(MediaVaultServer::new(api.clone())),
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default().with_json_response(true),
    );
    let app =
        Router::new()
            .nest_service("/mcp", mcp_service)
            .layer(middleware::from_fn_with_state(
                config.clone(),
                auth::bearer_auth,
            ));

    let response = app
        .oneshot(
            HttpRequest::builder()
                .method("POST")
                .uri("/mcp")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    assert!(mock_server.received_requests().await.unwrap().is_empty());
}
