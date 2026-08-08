//! `relate_items` の統合テスト（wiremock）
//!
//! TASK-0021: relate_items ツールの実装

use mediavault_mcp::api::models::RelationType;
use mediavault_mcp::result::outcome::Outcome;
use mediavault_mcp::services::relate_items::relate;
use mediavault_mcp::tools::relate_items::RelateItemsParams;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

mod common;

const ITEM_ID: &str = "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001";
const RELATED_ITEM_ID: &str = "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0002";
const RELATION_ID: &str = "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0003";

fn params(relation_type: RelationType) -> RelateItemsParams {
    RelateItemsParams {
        item_id: ITEM_ID.parse().unwrap(),
        related_item_id: RELATED_ITEM_ID.parse().unwrap(),
        relation_type,
    }
}

fn item_detail_json(id: &str, title: &str) -> serde_json::Value {
    json!({
        "id": id,
        "media_type": "novel",
        "title": title,
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
    })
}

async fn mount_items(mock_server: &wiremock::MockServer) {
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{ITEM_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": item_detail_json(ITEM_ID, "小説A"),
        })))
        .mount(mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{RELATED_ITEM_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": item_detail_json(RELATED_ITEM_ID, "映画A"),
        })))
        .mount(mock_server)
        .await;
}

async fn mount_create_relation_success(mock_server: &wiremock::MockServer) {
    Mock::given(method("POST"))
        .and(path("/api/v1/item-relations"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "success": true,
            "data": {
                "id": RELATION_ID,
                "item_id": ITEM_ID,
                "related_item_id": RELATED_ITEM_ID,
                "relation_type": "adaptation",
                "created_at": "2026-01-01T00:00:00",
            }
        })))
        .mount(mock_server)
        .await;
}

/// 統合テスト1: 6種別すべての登録
#[tokio::test]
async fn registers_all_six_relation_types() {
    for relation_type in [
        RelationType::Adaptation,
        RelationType::Sequel,
        RelationType::Prequel,
        RelationType::Spinoff,
        RelationType::Dlc,
        RelationType::Reference,
    ] {
        let mock_server = common::start_mock_server().await;
        mount_items(&mock_server).await;
        mount_create_relation_success(&mock_server).await;
        let api = common::build_client(&mock_server, "internal-key");

        let result = relate(&api, params(relation_type)).await;

        assert_eq!(result.outcome, Outcome::Success);
        assert!(!result.description.is_empty());
        assert_eq!(result.relation_id, Some(RELATION_ID.parse().unwrap()));
    }
}

/// 統合テスト2: 重複登録（冪等）
#[tokio::test]
async fn duplicate_relation_is_idempotent_and_not_an_error() {
    let mock_server = common::start_mock_server().await;
    mount_items(&mock_server).await;
    Mock::given(method("POST"))
        .and(path("/api/v1/item-relations"))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "success": false,
            "error": {"code": "DUPLICATE_RELATION", "message": "既に関連付けられています"}
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = relate(&api, params(RelationType::Adaptation)).await;

    assert_eq!(result.outcome, Outcome::Success);
    assert!(result.already_related);
    assert!(result.relation_id.is_none());
    assert!(result.error.is_none());
}

/// 統合テスト3: 自己参照
#[tokio::test]
async fn self_reference_is_rejected_without_calling_api() {
    let mock_server = common::start_mock_server().await;
    let api = common::build_client(&mock_server, "internal-key");

    let mut request = params(RelationType::Adaptation);
    request.related_item_id = request.item_id;

    let result = relate(&api, request).await;

    let error = result.error.expect("error should be present");
    assert_eq!(error.code, "MCP_INVALID_ARGUMENT");
    let requests = mock_server.received_requests().await.unwrap();
    assert!(requests.is_empty());
}

/// 統合テスト4: 片方の Item が存在しない
#[tokio::test]
async fn missing_related_item_returns_not_found_without_posting() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{ITEM_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": item_detail_json(ITEM_ID, "小説A"),
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{RELATED_ITEM_ID}")))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "success": false,
            "error": {"code": "ITEM_NOT_FOUND", "message": "見つかりません"}
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = relate(&api, params(RelationType::Adaptation)).await;

    assert_eq!(result.outcome, Outcome::NotFound);
    let requests = mock_server.received_requests().await.unwrap();
    assert!(
        requests
            .iter()
            .all(|req| req.url.path() != "/api/v1/item-relations")
    );
}

/// 統合テスト5: 種別違いは別の関係として両方登録される
#[tokio::test]
async fn different_relation_types_are_both_registered() {
    let mock_server = common::start_mock_server().await;
    mount_items(&mock_server).await;
    mount_create_relation_success(&mock_server).await;
    let api = common::build_client(&mock_server, "internal-key");

    let first = relate(&api, params(RelationType::Adaptation)).await;
    let second = relate(&api, params(RelationType::Reference)).await;

    assert_eq!(first.outcome, Outcome::Success);
    assert_eq!(second.outcome, Outcome::Success);
    assert!(!first.already_related);
    assert!(!second.already_related);
}

/// 統合テスト6: 両 Item の並列取得
#[tokio::test]
async fn fetches_both_items_before_posting() {
    let mock_server = common::start_mock_server().await;
    mount_items(&mock_server).await;
    mount_create_relation_success(&mock_server).await;
    let api = common::build_client(&mock_server, "internal-key");

    let _ = relate(&api, params(RelationType::Adaptation)).await;

    let requests = mock_server.received_requests().await.unwrap();
    let get_count = requests
        .iter()
        .filter(|req| req.method.as_str() == "GET")
        .count();
    assert_eq!(get_count, 2);
}

/// 統合テスト7: リトライしない。POST /item-relations が1回のみ
#[tokio::test]
async fn does_not_retry_the_write_call() {
    let mock_server = common::start_mock_server().await;
    mount_items(&mock_server).await;
    mount_create_relation_success(&mock_server).await;
    let api = common::build_client(&mock_server, "internal-key");

    let _ = relate(&api, params(RelationType::Adaptation)).await;

    let requests = mock_server.received_requests().await.unwrap();
    let post_count = requests
        .iter()
        .filter(|req| req.method.as_str() == "POST" && req.url.path() == "/api/v1/item-relations")
        .count();
    assert_eq!(post_count, 1);
}

/// 統合テスト8: 拡張前の api での失敗（400 が透過される）
#[tokio::test]
async fn api_validation_error_is_passed_through_as_non_retriable() {
    let mock_server = common::start_mock_server().await;
    mount_items(&mock_server).await;
    Mock::given(method("POST"))
        .and(path("/api/v1/item-relations"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "success": false,
            "error": {"code": "VALIDATION_ERROR", "message": "不正な relation_type です"}
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = relate(&api, params(RelationType::Adaptation)).await;

    assert_eq!(result.outcome, Outcome::Error);
    let error = result.error.expect("error should be present");
    assert_eq!(error.code, "VALIDATION_ERROR");
    assert!(!error.retriable);
}
