//! read-only トークンによるツールスコープ分離の統合テスト
//!
//! 設計決定 D-10（api-tool-mapping.md §4）より。
//!
//! 利用側（intrahub-mastra）の防御は「Agent へ渡すツール集合を絞る」というクライアント側の
//! 対策のみに依存しており、MCP 側の設定ミスや別クライアントの接続では防げない。
//! read-only トークンはその二重の防御線であり、本テストは**サーバ側で実際に止まること**を確認する。

use std::collections::HashMap;
use std::sync::Arc;

use axum::{Router, middleware};
use mediavault_mcp::auth;
use mediavault_mcp::config::Config;
use mediavault_mcp::server::MediaVaultServer;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use serde_json::{Value, json};
use tokio_util::sync::CancellationToken;
use wiremock::MockServer;

mod common;

const FULL_TOKEN: &str = "full-access-token-with-sufficient-length-0000";
const READONLY_TOKEN: &str = "read-only-token-with-sufficient-length-00000";

/// `readOnlyHint: true` のツール。read-only セッションでも見えて呼べる。
const READ_ONLY_TOOLS: [&str; 8] = [
    "health",
    "search_library",
    "search_external_catalog",
    "get_item_context",
    "get_item_text",
    "get_extraction_status",
    "collection_overview",
    "list_citations",
];

/// 書き込み系ツール。read-only セッションからは見えず、呼べない。
const WRITE_TOOLS: [&str; 9] = [
    "import_external_item",
    "create_item",
    "update_consumption",
    "organize_item",
    "relate_items",
    "add_access_link",
    "add_citation",
    "request_extraction",
    "cancel_extraction",
];

fn config_with(readonly: Option<&str>) -> Arc<Config> {
    let mut env = HashMap::new();
    env.insert("MCP_AUTH_TOKEN".to_string(), FULL_TOKEN.to_string());
    env.insert(
        "MEDIAVAULT_API_BASE_URL".to_string(),
        "http://mediavault-api:8080".to_string(),
    );
    if let Some(token) = readonly {
        env.insert("MCP_READONLY_TOKEN".to_string(), token.to_string());
    }
    Arc::new(Config::from_map_for_test(&env))
}

fn extract_json_rpc(body: &str) -> Option<Value> {
    if let Ok(value) = serde_json::from_str::<Value>(body) {
        return Some(value);
    }
    body.lines()
        .filter_map(|line| line.strip_prefix("data: "))
        .filter_map(|data| serde_json::from_str::<Value>(data).ok())
        .find(|value| value.get("result").is_some() || value.get("error").is_some())
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

/// 認証ミドルウェアを含む本番同等の構成で起動する。
/// `TokenScope` はミドルウェアが挿入するため、これを省くとテストの意味がなくなる。
async fn spawn(
    mock_server: &MockServer,
    config: Arc<Config>,
) -> (reqwest::Client, String, CancellationToken) {
    let api = Arc::new(common::build_client(mock_server, "internal-key"));
    let ct = CancellationToken::new();
    let http_config = StreamableHttpServerConfig::default()
        .with_json_response(true)
        .with_sse_keep_alive(None)
        .with_cancellation_token(ct.child_token());

    let service = StreamableHttpService::new(
        move || Ok(MediaVaultServer::new(api.clone())),
        Arc::new(LocalSessionManager::default()),
        http_config,
    );
    let router = Router::new()
        .nest_service("/mcp", service)
        .layer(middleware::from_fn_with_state(config, auth::bearer_auth));

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

/// initialize → 任意のリクエストを1本送り、JSON-RPC のレスポンスを返す。
async fn call_with_token(
    client: &reqwest::Client,
    url: &str,
    token: &str,
    request: Value,
) -> Value {
    let init = client
        .post(url)
        .bearer_auth(token)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&init_body())
        .send()
        .await
        .expect("initialize should reach the server");
    assert_eq!(init.status(), 200, "initialize が失敗した");
    let session_id = init
        .headers()
        .get("Mcp-Session-Id")
        .map(|v| v.to_str().unwrap().to_string());

    let mut req = client
        .post(url)
        .bearer_auth(token)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&request);
    if let Some(session_id) = &session_id {
        req = req.header("Mcp-Session-Id", session_id);
    }

    let response = req.send().await.expect("request should reach the server");
    let text = response.text().await.unwrap();
    extract_json_rpc(&text).expect("JSON-RPC レスポンスが取れる")
}

fn tools_list() -> Value {
    json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}})
}

fn tool_names(body: &Value) -> Vec<String> {
    body["result"]["tools"]
        .as_array()
        .expect("tools は配列")
        .iter()
        .map(|tool| tool["name"].as_str().unwrap().to_string())
        .collect()
}

// ============================================================
// tools/list のスコープ絞り込み
// ============================================================

/// read-only トークンでは書き込みツールが一覧に現れない。
#[tokio::test]
async fn readonly_token_lists_only_read_only_tools() {
    let mock_server = common::start_mock_server().await;
    let (client, url, ct) = spawn(&mock_server, config_with(Some(READONLY_TOKEN))).await;

    let body = call_with_token(&client, &url, READONLY_TOKEN, tools_list()).await;
    let names = tool_names(&body);

    for tool in READ_ONLY_TOOLS {
        assert!(names.contains(&tool.to_string()), "{tool} が見えるべき");
    }
    for tool in WRITE_TOOLS {
        assert!(
            !names.contains(&tool.to_string()),
            "{tool} は read-only セッションへ見せてはならない"
        );
    }

    ct.cancel();
}

/// 通常トークンでは全ツールが見える。
#[tokio::test]
async fn full_token_lists_all_tools() {
    let mock_server = common::start_mock_server().await;
    let (client, url, ct) = spawn(&mock_server, config_with(Some(READONLY_TOKEN))).await;

    let body = call_with_token(&client, &url, FULL_TOKEN, tools_list()).await;
    let names = tool_names(&body);

    for tool in READ_ONLY_TOOLS.iter().chain(WRITE_TOOLS.iter()) {
        assert!(names.contains(&tool.to_string()), "{tool} が見えるべき");
    }

    ct.cancel();
}

/// read-only トークンが未設定なら従来どおり（全ツールが見える）。
#[tokio::test]
async fn without_readonly_token_configured_all_tools_are_listed() {
    let mock_server = common::start_mock_server().await;
    let (client, url, ct) = spawn(&mock_server, config_with(None)).await;

    let body = call_with_token(&client, &url, FULL_TOKEN, tools_list()).await;
    let names = tool_names(&body);

    assert_eq!(
        names.len(),
        READ_ONLY_TOOLS.len() + WRITE_TOOLS.len(),
        "既定は単一トークン運用で全ツールを公開する"
    );

    ct.cancel();
}

/// `tools/list` は共有キャッシュ可能として返してはならない。
/// スコープごとに内容が変わるため、全権セッションの一覧が read-only 側へ配られると防御が崩れる。
#[tokio::test]
async fn tools_list_is_not_publicly_cacheable() {
    let mock_server = common::start_mock_server().await;
    let (client, url, ct) = spawn(&mock_server, config_with(Some(READONLY_TOKEN))).await;

    let body = call_with_token(&client, &url, READONLY_TOKEN, tools_list()).await;

    if let Some(scope) = body["result"].get("cacheScope") {
        assert_ne!(
            scope,
            &json!("public"),
            "スコープ依存の一覧を共有キャッシュさせてはならない"
        );
    }

    ct.cancel();
}

// ============================================================
// tools/call の拒否
// ============================================================

fn call_tool(name: &str, arguments: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {"name": name, "arguments": arguments}
    })
}

/// **一覧から隠すだけでは防御にならない。** 直接呼ばれても実行しない。
#[tokio::test]
async fn readonly_token_cannot_call_a_write_tool_directly() {
    let mock_server = common::start_mock_server().await;
    let (client, url, ct) = spawn(&mock_server, config_with(Some(READONLY_TOKEN))).await;

    let body = call_with_token(
        &client,
        &url,
        READONLY_TOKEN,
        call_tool(
            "create_item",
            json!({"title": "作品X", "media_type": "anime"}),
        ),
    )
    .await;

    assert!(
        body.get("error").is_some(),
        "書き込みツールの呼び出しは拒否されるべき: {body}"
    );
    assert!(
        mock_server.received_requests().await.unwrap().is_empty(),
        "拒否された呼び出しから MediaVault-api へ到達してはならない"
    );

    ct.cancel();
}

/// 拒否メッセージは「存在しない」と同じ形にする。
/// 「存在するが権限が無い」と返すと、書き込みツールの存在が read-only 側へ漏れる。
#[tokio::test]
async fn rejection_does_not_reveal_that_the_tool_exists() {
    let mock_server = common::start_mock_server().await;
    let (client, url, ct) = spawn(&mock_server, config_with(Some(READONLY_TOKEN))).await;

    let existing = call_with_token(
        &client,
        &url,
        READONLY_TOKEN,
        call_tool(
            "create_item",
            json!({"title": "作品X", "media_type": "anime"}),
        ),
    )
    .await;
    let nonexistent = call_with_token(
        &client,
        &url,
        READONLY_TOKEN,
        call_tool("no_such_tool_at_all", json!({})),
    )
    .await;

    let existing_msg = existing["error"]["message"].as_str().unwrap_or_default();
    let nonexistent_msg = nonexistent["error"]["message"].as_str().unwrap_or_default();

    assert!(
        existing_msg.contains("not found"),
        "存在しないツールと同じ語彙で返す: {existing_msg}"
    );
    assert!(
        !existing_msg.contains("permission")
            && !existing_msg.contains("read-only")
            && !existing_msg.contains("readonly"),
        "権限の存在を示唆してはならない: {existing_msg}"
    );
    assert!(
        nonexistent_msg.contains("not found"),
        "比較対象も not found であること: {nonexistent_msg}"
    );

    ct.cancel();
}

/// read-only トークンでも読み取りツールは通る（過剰な遮断をしていない）。
#[tokio::test]
async fn readonly_token_can_call_a_read_only_tool() {
    let mock_server = common::start_mock_server().await;
    let (client, url, ct) = spawn(&mock_server, config_with(Some(READONLY_TOKEN))).await;

    let body = call_with_token(
        &client,
        &url,
        READONLY_TOKEN,
        call_tool("health", json!({})),
    )
    .await;

    assert!(
        body.get("error").is_none(),
        "読み取りツールは通るべき: {body}"
    );

    ct.cancel();
}

/// 通常トークンなら書き込みツールを呼べる（スコープ分離が全体を壊していない）。
#[tokio::test]
async fn full_token_can_call_a_write_tool() {
    let mock_server = common::start_mock_server().await;
    let (client, url, ct) = spawn(&mock_server, config_with(Some(READONLY_TOKEN))).await;

    let body = call_with_token(
        &client,
        &url,
        FULL_TOKEN,
        call_tool(
            "create_item",
            json!({"title": "作品X", "media_type": "anime"}),
        ),
    )
    .await;

    // api 未モックのため MCP 側は接続エラーを結果本体で返すが、
    // **ツール自体は実行される**（プロトコル層の error にならない）
    assert!(
        body.get("error").is_none(),
        "通常トークンでは書き込みツールが実行される: {body}"
    );

    ct.cancel();
}

/// 誤ったトークンは従来どおり 401。read-only 追加で認証が緩んでいないことを確認する。
#[tokio::test]
async fn unknown_token_is_still_rejected_with_401() {
    let mock_server = common::start_mock_server().await;
    let (client, url, ct) = spawn(&mock_server, config_with(Some(READONLY_TOKEN))).await;

    let response = client
        .post(&url)
        .bearer_auth("neither-of-the-two-tokens")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&init_body())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 401);

    ct.cancel();
}

// ============================================================
// 設定の検証
// ============================================================

/// read-only トークンが書き込みトークンと同値なら起動しない。
/// 一致を許すと「read-only のつもりが全権」という設定ミスに気づけない。
#[test]
fn identical_tokens_fail_to_start() {
    let mut env = HashMap::new();
    env.insert("MCP_AUTH_TOKEN".to_string(), FULL_TOKEN.to_string());
    env.insert("MCP_READONLY_TOKEN".to_string(), FULL_TOKEN.to_string());
    env.insert(
        "MEDIAVAULT_API_BASE_URL".to_string(),
        "http://mediavault-api:8080".to_string(),
    );

    let result = std::panic::catch_unwind(|| Config::from_map_for_test(&env));
    assert!(
        result.is_err(),
        "同一トークンの設定は起動失敗にしなければならない"
    );
}
