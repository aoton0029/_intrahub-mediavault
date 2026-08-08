//! TASK-0024: エラー透過とエッジケースの網羅テスト
//!
//! 11ツールが同じ状況で同じ形のエラーを返すことを保証する横断テスト。
//! `tools/list` から動的にツールを取得するパターンは `tests/safety.rs`(TASK-0023) を踏襲する。

use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use mediavault_mcp::api::client::ApiClient;
use mediavault_mcp::config::SecretString;
use mediavault_mcp::server::MediaVaultServer;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use serde_json::{Value, json};
use tokio_util::sync::CancellationToken;
use url::Url;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

mod common;

const ITEM_ID: &str = "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001";
const RELATED_ITEM_ID: &str = "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0002";

// ---------------------------------------------------------------------------
// JSON-RPC / rmcp サーバー起動の共通ヘルパ（tests/safety.rs のパターンを踏襲）
// ---------------------------------------------------------------------------

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
            "clientInfo": {"name": "error-consistency-test-client", "version": "1.0.0"}
        }
    })
}

async fn spawn_service(api: Arc<ApiClient>) -> (reqwest::Client, String, CancellationToken) {
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

async fn init_session(client: &reqwest::Client, url: &str) -> Option<String> {
    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&init_body())
        .send()
        .await
        .expect("initialize should succeed");
    response
        .headers()
        .get("Mcp-Session-Id")
        .map(|v| v.to_str().unwrap().to_string())
}

async fn call_tool(client: &reqwest::Client, url: &str, name: &str, arguments: Value) -> Value {
    let session_id = init_session(client, url).await;
    let mut request = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {"name": name, "arguments": arguments}
        }));
    if let Some(session_id) = &session_id {
        request = request.header("Mcp-Session-Id", session_id);
    }
    let response = request.send().await.expect("tools/call should succeed");
    let text = response.text().await.unwrap();
    extract_json_rpc_result(&text).expect("response should contain a JSON-RPC result")
}

/// 各ツールを成功パスで呼ぶための最小限の引数（tests/safety.rs と同じ規約）。
fn success_arguments_for(name: &str) -> Value {
    match name {
        "health" => json!({}),
        "search_library" => json!({}),
        "search_external_catalog" => json!({"media_type": "anime", "q": "作品A"}),
        "get_item_context" => json!({"item_id": ITEM_ID}),
        "collection_overview" => json!({}),
        "import_external_item" => {
            json!({"media_type": "anime", "external_id": "12345", "provider": "annict"})
        }
        "create_item" => json!({"media_type": "manga", "title": "エラー透過テスト作品"}),
        "update_consumption" => json!({"item_id": ITEM_ID, "status": "completed"}),
        "organize_item" => json!({"item_id": ITEM_ID}),
        "relate_items" => json!({
            "item_id": ITEM_ID,
            "related_item_id": RELATED_ITEM_ID,
            "relation_type": "adaptation"
        }),
        "add_access_link" => json!({
            "item_id": ITEM_ID,
            "url": "https://example.com",
            "kind": "link",
            "label": "公式サイト"
        }),
        other => panic!("unknown tool in success_arguments_for: {other}"),
    }
}

/// 各ツールが最初に呼ぶ api エンドポイント。エラー透過・分類テストの注入先。
fn primary_endpoint_for(name: &str) -> (&'static str, String) {
    match name {
        "health" => ("GET", "/api/v1/health".to_string()),
        "search_library" => ("GET", "/api/v1/items".to_string()),
        "search_external_catalog" => ("GET", "/api/v1/items/search".to_string()),
        "get_item_context" => ("GET", format!("/api/v1/items/{ITEM_ID}")),
        "collection_overview" => ("GET", "/api/v1/collection/overview".to_string()),
        "import_external_item" => ("POST", "/api/v1/items/import".to_string()),
        "create_item" => ("POST", "/api/v1/items".to_string()),
        "update_consumption" => ("GET", format!("/api/v1/items/{ITEM_ID}")),
        "organize_item" => ("GET", format!("/api/v1/items/{ITEM_ID}")),
        "relate_items" => ("GET", format!("/api/v1/items/{ITEM_ID}")),
        "add_access_link" => ("POST", format!("/api/v1/items/{ITEM_ID}/links")),
        other => panic!("unknown tool in primary_endpoint_for: {other}"),
    }
}

fn all_eleven_tool_names() -> Vec<&'static str> {
    vec![
        "health",
        "search_library",
        "search_external_catalog",
        "get_item_context",
        "collection_overview",
        "import_external_item",
        "create_item",
        "update_consumption",
        "organize_item",
        "relate_items",
        "add_access_link",
    ]
}

async fn mount_error(
    mock_server: &MockServer,
    method_str: &str,
    endpoint: &str,
    status: u16,
    code: &str,
    message: &str,
) {
    Mock::given(method(method_str))
        .and(path(endpoint.to_string()))
        .respond_with(ResponseTemplate::new(status).set_body_json(json!({
            "success": false,
            "error": {"code": code, "message": message}
        })))
        .mount(mock_server)
        .await;
}

/// `relate_items` は item / related_item の両方を並列取得するため、両方の GET に
/// 同じレスポンスを張る必要がある。
async fn mount_error_for_tool(
    mock_server: &MockServer,
    name: &str,
    status: u16,
    code: &str,
    message: &str,
) {
    let (method_str, endpoint) = primary_endpoint_for(name);
    mount_error(mock_server, method_str, &endpoint, status, code, message).await;
    if name == "relate_items" {
        mount_error(
            mock_server,
            "GET",
            &format!("/api/v1/items/{RELATED_ITEM_ID}"),
            status,
            code,
            message,
        )
        .await;
    }
}

/// 接続失敗（ECONNREFUSED）を確実に発生させるための URL。
///
/// 🟡 Intent: `MockServer` を起動して drop するだけでは OS がポートを即座に再利用可能な
///    CLOSED 状態にしない場合があり、応答が不安定になる（EOF 等）ため、誰も listen していない
///    ことが保証されるローカルポートへ直接向ける。
fn unreachable_base_url() -> Url {
    Url::parse("http://127.0.0.1:1/").unwrap()
}

fn top_level_error(response: &Value) -> &Value {
    &response["result"]["structuredContent"]["error"]
}

fn top_level_outcome(response: &Value) -> &Value {
    &response["result"]["structuredContent"]["outcome"]
}

// ---------------------------------------------------------------------------
// 実装項目1: エラー透過の全ツール検証（REQ-146 / 統合テスト1）
// ---------------------------------------------------------------------------

/// 統合テスト1: 全11ツールで、api の code/message が原文のまま透過される。
#[tokio::test]
async fn all_tools_pass_through_api_error_code_and_message_verbatim() {
    const DISTINCTIVE_CODE: &str = "TAG_NOT_FOUND";
    const DISTINCTIVE_MESSAGE: &str = "タグが見つかりません";

    for name in all_eleven_tool_names() {
        if name == "health" {
            // health は接続失敗以外でも常に success を返す例外（統合テスト3で別途検証）
            continue;
        }

        let mock_server = common::start_mock_server().await;
        mount_error_for_tool(
            &mock_server,
            name,
            404,
            DISTINCTIVE_CODE,
            DISTINCTIVE_MESSAGE,
        )
        .await;
        let api = Arc::new(common::build_client(&mock_server, "internal-key"));
        let (client, url, ct) = spawn_service(api).await;

        let response = call_tool(&client, &url, name, success_arguments_for(name)).await;
        let error = top_level_error(&response);

        assert_eq!(
            error["code"], DISTINCTIVE_CODE,
            "{name} の code が原文のまま透過されていない: {response:?}"
        );
        assert_eq!(
            error["message"], DISTINCTIVE_MESSAGE,
            "{name} の message が原文のまま透過されていない: {response:?}"
        );

        ct.cancel();
    }
}

// ---------------------------------------------------------------------------
// 実装項目2: エラー分類の一貫性（REQ-120・EDGE-001 / EDGE-002・統合テスト2）
// ---------------------------------------------------------------------------

/// EDGE-001: api 停止時、全ツール（health を除く）が `MCP_API_UNREACHABLE` / `retriable: true` を返す。
#[tokio::test]
async fn edge_001_api_unreachable_is_classified_consistently_across_all_tools() {
    for name in all_eleven_tool_names() {
        let base_url = unreachable_base_url();
        let api = Arc::new(
            ApiClient::new(
                base_url,
                SecretString::from("internal-key".to_string()),
                Duration::from_millis(500),
                Duration::from_millis(500),
            )
            .expect("client should build"),
        );
        let (client, url, ct) = spawn_service(api).await;

        let response = call_tool(&client, &url, name, success_arguments_for(name)).await;

        if name == "health" {
            assert_eq!(
                top_level_outcome(&response),
                &json!("success"),
                "health は接続失敗でも success のはず: {response:?}"
            );
        } else {
            let error = top_level_error(&response);
            assert_eq!(
                error["code"], "MCP_API_UNREACHABLE",
                "{name} は api 停止時に MCP_API_UNREACHABLE を返すはず: {response:?}"
            );
            assert_eq!(
                error["retriable"], true,
                "{name} の api 停止エラーは retriable のはず"
            );
        }

        ct.cancel();
    }
}

/// api が 500 を返したとき、全ツールが api の code をそのまま・retriable: true で返す。
#[tokio::test]
async fn all_tools_classify_5xx_as_retriable_with_original_code() {
    for name in all_eleven_tool_names() {
        if name == "health" {
            continue;
        }
        let mock_server = common::start_mock_server().await;
        mount_error_for_tool(&mock_server, name, 500, "INTERNAL_ERROR", "サーバーエラー").await;
        let api = Arc::new(common::build_client(&mock_server, "internal-key"));
        let (client, url, ct) = spawn_service(api).await;

        let response = call_tool(&client, &url, name, success_arguments_for(name)).await;
        let error = top_level_error(&response);

        assert_eq!(error["code"], "INTERNAL_ERROR", "{name}: {response:?}");
        assert_eq!(error["retriable"], true, "{name}: {response:?}");

        ct.cancel();
    }
}

/// api が 400 を返したとき、全ツールが api の code をそのまま・retriable: false で返す。
#[tokio::test]
async fn all_tools_classify_400_as_non_retriable_with_original_code() {
    for name in all_eleven_tool_names() {
        if name == "health" {
            continue;
        }
        let mock_server = common::start_mock_server().await;
        mount_error_for_tool(
            &mock_server,
            name,
            400,
            "VALIDATION_ERROR",
            "不正なリクエストです",
        )
        .await;
        let api = Arc::new(common::build_client(&mock_server, "internal-key"));
        let (client, url, ct) = spawn_service(api).await;

        let response = call_tool(&client, &url, name, success_arguments_for(name)).await;
        let error = top_level_error(&response);

        assert_eq!(error["code"], "VALIDATION_ERROR", "{name}: {response:?}");
        assert_eq!(error["retriable"], false, "{name}: {response:?}");

        ct.cancel();
    }
}

/// api が 404 を返したとき、全ツールが `outcome: not_found` かつ api の code を返す。
#[tokio::test]
async fn all_tools_classify_404_as_not_found_with_original_code() {
    for name in all_eleven_tool_names() {
        if name == "health" {
            continue;
        }
        let mock_server = common::start_mock_server().await;
        mount_error_for_tool(&mock_server, name, 404, "ITEM_NOT_FOUND", "見つかりません").await;
        let api = Arc::new(common::build_client(&mock_server, "internal-key"));
        let (client, url, ct) = spawn_service(api).await;

        let response = call_tool(&client, &url, name, success_arguments_for(name)).await;
        let error = top_level_error(&response);

        assert_eq!(
            top_level_outcome(&response),
            &json!("not_found"),
            "{name}: {response:?}"
        );
        assert_eq!(error["code"], "ITEM_NOT_FOUND", "{name}: {response:?}");

        ct.cancel();
    }
}

/// 統合テスト3: health だけが接続失敗でも `outcome: success` を返す例外であることを明示する。
#[tokio::test]
async fn health_is_the_only_exception_returning_success_on_connection_failure() {
    let base_url = unreachable_base_url();
    let api = Arc::new(
        ApiClient::new(
            base_url,
            SecretString::from("internal-key".to_string()),
            Duration::from_millis(500),
            Duration::from_millis(500),
        )
        .expect("client should build"),
    );
    let (client, url, ct) = spawn_service(api).await;

    let response = call_tool(&client, &url, "health", json!({})).await;
    assert_eq!(top_level_outcome(&response), &json!("success"));
    assert_eq!(
        response["result"]["structuredContent"]["api"]["reachable"],
        json!(false)
    );

    let other_response = call_tool(
        &client,
        &url,
        "search_library",
        success_arguments_for("search_library"),
    )
    .await;
    assert_eq!(top_level_outcome(&other_response), &json!("error"));

    ct.cancel();
}

// ---------------------------------------------------------------------------
// 実装項目3: 冪等化する409の検証（REQ-112 / REQ-113・統合テスト4）
// ---------------------------------------------------------------------------

async fn mount_success_backend_for_idempotency(mock_server: &MockServer) {
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{ITEM_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": item_detail_json(ITEM_ID)
        })))
        .mount(mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{RELATED_ITEM_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": item_detail_json(RELATED_ITEM_ID)
        })))
        .mount(mock_server)
        .await;
}

fn item_detail_json(id: &str) -> Value {
    json!({
        "id": id,
        "media_type": "anime",
        "title": "冪等性テスト作品",
        "original_title": null,
        "description": null,
        "release_date": null,
        "homepage_url": null,
        "status": "not_started",
        "consumed_date": null,
        "rating": null,
        "is_favorite": false,
        "external_id": null,
        "created_at": "2026-01-01T00:00:00",
        "updated_at": "2026-01-01T00:00:00",
        "detail": null,
        "tags": [],
        "categories": [],
        "streaming_links": []
    })
}

/// `ITEM_ALREADY_IMPORTED` → `import_external_item` は success + `already_existed: true`。
#[tokio::test]
async fn idempotent_409_item_already_imported_becomes_success() {
    let mock_server = common::start_mock_server().await;
    mount_error(
        &mock_server,
        "POST",
        "/api/v1/items/import",
        409,
        "ITEM_ALREADY_IMPORTED",
        "既にインポート済みです",
    )
    .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [],
            "pagination": {"limit": 20, "has_more": false, "next_after_created_at": null, "next_after_id": null, "total": 0}
        })))
        .mount(&mock_server)
        .await;
    let api = Arc::new(common::build_client(&mock_server, "internal-key"));
    let (client, url, ct) = spawn_service(api).await;

    let response = call_tool(
        &client,
        &url,
        "import_external_item",
        success_arguments_for("import_external_item"),
    )
    .await;

    assert_eq!(top_level_outcome(&response), &json!("success"));
    assert_eq!(
        response["result"]["structuredContent"]["already_existed"],
        json!(true)
    );

    ct.cancel();
}

/// `DUPLICATE_RELATION` → `relate_items` は success + `already_related: true`。
#[tokio::test]
async fn idempotent_409_duplicate_relation_becomes_success() {
    let mock_server = common::start_mock_server().await;
    mount_success_backend_for_idempotency(&mock_server).await;
    mount_error(
        &mock_server,
        "POST",
        "/api/v1/item-relations",
        409,
        "DUPLICATE_RELATION",
        "既に関連付けられています",
    )
    .await;
    let api = Arc::new(common::build_client(&mock_server, "internal-key"));
    let (client, url, ct) = spawn_service(api).await;

    let response = call_tool(
        &client,
        &url,
        "relate_items",
        success_arguments_for("relate_items"),
    )
    .await;

    assert_eq!(top_level_outcome(&response), &json!("success"));
    assert_eq!(
        response["result"]["structuredContent"]["already_related"],
        json!(true)
    );

    ct.cancel();
}

/// `DUPLICATE_STREAMING_LINK` → `add_access_link` は success + `already_registered: true`。
#[tokio::test]
async fn idempotent_409_duplicate_streaming_link_becomes_success() {
    let mock_server = common::start_mock_server().await;
    mount_error(
        &mock_server,
        "POST",
        &format!("/api/v1/items/{ITEM_ID}/streaming-links"),
        409,
        "DUPLICATE_STREAMING_LINK",
        "既に登録されています",
    )
    .await;
    let api = Arc::new(common::build_client(&mock_server, "internal-key"));
    let (client, url, ct) = spawn_service(api).await;

    let response = call_tool(
        &client,
        &url,
        "add_access_link",
        json!({
            "item_id": ITEM_ID,
            "url": "https://www.netflix.com/title/12345",
            "kind": "streaming",
            "platform": "netflix"
        }),
    )
    .await;

    assert_eq!(top_level_outcome(&response), &json!("success"));
    assert_eq!(
        response["result"]["structuredContent"]["already_registered"],
        json!(true)
    );

    ct.cancel();
}

/// `DUPLICATE_TAG_NAME` → `organize_item` は再取得して付与へ進む（エラーにしない）。
#[tokio::test]
async fn idempotent_409_duplicate_tag_name_recovers_by_relookup() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{ITEM_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": item_detail_json(ITEM_ID)
        })))
        .mount(&mock_server)
        .await;
    // 1回目の GET（`resolve_tag`）は空、2回目の GET（409後の `find_tag_id_by_name`）は
    // 競合作成者が登録したタグを含める。`up_to_n_times` + `priority` で呼び出し順を固定する。
    Mock::given(method("GET"))
        .and(path("/api/v1/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": []
        })))
        .up_to_n_times(1)
        .with_priority(1)
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [{"id": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0099", "name": "既存タグ", "item_count": 1}]
        })))
        .with_priority(2)
        .mount(&mock_server)
        .await;
    mount_error(
        &mock_server,
        "POST",
        "/api/v1/tags",
        409,
        "DUPLICATE_TAG_NAME",
        "既に存在します",
    )
    .await;
    Mock::given(method("POST"))
        .and(path(
            "/api/v1/items/b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001/tags/b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0099",
        ))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let api = Arc::new(common::build_client(&mock_server, "internal-key"));
    let (client, url, ct) = spawn_service(api).await;

    let response = call_tool(
        &client,
        &url,
        "organize_item",
        json!({"item_id": ITEM_ID, "tags": ["既存タグ"], "create_if_missing": true}),
    )
    .await;

    // 🔵 Intent: 実装項目5「別経路で同時に作られた場合、マスタを再取得してIDを得てから
    //    付与へ進む」。再取得で見つかれば success になる。
    assert_eq!(
        top_level_outcome(&response),
        &json!("success"),
        "response: {response:?}"
    );

    ct.cancel();
}

/// 冪等化しない409: `relate_items` に未知の409コードが返るとエラーとして扱われる。
#[tokio::test]
async fn non_idempotent_409_stays_an_error() {
    let mock_server = common::start_mock_server().await;
    mount_success_backend_for_idempotency(&mock_server).await;
    mount_error(
        &mock_server,
        "POST",
        "/api/v1/item-relations",
        409,
        "SOME_OTHER_CONFLICT",
        "他の競合です",
    )
    .await;
    let api = Arc::new(common::build_client(&mock_server, "internal-key"));
    let (client, url, ct) = spawn_service(api).await;

    let response = call_tool(
        &client,
        &url,
        "relate_items",
        success_arguments_for("relate_items"),
    )
    .await;

    assert_eq!(top_level_outcome(&response), &json!("error"));
    let error = top_level_error(&response);
    assert_eq!(error["code"], "SOME_OTHER_CONFLICT");
    assert_eq!(
        error["retriable"], false,
        "409 は5xxではないため retriable: false のはず"
    );

    ct.cancel();
}

// ---------------------------------------------------------------------------
// 実装項目5: api 全停止時の全ツール動作（REQ-120 / REQ-121・統合テスト5）
// ---------------------------------------------------------------------------

/// EDGE-001再確認: api を停止した状態で11ツールすべてを呼んでもプロセスがクラッシュしない。
#[tokio::test]
async fn all_eleven_tools_survive_when_api_is_completely_down() {
    let base_url = unreachable_base_url();
    let api = Arc::new(
        ApiClient::new(
            base_url,
            SecretString::from("internal-key".to_string()),
            Duration::from_millis(500),
            Duration::from_millis(500),
        )
        .expect("client should build"),
    );
    let (client, url, ct) = spawn_service(api).await;

    for name in all_eleven_tool_names() {
        let response = call_tool(&client, &url, name, success_arguments_for(name)).await;
        assert!(
            response.get("result").is_some(),
            "{name} は応答を返さなかった（プロセスが応答不能になった可能性）: {response:?}"
        );
    }

    ct.cancel();
}

// ---------------------------------------------------------------------------
// 実装項目6: 部分失敗の一貫性（REQ-114・統合テスト6）
// ---------------------------------------------------------------------------

/// `create_item` と `organize_item` の `OperationResult` 集約規則が同一であることを検証する。
/// 両者とも共通の `services::attach` / `result::operation::OperationResult` を使うため、
/// 同じ `result` タグを持つ JSON 形状（`applied` / `already_applied` / `not_resolved` / `failed`）を返す。
#[tokio::test]
async fn create_item_and_organize_item_share_the_same_operation_result_shape() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/items"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "success": true,
            "data": {
                "id": ITEM_ID,
                "media_type": "manga",
                "title": "部分失敗テスト作品",
                "original_title": null,
                "release_date": null,
                "status": "not_started",
                "rating": null,
                "is_favorite": false
            }
        })))
        .mount(&mock_server)
        .await;
    const EXISTING_TAG_ID: &str = "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0099";
    // 🟡 Intent: 両ツールとも `aggregate_outcome` を通した「一部成功・一部失敗」で
    //    partial にする。organize_item は create_item と違い、全滅時は promote されず
    //    error になる（作成済みItemの有無という前提が違うため）ので、意図的に
    //    1件成功させて aggregate_outcome の一般ロジックだけを比較する。
    Mock::given(method("GET"))
        .and(path("/api/v1/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [{"id": EXISTING_TAG_ID, "name": "成功タグ", "item_count": 0}]
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!(
            "/api/v1/items/{ITEM_ID}/tags/{EXISTING_TAG_ID}"
        )))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;
    mount_error(
        &mock_server,
        "POST",
        "/api/v1/tags",
        500,
        "INTERNAL_ERROR",
        "サーバーエラー",
    )
    .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{ITEM_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": item_detail_json(ITEM_ID)
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{ITEM_ID}/mylists")))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"success": true, "data": []})),
        )
        .mount(&mock_server)
        .await;

    let api = Arc::new(common::build_client(&mock_server, "internal-key"));
    let (client, url, ct) = spawn_service(api).await;

    let create_response = call_tool(
        &client,
        &url,
        "create_item",
        json!({"media_type": "manga", "title": "部分失敗テスト作品", "tags": ["成功タグ", "失敗タグ"], "create_if_missing": true}),
    )
    .await;
    let organize_response = call_tool(
        &client,
        &url,
        "organize_item",
        json!({"item_id": ITEM_ID, "tags": ["成功タグ", "失敗タグ"], "create_if_missing": true}),
    )
    .await;

    let create_tags = &create_response["result"]["structuredContent"]["tags"];
    let organize_tags = &organize_response["result"]["structuredContent"]["tags"];

    assert_eq!(top_level_outcome(&create_response), &json!("partial"));
    assert_eq!(top_level_outcome(&organize_response), &json!("partial"));

    let create_shape: Vec<&str> = create_tags
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v["result"].as_str())
        .collect();
    let organize_shape: Vec<&str> = organize_tags
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v["result"].as_str())
        .collect();
    assert_eq!(
        create_shape, organize_shape,
        "create_item と organize_item の OperationResult タグが一致しない"
    );
    assert_eq!(create_shape, vec!["applied", "failed"]);

    ct.cancel();
}

// ---------------------------------------------------------------------------
// 実装項目7: 不正な引数の扱いの一貫性（NFR-201）
// ---------------------------------------------------------------------------

/// EDGE-101再確認: `search_library` と `collection_overview` の両方が limit 超過を
/// 丸めずに `MCP_INVALID_ARGUMENT` で拒否する（片方が丸めて片方が拒否、という不一致がない）。
#[tokio::test]
async fn edge_101_limit_overflow_is_rejected_consistently_across_list_tools() {
    let mock_server = common::start_mock_server().await;
    let api = Arc::new(common::build_client(&mock_server, "internal-key"));
    let (client, url, ct) = spawn_service(api).await;

    let search_response = call_tool(&client, &url, "search_library", json!({"limit": 51})).await;
    let overview_response = call_tool(
        &client,
        &url,
        "collection_overview",
        json!({"recent_limit": 51}),
    )
    .await;

    let search_error = top_level_error(&search_response);
    let overview_error = top_level_error(&overview_response);

    assert_eq!(search_error["code"], "MCP_INVALID_ARGUMENT");
    assert_eq!(overview_error["code"], "MCP_INVALID_ARGUMENT");
    assert_eq!(top_level_outcome(&search_response), &json!("error"));
    assert_eq!(top_level_outcome(&overview_response), &json!("error"));

    // api には到達していない(丸めていない = api へ limit=51 のまま渡すことも、
    // limit=50 に丸めて渡すこともしていない)ことを確認する。
    assert!(mock_server.received_requests().await.unwrap().is_empty());

    ct.cancel();
}

/// 型違反（`item_id` に非UUID文字列）はプロトコルレベルのエラーになる
/// （構造化された `structuredContent.error` ではなく、JSON-RPC / MCP のエラー応答）。
#[tokio::test]
async fn invalid_type_argument_is_rejected_at_protocol_level_not_as_structured_error() {
    let mock_server = common::start_mock_server().await;
    let api = Arc::new(common::build_client(&mock_server, "internal-key"));
    let (client, url, ct) = spawn_service(api).await;

    let response = call_tool(
        &client,
        &url,
        "update_consumption",
        json!({"item_id": "not-a-uuid", "status": "completed"}),
    )
    .await;

    // rmcp のパラメータデコードに失敗するため、`result.structuredContent` を持たない
    // プロトコルレベルのエラー（`result.isError` または JSON-RPC `error`）になるはず。
    let has_structured_content = response["result"]["structuredContent"].is_object();
    assert!(
        !has_structured_content,
        "型違反が structuredContent.error として構造化されてしまっている: {response:?}"
    );
    let is_protocol_error =
        response.get("error").is_some() || response["result"]["isError"] == json!(true);
    assert!(
        is_protocol_error,
        "型違反がプロトコルレベルのエラーとして扱われていない: {response:?}"
    );

    ct.cancel();
}

// ---------------------------------------------------------------------------
// 単体テスト1: retriable 判定の網羅（dataflow.md エラーハンドリングフロー）
// ---------------------------------------------------------------------------
//
// 🔵 Intent: `classify_api_error` そのものの retriable 網羅は
//    `src/result/outcome.rs` の単体テストで既にHTTPステータス×エラー種別の全組み合わせ
//    （Connection / Auth / Api(400,404,422,500,502,503) / Decode）を検証済みのため、
//    ここでは重複させず MCP 層を通した後も retriable が保存されることのみを確認する。

#[test]
fn retriable_flag_survives_serialization_roundtrip() {
    use mediavault_mcp::result::ToolError;

    let error = ToolError {
        code: "MCP_API_UNREACHABLE".to_string(),
        message: "接続に失敗しました".to_string(),
        retriable: true,
    };
    let value = serde_json::to_value(&error).unwrap();
    assert_eq!(value["retriable"], json!(true));

    let non_retriable = ToolError {
        code: "VALIDATION_ERROR".to_string(),
        message: "不正なリクエストです".to_string(),
        retriable: false,
    };
    let value = serde_json::to_value(&non_retriable).unwrap();
    assert_eq!(value["retriable"], json!(false));
}

// ---------------------------------------------------------------------------
// 統合テスト8: Edgeケースの網羅確認
// ---------------------------------------------------------------------------
//
// 🟡 Intent: EDGE-001 / EDGE-002 / EDGE-101 は本タスクで横断的に追加した
//    (`edge_001_*` / `edge_101_*` の関数名を参照)。EDGE-002（5xxを空の成功にしない）は
//    `all_tools_classify_5xx_as_retriable_with_original_code` で全ツール確認済み。
//    その他の EDGE-003〜EDGE-006 / EDGE-102〜EDGE-106 は担当タスク
//    （TASK-0013/0014/0015/0018/0019/0020/0021）で個別に実装済みのため本タスクでは重複実装しない。
