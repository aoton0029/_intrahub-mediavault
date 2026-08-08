//! `update_consumption` の統合テスト（wiremock）
//!
//! TASK-0019: update_consumption ツールの実装

use mediavault_mcp::api::models::ItemStatus;
use mediavault_mcp::result::outcome::Outcome;
use mediavault_mcp::services::update_consumption::update;
use mediavault_mcp::tools::update_consumption::UpdateConsumptionParams;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

mod common;

const ITEM_ID: &str = "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001";

fn item_detail_json(
    status: &str,
    consumed_date: Option<&str>,
    rating: Option<f32>,
    is_favorite: bool,
) -> serde_json::Value {
    json!({
        "id": ITEM_ID,
        "media_type": "anime",
        "title": "作品A",
        "original_title": null,
        "description": null,
        "release_date": null,
        "homepage_url": null,
        "status": status,
        "consumed_date": consumed_date,
        "rating": rating,
        "is_favorite": is_favorite,
        "external_id": null,
        "created_at": "2026-01-01T00:00:00",
        "updated_at": "2026-01-01T00:00:00",
        "detail": null,
        "tags": [],
        "categories": [],
        "streaming_links": []
    })
}

fn params(
    status: Option<ItemStatus>,
    consumed_date: Option<chrono::NaiveDate>,
    rating: Option<f32>,
    is_favorite: Option<bool>,
) -> UpdateConsumptionParams {
    UpdateConsumptionParams {
        item_id: ITEM_ID.parse().unwrap(),
        status,
        consumed_date,
        rating,
        is_favorite,
    }
}

/// 統合テスト1: 4項目を1回で更新
#[tokio::test]
async fn updates_all_four_fields_in_a_single_patch() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{ITEM_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": item_detail_json("in_progress", None, Some(7.0), false)
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("/api/v1/items/{ITEM_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": item_detail_json("completed", Some("2026-08-06"), Some(8.0), true)
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = update(
        &api,
        params(
            Some(ItemStatus::Completed),
            Some("2026-08-06".parse().unwrap()),
            Some(8.0),
            Some(true),
        ),
    )
    .await;

    assert_eq!(result.outcome, Outcome::Success);
    assert_eq!(result.title, Some("作品A".to_string()));
    assert_eq!(result.changes.len(), 4);

    let requests = mock_server.received_requests().await.unwrap();
    let patch_requests: Vec<_> = requests
        .iter()
        .filter(|req| req.method.as_str() == "PATCH")
        .collect();
    assert_eq!(patch_requests.len(), 1);
    let body: serde_json::Value = patch_requests[0].body_json().unwrap();
    assert_eq!(body["status"], "completed");
    assert_eq!(body["consumed_date"], "2026-08-06");
    assert_eq!(body["rating"], 8.0);
    assert_eq!(body["is_favorite"], true);
}

/// 統合テスト2: 一部項目のみ更新（他のフィールドは送らない・nullも送らない）
#[tokio::test]
async fn sends_only_specified_field_in_patch_body() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{ITEM_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": item_detail_json("in_progress", None, Some(7.0), false)
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("/api/v1/items/{ITEM_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": item_detail_json("in_progress", None, Some(7.0), true)
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = update(&api, params(None, None, None, Some(true))).await;

    assert_eq!(result.outcome, Outcome::Success);

    let requests = mock_server.received_requests().await.unwrap();
    let patch_requests: Vec<_> = requests
        .iter()
        .filter(|req| req.method.as_str() == "PATCH")
        .collect();
    assert_eq!(patch_requests.len(), 1);
    let body: serde_json::Value = patch_requests[0].body_json().unwrap();
    let object = body.as_object().unwrap();
    assert_eq!(object.len(), 1);
    assert_eq!(object["is_favorite"], true);
}

/// 統合テスト3: 更新前後の値
#[tokio::test]
async fn returns_before_and_after_values_with_title() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{ITEM_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": item_detail_json("in_progress", None, Some(7.0), false)
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("/api/v1/items/{ITEM_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": item_detail_json("completed", None, Some(8.0), false)
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = update(
        &api,
        params(Some(ItemStatus::Completed), None, Some(8.0), None),
    )
    .await;

    assert_eq!(result.title, Some("作品A".to_string()));
    let status_change = result
        .changes
        .iter()
        .find(|change| change.field == "status")
        .expect("status should be in changes");
    assert_eq!(status_change.before, Some(json!("in_progress")));
    assert_eq!(status_change.after, Some(json!("completed")));
}

/// 統合テスト4: 存在しない Item
#[tokio::test]
async fn not_found_item_does_not_call_patch() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{ITEM_ID}")))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "success": false,
            "error": {"code": "ITEM_NOT_FOUND", "message": "見つかりません"}
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = update(&api, params(Some(ItemStatus::Completed), None, None, None)).await;

    assert_eq!(result.outcome, Outcome::NotFound);
    assert_eq!(result.error.unwrap().code, "ITEM_NOT_FOUND");
    assert!(result.changes.is_empty());

    let requests = mock_server.received_requests().await.unwrap();
    let patch_count = requests
        .iter()
        .filter(|req| req.method.as_str() == "PATCH")
        .count();
    assert_eq!(patch_count, 0, "PATCH は呼ばれないこと");
}

/// 統合テスト5: PATCH が失敗
#[tokio::test]
async fn patch_failure_returns_error_with_empty_changes() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{ITEM_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": item_detail_json("in_progress", None, Some(7.0), false)
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("/api/v1/items/{ITEM_ID}")))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "success": false,
            "error": {"code": "VALIDATION_ERROR", "message": "不正な値です"}
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = update(&api, params(Some(ItemStatus::Completed), None, None, None)).await;

    assert_eq!(result.outcome, Outcome::Error);
    let error = result.error.unwrap();
    assert_eq!(error.code, "VALIDATION_ERROR");
    assert!(result.changes.is_empty());
}

/// 統合テスト6: rating の範囲外（api のバリデーションエラーが透過する）
#[tokio::test]
async fn rating_out_of_range_is_passed_through_from_api() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{ITEM_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": item_detail_json("in_progress", None, None, false)
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("/api/v1/items/{ITEM_ID}")))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "success": false,
            "error": {"code": "VALIDATION_ERROR", "message": "rating は0〜10の範囲です"}
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = update(&api, params(None, None, Some(999.0), None)).await;

    assert_eq!(result.outcome, Outcome::Error);
    assert_eq!(result.error.unwrap().code, "VALIDATION_ERROR");
}

/// 統合テスト7: リトライしない（PATCH は1回のみ）
#[tokio::test]
async fn does_not_retry_the_patch_on_failure() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{ITEM_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": item_detail_json("in_progress", None, None, false)
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("/api/v1/items/{ITEM_ID}")))
        .respond_with(ResponseTemplate::new(502).set_body_json(json!({
            "success": false,
            "error": {"code": "INTERNAL_ERROR", "message": "サーバーエラー"}
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = update(&api, params(Some(ItemStatus::Completed), None, None, None)).await;

    assert_eq!(result.outcome, Outcome::Error);
    assert!(result.error.unwrap().retriable);

    let requests = mock_server.received_requests().await.unwrap();
    let patch_count = requests
        .iter()
        .filter(|req| req.method.as_str() == "PATCH")
        .count();
    assert_eq!(patch_count, 1, "PATCH は1回のみで再送しないこと");
}

/// 統合テスト8: MCPツール呼び出し1回で完了する（api は GET + PATCH の2本）
#[tokio::test]
async fn completes_within_a_single_tool_call() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{ITEM_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": item_detail_json("in_progress", None, None, false)
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(format!("/api/v1/items/{ITEM_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": item_detail_json("completed", None, None, false)
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = update(&api, params(Some(ItemStatus::Completed), None, None, None)).await;

    assert_eq!(result.outcome, Outcome::Success);

    let requests = mock_server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 2, "GET + PATCH の2本のみ");
}
