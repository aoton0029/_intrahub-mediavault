//! `search_external_catalog` の統合テスト（wiremock）
//!
//! TASK-0015: search_external_catalog ツールの実装

use mediavault_mcp::services::search_external_catalog::search;
use mediavault_mcp::services::search_library;
use mediavault_mcp::tools::search_external_catalog::SearchExternalCatalogParams;
use mediavault_mcp::tools::search_library::SearchLibraryParams;
use serde_json::json;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

mod common;

fn search_result_json(
    id: &str,
    provider: Option<&str>,
    title: &str,
    media_type: &str,
    year: Option<i32>,
) -> serde_json::Value {
    json!({
        "id": id,
        "media_type": media_type,
        "provider": provider,
        "title": title,
        "thumbnail_url": null,
        "year": year
    })
}

fn params(
    media_type: mediavault_mcp::api::models::MediaType,
    q: &str,
) -> SearchExternalCatalogParams {
    SearchExternalCatalogParams {
        media_type,
        q: q.to_string(),
    }
}

/// 統合テスト1: 候補が返る
#[tokio::test]
async fn returns_candidates_that_carry_provider_and_external_id() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/items/search"))
        .and(query_param("media_type", "anime"))
        .and(query_param("q", "作品A"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [search_result_json("12345", Some("annict"), "作品A", "anime", Some(2023))]
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = search(
        &api,
        params(mediavault_mcp::api::models::MediaType::Anime, "作品A"),
    )
    .await;

    assert_eq!(
        result.outcome,
        mediavault_mcp::result::outcome::Outcome::Success
    );
    assert_eq!(result.source, "external_catalog");
    assert_eq!(result.candidates.len(), 1);
    let candidate = &result.candidates[0];
    assert_eq!(candidate.external_id, "12345");
    assert_eq!(candidate.provider.as_deref(), Some("annict"));
    assert_eq!(candidate.release_year, Some(2023));
}

/// 統合テスト2: ローカル検索との型の違い
#[tokio::test]
async fn differs_from_search_library_in_source_and_item_id_presence() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/items/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [search_result_json("12345", Some("annict"), "作品A", "anime", Some(2023))]
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

    let external_result = search(
        &api,
        params(mediavault_mcp::api::models::MediaType::Anime, "作品A"),
    )
    .await;
    let library_result = search_library::search(&api, SearchLibraryParams::default()).await;

    let external_value = serde_json::to_value(&external_result).unwrap();
    let library_value = serde_json::to_value(&library_result).unwrap();

    assert_eq!(external_value["source"], json!("external_catalog"));
    assert_eq!(library_value["source"], json!("mediavault_library"));
    assert!(external_value["candidates"][0].get("item_id").is_none());
}

/// 統合テスト3: APIキー未設定
#[tokio::test]
async fn api_key_not_configured_returns_transparent_error() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/items/search"))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "success": false,
            "error": {
                "code": "API_KEY_NOT_CONFIGURED",
                "message": "Annict の API キーが設定されていません"
            }
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = search(
        &api,
        params(mediavault_mcp::api::models::MediaType::Anime, "作品A"),
    )
    .await;

    assert_eq!(
        result.outcome,
        mediavault_mcp::result::outcome::Outcome::Error
    );
    let error = result.error.unwrap();
    assert_eq!(error.code, "API_KEY_NOT_CONFIGURED");
    assert!(!error.retriable);
    assert_eq!(error.message, "Annict の API キーが設定されていません");
}

/// 統合テスト4: 外部APIタイムアウト
#[tokio::test]
async fn external_api_timeout_is_retriable() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/items/search"))
        .respond_with(ResponseTemplate::new(502).set_body_json(json!({
            "success": false,
            "error": {
                "code": "EXTERNAL_API_TIMEOUT",
                "message": "外部APIがタイムアウトしました"
            }
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = search(
        &api,
        params(mediavault_mcp::api::models::MediaType::Anime, "作品A"),
    )
    .await;

    let error = result.error.unwrap();
    assert_eq!(error.code, "EXTERNAL_API_TIMEOUT");
    assert!(error.retriable);
}

/// 統合テスト5: 0件
#[tokio::test]
async fn zero_results_is_success_with_empty_candidates() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/items/search"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"success": true, "data": []})),
        )
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = search(
        &api,
        params(
            mediavault_mcp::api::models::MediaType::Anime,
            "存在しない作品",
        ),
    )
    .await;

    assert_eq!(
        result.outcome,
        mediavault_mcp::result::outcome::Outcome::Success
    );
    assert!(result.candidates.is_empty());
    assert_eq!(result.source, "external_catalog");
}

/// 統合テスト7: tools/list の annotation
#[tokio::test]
async fn tools_list_marks_search_external_catalog_as_read_only() {
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
        .find(|tool| tool["name"] == "search_external_catalog")
        .expect("search_external_catalog should be listed");
    assert_eq!(tool["annotations"]["readOnlyHint"], json!(true));

    ct.cancel();
}

/// テストケース6: media_type 未指定はスキーマ上必須のためツール層で拒否される
///（api を呼ばない）ことを型で保証する。`SearchExternalCatalogParams::media_type` は
/// `Option` ではないため、コンパイルが通ること自体がこの要件を満たす。
#[test]
fn media_type_is_required_by_type() {
    fn assert_required(_p: SearchExternalCatalogParams) {}
    let _ = assert_required;
}
