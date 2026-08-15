mod common;

use std::time::Duration;

use mediavault_mcp::api::client::ApiClient;
use mediavault_mcp::config::SecretString;
use mediavault_mcp::result::outcome::Outcome;
use mediavault_mcp::services::item_text::get_item_text;
use mediavault_mcp::tools::get_item_text::GetItemTextParams;
use serde_json::json;
use url::Url;
use uuid::Uuid;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

fn params(item_id: Uuid) -> GetItemTextParams {
    GetItemTextParams {
        item_id,
        file_id: None,
        chunk_index: None,
        chunk_size: None,
    }
}

fn ok(data: serde_json::Value) -> serde_json::Value {
    json!({"success": true, "data": data})
}

fn err(code: &str, message: &str) -> serde_json::Value {
    json!({"success": false, "error": {"code": code, "message": message}})
}

#[tokio::test]
async fn returns_chunk_and_citation_version() {
    let server = common::start_mock_server().await;
    let item_id = Uuid::new_v4();
    let file_id = Uuid::new_v4();
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{item_id}/text")))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok(json!({
            "item_id": item_id,
            "file_id": file_id,
            "extracted_at": "2026-08-15T12:00:00",
            "extraction_version": "pdf-v1",
            "extractor": {"method": "embedded_text"},
            "chunk": {"index": 0, "size": 4000, "total_chunks": 2, "label": "p.1-3", "text": "本文"}
        }))))
        .mount(&server)
        .await;

    let result = get_item_text(&common::build_client(&server, "unused"), params(item_id)).await;
    assert_eq!(result.outcome, Outcome::Success);
    assert_eq!(result.file_id, Some(file_id));
    assert_eq!(result.extraction_version.as_deref(), Some("pdf-v1"));
    let chunk = result.chunk.unwrap();
    assert_eq!((chunk.index, chunk.total_chunks), (0, 2));
    assert_eq!(chunk.label.as_deref(), Some("p.1-3"));
}

#[tokio::test]
async fn text_not_extracted_points_to_request_extraction() {
    let server = common::start_mock_server().await;
    let item_id = Uuid::new_v4();
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(422)
                .set_body_json(err("TEXT_NOT_EXTRACTED", "まだ抽出されていません")),
        )
        .mount(&server)
        .await;
    let result = get_item_text(&common::build_client(&server, "unused"), params(item_id)).await;
    let error = result.error.unwrap();
    assert_eq!(error.code, "TEXT_NOT_EXTRACTED");
    assert!(error.message.contains("request_extraction"));
}

#[tokio::test]
async fn file_not_found_is_not_not_extracted() {
    let server = common::start_mock_server().await;
    let item_id = Uuid::new_v4();
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(404).set_body_json(err("FILE_NOT_FOUND", "ファイルなし")),
        )
        .mount(&server)
        .await;
    let result = get_item_text(&common::build_client(&server, "unused"), params(item_id)).await;
    assert_eq!(result.outcome, Outcome::NotFound);
    let error = result.error.unwrap();
    assert_eq!(error.code, "FILE_NOT_FOUND");
    assert!(!error.message.contains("request_extraction"));
}

#[tokio::test]
async fn ambiguous_file_preserves_candidates() {
    let server = common::start_mock_server().await;
    let item_id = Uuid::new_v4();
    let first = Uuid::new_v4();
    let second = Uuid::new_v4();
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "success": false,
            "error": {
                "code": "AMBIGUOUS_FILE",
                "message": "file_idを指定してください",
                "candidates": [
                    {"file_id": first, "label": "本編", "file_type": "pdf"},
                    {"file_id": second, "label": null, "file_type": "image"}
                ]
            }
        })))
        .mount(&server)
        .await;
    let result = get_item_text(&common::build_client(&server, "unused"), params(item_id)).await;
    assert_eq!(result.outcome, Outcome::Ambiguous);
    assert_eq!(result.candidates.len(), 2);
    assert_eq!(result.candidates[0].file_id, first);
    assert!(result.error.unwrap().message.contains("file_id"));
}

#[tokio::test]
async fn unreachable_api_has_distinct_error_code() {
    let api = ApiClient::new(
        Url::parse("http://127.0.0.1:9").unwrap(),
        SecretString::from("unused".to_string()),
        Duration::from_millis(50),
        Duration::from_millis(50),
    )
    .unwrap();
    let result = get_item_text(&api, params(Uuid::new_v4())).await;
    assert_eq!(result.error.unwrap().code, "MCP_API_UNREACHABLE");
}

#[tokio::test]
async fn passes_all_optional_query_parameters() {
    let server = common::start_mock_server().await;
    let item_id = Uuid::new_v4();
    let file_id = Uuid::new_v4();
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{item_id}/text")))
        .and(query_param("file_id", file_id.to_string()))
        .and(query_param("chunk_index", "3"))
        .and(query_param("chunk_size", "1200"))
        .respond_with(ResponseTemplate::new(404).set_body_json(err("FILE_NOT_FOUND", "なし")))
        .expect(1)
        .mount(&server)
        .await;
    let mut input = params(item_id);
    input.file_id = Some(file_id);
    input.chunk_index = Some(3);
    input.chunk_size = Some(1200);
    let _ = get_item_text(&common::build_client(&server, "unused"), input).await;
}

#[tokio::test]
async fn omitted_chunk_values_are_not_filled_by_mcp() {
    let server = common::start_mock_server().await;
    let item_id = Uuid::new_v4();
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{item_id}/text")))
        .respond_with(ResponseTemplate::new(404).set_body_json(err("FILE_NOT_FOUND", "なし")))
        .expect(1)
        .mount(&server)
        .await;
    let _ = get_item_text(&common::build_client(&server, "unused"), params(item_id)).await;
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    assert!(requests[0].url.query().is_none());
}
