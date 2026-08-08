//! `collection_overview` の統合テスト（wiremock）
//!
//! TASK-0016: collection_overview ツールの実装

use mediavault_mcp::result::outcome::Outcome;
use mediavault_mcp::services::collection_overview::get_overview;
use mediavault_mcp::tools::collection_overview::CollectionOverviewParams;
use serde_json::json;
use uuid::Uuid;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

mod common;

fn item_json(id: Uuid, title: &str, media_type: &str, status: &str) -> serde_json::Value {
    json!({
        "id": id.to_string(),
        "media_type": media_type,
        "title": title,
        "original_title": null,
        "release_date": "2023-04-01",
        "status": status,
        "rating": null,
        "is_favorite": false,
        "tags": [],
        "categories": []
    })
}

fn overview_response(
    total_items: i64,
    favorite_count: i64,
    by_media_type: Vec<serde_json::Value>,
    by_status: Vec<serde_json::Value>,
    recently_added: Vec<serde_json::Value>,
    recently_updated: Vec<serde_json::Value>,
) -> serde_json::Value {
    json!({
        "success": true,
        "data": {
            "total_items": total_items,
            "favorite_count": favorite_count,
            "by_media_type": by_media_type,
            "by_status": by_status,
            "recently_added": recently_added,
            "recently_updated": recently_updated
        }
    })
}

/// 統合テスト1: 全項目が返る
#[tokio::test]
async fn returns_all_fields() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/collection/overview"))
        .respond_with(ResponseTemplate::new(200).set_body_json(overview_response(
            181,
            34,
            vec![
                json!({"key": "manga", "count": 87}),
                json!({"key": "anime", "count": 42}),
            ],
            vec![
                json!({"key": "not_started", "count": 90}),
                json!({"key": "in_progress", "count": 21}),
                json!({"key": "completed", "count": 70}),
            ],
            vec![item_json(Uuid::new_v4(), "作品A", "anime", "in_progress")],
            vec![item_json(Uuid::new_v4(), "作品B", "manga", "completed")],
        )))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = get_overview(&api, CollectionOverviewParams::default()).await;

    assert_eq!(result.outcome, Outcome::Success);
    assert_eq!(result.total_items, 181);
    assert_eq!(result.favorite_count, 34);
    assert_eq!(result.by_media_type.len(), 2);
    assert_eq!(result.by_status.len(), 3);
    assert_eq!(result.recently_added.len(), 1);
    assert_eq!(result.recently_updated.len(), 1);
}

/// 統合テスト2: status別内訳（3種すべて含まれる）
#[tokio::test]
async fn by_status_includes_all_three_kinds() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/collection/overview"))
        .respond_with(ResponseTemplate::new(200).set_body_json(overview_response(
            0,
            0,
            vec![],
            vec![
                json!({"key": "not_started", "count": 0}),
                json!({"key": "in_progress", "count": 0}),
                json!({"key": "completed", "count": 0}),
            ],
            vec![],
            vec![],
        )))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = get_overview(&api, CollectionOverviewParams::default()).await;

    let keys: Vec<&str> = result.by_status.iter().map(|e| e.key.as_str()).collect();
    assert_eq!(keys, vec!["not_started", "in_progress", "completed"]);
}

/// 統合テスト3: recent_limit=5 が api へ渡り、最近の Item が5件以内で返る
#[tokio::test]
async fn recent_limit_is_forwarded_and_bounded() {
    let mock_server = common::start_mock_server().await;
    let items: Vec<_> = (0..5)
        .map(|_| item_json(Uuid::new_v4(), "作品", "anime", "not_started"))
        .collect();
    Mock::given(method("GET"))
        .and(path("/api/v1/collection/overview"))
        .and(query_param("recent_limit", "5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(overview_response(
            5,
            0,
            vec![],
            vec![],
            items.clone(),
            items,
        )))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = get_overview(
        &api,
        CollectionOverviewParams {
            recent_limit: Some(5),
        },
    )
    .await;

    assert_eq!(result.outcome, Outcome::Success);
    assert!(result.recently_added.len() <= 5);
    assert!(result.recently_updated.len() <= 5);
}

/// 統合テスト4: 集計API のエラー（500）は outcome: error として透過する（EDGE-002）
#[tokio::test]
async fn api_500_returns_error_outcome_not_empty_success() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/collection/overview"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "success": false,
            "error": {"code": "INTERNAL_ERROR", "message": "サーバーエラー"}
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = get_overview(&api, CollectionOverviewParams::default()).await;

    assert_eq!(result.outcome, Outcome::Error);
    let error = result.error.unwrap();
    assert_eq!(error.code, "INTERNAL_ERROR");
    assert_eq!(result.total_items, 0);
}

/// 統合テスト5: 接続エラー
#[tokio::test]
async fn connection_failure_returns_api_unreachable_and_retriable() {
    let mock_server = common::start_mock_server().await;
    drop(mock_server);
    let api = mediavault_mcp::api::client::ApiClient::new(
        url::Url::parse("http://127.0.0.1:1").unwrap(),
        mediavault_mcp::config::SecretString::from("internal-key".to_string()),
        std::time::Duration::from_millis(200),
        std::time::Duration::from_millis(200),
    )
    .unwrap();

    let result = get_overview(&api, CollectionOverviewParams::default()).await;

    assert_eq!(result.outcome, Outcome::Error);
    let error = result.error.unwrap();
    assert_eq!(error.code, "MCP_API_UNREACHABLE");
    assert!(error.retriable);
}

/// 統合テスト6: 空コレクションはエラーにせず0件の成功として返る
#[tokio::test]
async fn empty_collection_returns_success_with_zero_counts() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/collection/overview"))
        .respond_with(ResponseTemplate::new(200).set_body_json(overview_response(
            0,
            0,
            vec![],
            vec![
                json!({"key": "not_started", "count": 0}),
                json!({"key": "in_progress", "count": 0}),
                json!({"key": "completed", "count": 0}),
            ],
            vec![],
            vec![],
        )))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = get_overview(&api, CollectionOverviewParams::default()).await;

    assert_eq!(result.outcome, Outcome::Success);
    assert_eq!(result.total_items, 0);
    assert!(result.recently_added.is_empty());
    assert!(result.recently_updated.is_empty());
}

/// 統合テスト7: 呼び出し回数（`counts-by-media-type` を併用しない）
#[tokio::test]
async fn calls_overview_endpoint_exactly_once() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/collection/overview"))
        .respond_with(ResponseTemplate::new(200).set_body_json(overview_response(
            0,
            0,
            vec![],
            vec![],
            vec![],
            vec![],
        )))
        .expect(1)
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/items/counts-by-media-type"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let _ = get_overview(&api, CollectionOverviewParams::default()).await;
}

/// テストケース1: recent_limit のバリデーション（統合経路）
#[tokio::test]
async fn recent_limit_51_returns_invalid_argument_without_calling_api() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/collection/overview"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = get_overview(
        &api,
        CollectionOverviewParams {
            recent_limit: Some(51),
        },
    )
    .await;

    assert_eq!(result.outcome, Outcome::Error);
    assert_eq!(result.error.unwrap().code, "MCP_INVALID_ARGUMENT");
}

#[tokio::test]
async fn recent_limit_0_returns_invalid_argument_without_calling_api() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/collection/overview"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = get_overview(
        &api,
        CollectionOverviewParams {
            recent_limit: Some(0),
        },
    )
    .await;

    assert_eq!(result.outcome, Outcome::Error);
    assert_eq!(result.error.unwrap().code, "MCP_INVALID_ARGUMENT");
}
