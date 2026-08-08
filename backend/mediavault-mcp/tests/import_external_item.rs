//! `import_external_item` の統合テスト（wiremock）
//!
//! TASK-0017: import_external_item ツールの実装

use mediavault_mcp::result::outcome::Outcome;
use mediavault_mcp::services::import_external_item::import;
use mediavault_mcp::tools::import_external_item::ImportExternalItemParams;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

mod common;

fn item_json(id: &str, external_id: &str, media_type: &str, title: &str) -> serde_json::Value {
    json!({
        "id": id,
        "media_type": media_type,
        "title": title,
        "original_title": null,
        "release_date": null,
        "status": "not_started",
        "rating": null,
        "is_favorite": false,
        "source": "api",
        "external_id": external_id
    })
}

fn item_with_refs_json(
    id: &str,
    external_id: &str,
    media_type: &str,
    title: &str,
) -> serde_json::Value {
    let mut value = item_json(id, external_id, media_type, title);
    value["tags"] = json!([]);
    value["categories"] = json!([]);
    value
}

fn params(
    media_type: mediavault_mcp::api::models::MediaType,
    external_id: &str,
) -> ImportExternalItemParams {
    ImportExternalItemParams {
        media_type,
        external_id: external_id.to_string(),
        provider: Some("annict".to_string()),
    }
}

/// 統合テスト1: 新規登録
#[tokio::test]
async fn imports_new_item_and_returns_summary() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/items/import"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "success": true,
            "data": item_json("b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001", "12345", "anime", "作品A")
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = import(
        &api,
        params(mediavault_mcp::api::models::MediaType::Anime, "12345"),
    )
    .await;

    assert_eq!(result.outcome, Outcome::Success);
    assert!(!result.already_existed);
    let item = result.item.expect("item should be present");
    assert_eq!(item.title, "作品A");
}

/// 統合テスト2: 既にインポート済み(既存Itemを特定できる)
#[tokio::test]
async fn already_imported_returns_existing_item_without_error() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/items/import"))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "success": false,
            "error": {"code": "ITEM_ALREADY_IMPORTED", "message": "既にインポート済みです"}
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [item_with_refs_json("b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001", "12345", "anime", "作品A")],
            "pagination": {"limit": 20, "has_more": false, "next_after_created_at": null, "next_after_id": null, "total": 1}
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = import(
        &api,
        params(mediavault_mcp::api::models::MediaType::Anime, "12345"),
    )
    .await;

    assert_eq!(result.outcome, Outcome::Success);
    assert!(result.already_existed);
    let item = result.item.expect("existing item should be found");
    assert_eq!(item.title, "作品A");
}

/// 統合テスト3: 409 だが既存Itemが特定できない
#[tokio::test]
async fn already_imported_without_findable_existing_item_returns_null_item() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/items/import"))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "success": false,
            "error": {"code": "ITEM_ALREADY_IMPORTED", "message": "既にインポート済みです"}
        })))
        .mount(&mock_server)
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
    let api = common::build_client(&mock_server, "internal-key");

    let result = import(
        &api,
        params(mediavault_mcp::api::models::MediaType::Anime, "12345"),
    )
    .await;

    assert_eq!(result.outcome, Outcome::Success);
    assert!(result.already_existed);
    assert!(result.item.is_none());
    assert!(result.error.is_none());
}

/// 統合テスト4: 走査のページ数上限
#[tokio::test]
async fn scan_stops_at_page_limit_and_does_not_loop_forever() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/items/import"))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "success": false,
            "error": {"code": "ITEM_ALREADY_IMPORTED", "message": "既にインポート済みです"}
        })))
        .mount(&mock_server)
        .await;
    // 常に has_more: true を返す(external_id は一致しない) → 上限ページで打ち切られること
    Mock::given(method("GET"))
        .and(path("/api/v1/items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [item_with_refs_json("b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001", "99999", "anime", "別の作品")],
            "pagination": {
                "limit": 20,
                "has_more": true,
                "next_after_created_at": "2026-07-01T12:00:00",
                "next_after_id": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0002",
                "total": 1000
            }
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = import(
        &api,
        params(mediavault_mcp::api::models::MediaType::Anime, "12345"),
    )
    .await;

    assert_eq!(result.outcome, Outcome::Success);
    assert!(result.already_existed);
    assert!(result.item.is_none());

    let requests = mock_server.received_requests().await.unwrap();
    let scan_requests = requests
        .iter()
        .filter(|req| req.url.path() == "/api/v1/items")
        .count();
    assert_eq!(scan_requests, 5, "5ページで打ち切られること");
}

/// 統合テスト5: external_id が空
#[tokio::test]
async fn validation_error_is_passed_through_transparently() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/items/import"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "success": false,
            "error": {"code": "VALIDATION_ERROR", "message": "external_id は必須です"}
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = import(
        &api,
        params(mediavault_mcp::api::models::MediaType::Anime, ""),
    )
    .await;

    assert_eq!(result.outcome, Outcome::Error);
    let error = result.error.unwrap();
    assert_eq!(error.code, "VALIDATION_ERROR");
    assert!(!error.retriable);
}

/// 統合テスト6: プロバイダ側で見つからない
#[tokio::test]
async fn not_found_by_provider_maps_to_not_found_outcome() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/items/import"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "success": false,
            "error": {"code": "ITEM_NOT_FOUND", "message": "プロバイダ側で見つかりません"}
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = import(
        &api,
        params(mediavault_mcp::api::models::MediaType::Anime, "99999"),
    )
    .await;

    assert_eq!(result.outcome, Outcome::NotFound);
}

/// 統合テスト7: リトライしない(POST は1回のみ)
#[tokio::test]
async fn does_not_retry_the_write_on_connection_failure() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/items/import"))
        .respond_with(ResponseTemplate::new(502).set_body_json(json!({
            "success": false,
            "error": {"code": "EXTERNAL_API_TIMEOUT", "message": "外部APIがタイムアウトしました"}
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = import(
        &api,
        params(mediavault_mcp::api::models::MediaType::Anime, "12345"),
    )
    .await;

    assert_eq!(result.outcome, Outcome::Error);
    assert!(result.error.unwrap().retriable);

    let requests = mock_server.received_requests().await.unwrap();
    let post_count = requests
        .iter()
        .filter(|req| req.url.path() == "/api/v1/items/import")
        .count();
    assert_eq!(post_count, 1, "POST は1回のみで再送しないこと");
}

/// 統合テスト8: annotation
#[tokio::test]
async fn tools_list_marks_import_external_item_as_writable_and_non_destructive() {
    use axum::Router;
    use rmcp::transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    };
    use tokio_util::sync::CancellationToken;

    let mock_server = common::start_mock_server().await;
    let api = std::sync::Arc::new(common::build_client(&mock_server, "internal-key"));

    let ct = CancellationToken::new();
    let config = StreamableHttpServerConfig::default()
        .with_json_response(true)
        .with_sse_keep_alive(None)
        .with_cancellation_token(ct.child_token());
    let service = StreamableHttpService::new(
        move || Ok(mediavault_mcp::server::MediaVaultServer::new(api.clone())),
        std::sync::Arc::new(LocalSessionManager::default()),
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
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-03-26",
                "capabilities": {},
                "clientInfo": {"name": "test-client", "version": "1.0.0"}
            }
        }))
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
    let response = list_request
        .send()
        .await
        .expect("tools/list should succeed");
    let text = response.text().await.unwrap();
    let body: serde_json::Value = if let Ok(value) = serde_json::from_str(&text) {
        value
    } else {
        text.lines()
            .filter_map(|line| line.strip_prefix("data: "))
            .filter_map(|data| serde_json::from_str::<serde_json::Value>(data).ok())
            .find(|value| value.get("result").is_some())
            .expect("response should contain a JSON-RPC result")
    };

    let tools = body["result"]["tools"].as_array().unwrap();
    let tool = tools
        .iter()
        .find(|tool| tool["name"] == "import_external_item")
        .expect("import_external_item should be listed");
    assert_eq!(tool["annotations"]["readOnlyHint"], json!(false));
    assert_eq!(tool["annotations"]["destructiveHint"], json!(false));

    ct.cancel();
}

/// 統合テスト9: 呼び出し回数(候補選択を除き2回で完了する: search_external_catalog → import_external_item)
#[tokio::test]
async fn completes_within_two_tool_calls_after_candidate_selection() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/items/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [{
                "id": "12345",
                "media_type": "anime",
                "provider": "annict",
                "title": "作品A",
                "thumbnail_url": null,
                "year": 2023
            }]
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/items/import"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "success": true,
            "data": item_json("b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001", "12345", "anime", "作品A")
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let search_result = mediavault_mcp::services::search_external_catalog::search(
        &api,
        mediavault_mcp::tools::search_external_catalog::SearchExternalCatalogParams {
            media_type: mediavault_mcp::api::models::MediaType::Anime,
            q: "作品A".to_string(),
        },
    )
    .await;
    let candidate = &search_result.candidates[0];

    let import_result = import(
        &api,
        ImportExternalItemParams {
            media_type: candidate.media_type,
            external_id: candidate.external_id.clone(),
            provider: candidate.provider.clone(),
        },
    )
    .await;

    assert_eq!(import_result.outcome, Outcome::Success);
    assert!(!import_result.already_existed);
}
