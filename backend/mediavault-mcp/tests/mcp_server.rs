//! `/mcp` の rmcp サーバー統合テスト
//!
//! TASK-0009: 共通結果型と rmcp サーバー骨格

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

/// レスポンスが `application/json` でも `text/event-stream`（`data: {...}` 行）でも、
/// `"result"` を含む最初のJSON-RPCメッセージを取り出す。
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

/// 統合テスト1・2: rmcp の axum サービスを直接起動し、initialize → tools/list を確認する。
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

/// 統合テスト1: tools/list が応答する
#[tokio::test]
async fn tools_list_responds() {
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
    assert_eq!(init_response.status(), 200);
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

    let response = list_request
        .send()
        .await
        .expect("tools/list should succeed");
    assert_eq!(response.status(), 200);
    let text = response.text().await.unwrap();
    let body = extract_json_rpc_result(&text).expect("response should contain a JSON-RPC result");
    assert!(body["result"]["tools"].is_array());

    ct.cancel();
}

/// ダミーツール1つだけを持つ最小サーバー。設計決定 D-01（構造化結果への統一）が
/// `rmcp` で実現できることを、`Outcome` を含む構造体を実際に返して確認する。
#[derive(Debug, Clone, Default)]
#[allow(
    dead_code,
    reason = "tool_router は #[tool_handler] マクロが内部で使用する"
)]
struct DummyServer {
    tool_router: rmcp::handler::server::router::tool::ToolRouter<Self>,
}

#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
struct DummyResult {
    outcome: mediavault_mcp::result::outcome::Outcome,
    message: String,
}

#[rmcp::tool_router]
impl DummyServer {
    fn build() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[rmcp::tool(description = "テスト用のダミーツール")]
    async fn dummy(&self) -> rmcp::Json<DummyResult> {
        rmcp::Json(DummyResult {
            outcome: mediavault_mcp::result::outcome::Outcome::Success,
            message: "ok".to_string(),
        })
    }
}

#[rmcp::tool_handler]
impl rmcp::ServerHandler for DummyServer {}

/// 統合テスト2: 構造化結果をツール結果として返せる（設計決定 D-01 が成立する）
#[tokio::test]
async fn structured_result_round_trips_as_json() {
    let ct = CancellationToken::new();
    let config = StreamableHttpServerConfig::default()
        .with_json_response(true)
        .with_sse_keep_alive(None)
        .with_cancellation_token(ct.child_token());
    let service = StreamableHttpService::new(
        || Ok(DummyServer::build()),
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
    let client = reqwest::Client::new();
    let url = format!("http://{addr}/mcp");

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

    let mut call_request = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {"name": "dummy", "arguments": {}}
        }));
    if let Some(session_id) = &session_id {
        call_request = call_request.header("Mcp-Session-Id", session_id);
    }
    let response = call_request.send().await.unwrap();
    assert_eq!(response.status(), 200);
    let text = response.text().await.unwrap();
    let body = extract_json_rpc_result(&text).expect("response should contain a JSON-RPC result");

    let structured = &body["result"]["structuredContent"];
    assert_eq!(structured["outcome"], "success");
    assert_eq!(structured["message"], "ok");

    ct.cancel();
}

/// 統合テスト3: 認証との統合（無認証で401、api には到達しない）
#[tokio::test]
async fn unauthenticated_request_is_rejected_before_reaching_api() {
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
