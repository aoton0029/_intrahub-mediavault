use mediavault_mcp::services::extraction;
use mediavault_mcp::tools::extraction::{ExtractionNextAction, ExtractionParams, ExtractionState};
use serde_json::json;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

mod common;

fn params() -> ExtractionParams {
    ExtractionParams {
        item_id: Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap(),
        file_id: Uuid::parse_str("22222222-2222-4222-8222-222222222222").unwrap(),
    }
}

fn response(state: &str, attempts: i32, error: serde_json::Value) -> serde_json::Value {
    json!({
        "success": true,
        "data": {
            "id": "33333333-3333-4333-8333-333333333333",
            "item_file_id": params().file_id,
            "state": state,
            "attempts": attempts,
            "max_attempts": 3,
            "progress_current": 2,
            "progress_total": 10,
            "error": error,
            "created_at": "2026-08-15T12:00:00",
            "updated_at": "2026-08-15T12:01:00"
        }
    })
}

fn extraction_path() -> String {
    format!(
        "/api/v1/items/{}/files/{}/extraction",
        params().item_id,
        params().file_id
    )
}

#[tokio::test]
async fn request_accepts_created_without_authorization_header() {
    let server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path(extraction_path()))
        .respond_with(ResponseTemplate::new(201).set_body_json(response("queued", 0, json!(null))))
        .expect(1)
        .mount(&server)
        .await;
    let result =
        extraction::request(&common::build_client(&server, "must-not-leak"), params()).await;
    assert_eq!(result.state, Some(ExtractionState::Queued));
    assert_eq!(result.next_action, ExtractionNextAction::Wait);
    assert!(result.error.is_none());
    let requests = server.received_requests().await.unwrap();
    assert!(requests[0].headers.get("authorization").is_none());
}

#[tokio::test]
async fn request_treats_existing_200_as_success() {
    let server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path(extraction_path()))
        .respond_with(ResponseTemplate::new(200).set_body_json(response("running", 1, json!(null))))
        .mount(&server)
        .await;
    let result = extraction::request(&common::build_client(&server, "key"), params()).await;
    assert_eq!(result.state, Some(ExtractionState::Running));
    assert!(result.error.is_none());
}

#[tokio::test]
async fn status_translates_retryable_and_exhausted_failures() {
    for (attempts, expected) in [
        (1, ExtractionNextAction::Wait),
        (3, ExtractionNextAction::GiveUp),
    ] {
        let server = common::start_mock_server().await;
        Mock::given(method("GET"))
            .and(path(extraction_path()))
            .respond_with(ResponseTemplate::new(200).set_body_json(response(
                "failed",
                attempts,
                json!({"kind": "ocr_failed", "message": "OCR failed", "retryable": true}),
            )))
            .mount(&server)
            .await;
        let result = extraction::status(&common::build_client(&server, "key"), params()).await;
        assert_eq!(result.next_action, expected);
        assert_eq!(result.progress_current, Some(2));
        assert_eq!(result.progress_total, Some(10));
    }
}

#[tokio::test]
async fn cancel_calls_public_endpoint_and_finished_is_success() {
    let server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!("{}/cancel", extraction_path())))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "success": false,
            "error": {"code": "EXTRACTION_ALREADY_FINISHED", "message": "finished"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let result =
        extraction::cancel(&common::build_client(&server, "must-not-leak"), params()).await;
    assert!(result.error.is_none());
    assert!(result.message.contains("すでに終了"));
    let requests = server.received_requests().await.unwrap();
    assert!(requests[0].headers.get("authorization").is_none());
}

#[tokio::test]
async fn unsupported_type_tells_agent_not_to_retry() {
    let server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path(extraction_path()))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "success": false,
            "error": {"code": "UNSUPPORTED_FILE_TYPE", "message": "unsupported"}
        })))
        .mount(&server)
        .await;
    let result = extraction::request(&common::build_client(&server, "key"), params()).await;
    assert_eq!(result.next_action, ExtractionNextAction::UseAnotherFile);
    assert!(!result.error.unwrap().retriable);
}
