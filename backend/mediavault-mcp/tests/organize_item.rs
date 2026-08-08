//! `organize_item` の統合テスト（wiremock）
//!
//! TASK-0020: organize_item ツールの実装

use mediavault_mcp::result::operation::OperationResult;
use mediavault_mcp::result::outcome::Outcome;
use mediavault_mcp::services::organize::organize;
use mediavault_mcp::tools::organize_item::OrganizeItemParams;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

mod common;

const ITEM_ID: &str = "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001";
const TAG_ID: &str = "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0004";
const CATEGORY_ID: &str = "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0006";
const MYLIST_ID: &str = "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0005";

fn minimal_params() -> OrganizeItemParams {
    OrganizeItemParams {
        item_id: ITEM_ID.parse().unwrap(),
        tags: Vec::new(),
        categories: Vec::new(),
        mylists: Vec::new(),
        create_if_missing: false,
    }
}

fn item_detail_json(tags: serde_json::Value, categories: serde_json::Value) -> serde_json::Value {
    json!({
        "id": ITEM_ID,
        "media_type": "manga",
        "title": "同人誌A",
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
        "tags": tags,
        "categories": categories,
    })
}

async fn mount_item_detail(
    mock_server: &wiremock::MockServer,
    tags: serde_json::Value,
    categories: serde_json::Value,
) {
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{ITEM_ID}")))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(
                json!({"success": true, "data": item_detail_json(tags, categories)}),
            ),
        )
        .mount(mock_server)
        .await;
}

/// 統合テスト1: 既存タグの付与
#[tokio::test]
async fn attaches_existing_tag_resolved_by_name() {
    let mock_server = common::start_mock_server().await;
    mount_item_detail(&mock_server, json!([]), json!([])).await;
    Mock::given(method("GET"))
        .and(path("/api/v1/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [{"id": TAG_ID, "name": "積読", "item_count": 1}]
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("/api/v1/items/{ITEM_ID}/tags/{TAG_ID}")))
        .respond_with(ResponseTemplate::new(201))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let mut params = minimal_params();
    params.tags = vec!["積読".to_string()];

    let result = organize(&api, params).await;

    match &result.tags[0] {
        OperationResult::Applied {
            created_new,
            target_name,
            ..
        } => {
            assert!(!created_new);
            assert_eq!(target_name, "積読");
        }
        other => panic!("expected Applied, got {other:?}"),
    }
    assert_eq!(result.outcome, Outcome::Success);
}

/// 統合テスト2: create_if_missing で新規作成
#[tokio::test]
async fn creates_missing_tag_when_create_if_missing_true() {
    let mock_server = common::start_mock_server().await;
    mount_item_detail(&mock_server, json!([]), json!([])).await;
    Mock::given(method("GET"))
        .and(path("/api/v1/tags"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"success": true, "data": []})),
        )
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/tags"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "success": true,
            "data": {"id": TAG_ID, "name": "積読"}
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("/api/v1/items/{ITEM_ID}/tags/{TAG_ID}")))
        .respond_with(ResponseTemplate::new(201))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let mut params = minimal_params();
    params.tags = vec!["積読".to_string()];
    params.create_if_missing = true;

    let result = organize(&api, params).await;

    match &result.tags[0] {
        OperationResult::Applied { created_new, .. } => assert!(created_new),
        other => panic!("expected Applied, got {other:?}"),
    }
}

/// 統合テスト3: 既に付与済み（冪等）。POST /items/{id}/tags/{tag_id} が呼ばれないこと。
#[tokio::test]
async fn already_attached_tag_is_idempotent_and_does_not_reattach() {
    let mock_server = common::start_mock_server().await;
    mount_item_detail(
        &mock_server,
        json!([{"id": TAG_ID, "name": "積読"}]),
        json!([]),
    )
    .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [{"id": TAG_ID, "name": "積読", "item_count": 1}]
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let mut params = minimal_params();
    params.tags = vec!["積読".to_string()];

    let result = organize(&api, params).await;

    assert!(matches!(
        result.tags[0],
        OperationResult::AlreadyApplied { .. }
    ));
    let requests = mock_server.received_requests().await.unwrap();
    assert!(
        requests
            .iter()
            .all(|req| req.url.path() != format!("/api/v1/items/{ITEM_ID}/tags/{TAG_ID}"))
    );
}

/// 統合テスト4: create_if_missing 未指定で候補なし。POST /tags が呼ばれないこと。
#[tokio::test]
async fn tag_not_found_without_create_if_missing_does_not_create() {
    let mock_server = common::start_mock_server().await;
    mount_item_detail(&mock_server, json!([]), json!([])).await;
    Mock::given(method("GET"))
        .and(path("/api/v1/tags"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"success": true, "data": []})),
        )
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let mut params = minimal_params();
    params.tags = vec!["存在しないタグ".to_string()];

    let result = organize(&api, params).await;

    assert!(matches!(
        result.tags[0],
        OperationResult::NotResolved { .. }
    ));
    let requests = mock_server.received_requests().await.unwrap();
    assert!(
        requests
            .iter()
            .all(|req| req.url.path() != "/api/v1/tags" || req.method.as_str() != "POST")
    );
}

/// 統合テスト5: 3種を1回で処理
#[tokio::test]
async fn processes_tags_categories_and_mylists_in_one_call() {
    let mock_server = common::start_mock_server().await;
    mount_item_detail(&mock_server, json!([]), json!([])).await;
    Mock::given(method("GET"))
        .and(path("/api/v1/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [{"id": TAG_ID, "name": "積読", "item_count": 1}]
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("/api/v1/items/{ITEM_ID}/tags/{TAG_ID}")))
        .respond_with(ResponseTemplate::new(201))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/categories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [{"id": CATEGORY_ID, "name": "映画", "item_count": 1}]
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!(
            "/api/v1/items/{ITEM_ID}/categories/{CATEGORY_ID}"
        )))
        .respond_with(ResponseTemplate::new(201))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{ITEM_ID}/mylists")))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"success": true, "data": []})),
        )
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/mylists"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [{"id": MYLIST_ID, "name": "夏コミ"}]
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("/api/v1/mylists/{MYLIST_ID}/items")))
        .respond_with(ResponseTemplate::new(201))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let mut params = minimal_params();
    params.tags = vec!["積読".to_string()];
    params.categories = vec!["映画".to_string()];
    params.mylists = vec!["夏コミ".to_string()];

    let result = organize(&api, params).await;

    assert_eq!(result.outcome, Outcome::Success);
    assert!(matches!(result.tags[0], OperationResult::Applied { .. }));
    assert!(matches!(
        result.categories[0],
        OperationResult::Applied { .. }
    ));
    assert!(matches!(result.mylists[0], OperationResult::Applied { .. }));
}

/// 統合テスト6: 一部失敗
#[tokio::test]
async fn partial_failure_marks_outcome_as_partial() {
    let mock_server = common::start_mock_server().await;
    mount_item_detail(&mock_server, json!([]), json!([])).await;
    Mock::given(method("GET"))
        .and(path("/api/v1/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [
                {"id": TAG_ID, "name": "積読", "item_count": 1},
                {"id": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0007", "name": "映画", "item_count": 1}
            ]
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("/api/v1/items/{ITEM_ID}/tags/{TAG_ID}")))
        .respond_with(ResponseTemplate::new(201))
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!(
            "/api/v1/items/{ITEM_ID}/tags/b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0007"
        )))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "success": false,
            "error": {"code": "INTERNAL_ERROR", "message": "サーバーエラー"}
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let mut params = minimal_params();
    params.tags = vec!["積読".to_string(), "映画".to_string()];

    let result = organize(&api, params).await;

    assert!(matches!(result.tags[0], OperationResult::Applied { .. }));
    assert!(matches!(result.tags[1], OperationResult::Failed { .. }));
    assert_eq!(result.outcome, Outcome::Partial);
}

/// 統合テスト9: マイリストのエンドポイント方向
#[tokio::test]
async fn attaches_mylist_via_mylist_owned_endpoint() {
    let mock_server = common::start_mock_server().await;
    mount_item_detail(&mock_server, json!([]), json!([])).await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{ITEM_ID}/mylists")))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"success": true, "data": []})),
        )
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/mylists"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [{"id": MYLIST_ID, "name": "夏コミ"}]
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("/api/v1/mylists/{MYLIST_ID}/items")))
        .respond_with(ResponseTemplate::new(201))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let mut params = minimal_params();
    params.mylists = vec!["夏コミ".to_string()];

    let result = organize(&api, params).await;

    assert!(matches!(result.mylists[0], OperationResult::Applied { .. }));

    let requests = mock_server.received_requests().await.unwrap();
    let mylist_add_request = requests
        .iter()
        .find(|req| req.url.path() == format!("/api/v1/mylists/{MYLIST_ID}/items"))
        .expect("POST /mylists/{id}/items should be called");
    let body: serde_json::Value = serde_json::from_slice(&mylist_add_request.body).unwrap();
    assert_eq!(body["item_id"], ITEM_ID);
    assert!(requests.iter().all(|req| req.url.path()
        != format!("/api/v1/items/{ITEM_ID}/mylists")
        || req.method.as_str() != "POST"));
}

/// 統合テスト10: あらゆる失敗パターンで DELETE が発行されない
#[tokio::test]
async fn never_calls_delete_across_failure_modes() {
    let mock_server = common::start_mock_server().await;
    mount_item_detail(&mock_server, json!([]), json!([])).await;
    Mock::given(method("GET"))
        .and(path("/api/v1/tags"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "success": false,
            "error": {"code": "INTERNAL_ERROR", "message": "サーバーエラー"}
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let mut params = minimal_params();
    params.tags = vec!["積読".to_string()];

    let result = organize(&api, params).await;

    assert!(matches!(result.tags[0], OperationResult::Failed { .. }));
    let requests = mock_server.received_requests().await.unwrap();
    assert!(requests.iter().all(|req| req.method.as_str() != "DELETE"));
}

/// 統合テスト11: 呼び出し内メモ化。タグ3件を指定しても GET /tags は1回のみ。
#[tokio::test]
async fn memoizes_master_lookup_within_single_call() {
    let mock_server = common::start_mock_server().await;
    mount_item_detail(&mock_server, json!([]), json!([])).await;
    Mock::given(method("GET"))
        .and(path("/api/v1/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [
                {"id": TAG_ID, "name": "積読", "item_count": 1},
                {"id": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0007", "name": "映画", "item_count": 1},
                {"id": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0008", "name": "SF", "item_count": 1}
            ]
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(wiremock::matchers::path_regex(format!(
            "^/api/v1/items/{ITEM_ID}/tags/.+$"
        )))
        .respond_with(ResponseTemplate::new(201))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let mut params = minimal_params();
    params.tags = vec!["積読".to_string(), "映画".to_string(), "SF".to_string()];

    let result = organize(&api, params).await;

    assert!(
        result
            .tags
            .iter()
            .all(|r| matches!(r, OperationResult::Applied { .. }))
    );
    let requests = mock_server.received_requests().await.unwrap();
    let tag_master_calls = requests
        .iter()
        .filter(|req| req.url.path() == "/api/v1/tags" && req.method.as_str() == "GET")
        .count();
    assert_eq!(tag_master_calls, 1);
}

/// 統合テスト7: マスタ取得の最適化。tags のみ指定時は GET /categories / GET /mylists が呼ばれない。
#[tokio::test]
async fn does_not_fetch_unrequested_masters() {
    let mock_server = common::start_mock_server().await;
    mount_item_detail(&mock_server, json!([]), json!([])).await;
    Mock::given(method("GET"))
        .and(path("/api/v1/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [{"id": TAG_ID, "name": "積読", "item_count": 1}]
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("/api/v1/items/{ITEM_ID}/tags/{TAG_ID}")))
        .respond_with(ResponseTemplate::new(201))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let mut params = minimal_params();
    params.tags = vec!["積読".to_string()];

    let result = organize(&api, params).await;

    assert!(matches!(result.tags[0], OperationResult::Applied { .. }));
    let requests = mock_server.received_requests().await.unwrap();
    assert!(
        requests.iter().all(
            |req| req.url.path() != "/api/v1/categories" && req.url.path() != "/api/v1/mylists"
        )
    );
}

/// 統合テスト8: 作成競合（409）。再取得で見つかれば付与まで進む。
#[tokio::test]
async fn duplicate_tag_creation_conflict_resolves_via_refetch() {
    let mock_server = common::start_mock_server().await;
    mount_item_detail(&mock_server, json!([]), json!([])).await;
    Mock::given(method("GET"))
        .and(path("/api/v1/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": []
        })))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v1/tags"))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "success": false,
            "error": {"code": "DUPLICATE_TAG_NAME", "message": "既に存在します"}
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [{"id": TAG_ID, "name": "積読", "item_count": 1}]
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("/api/v1/items/{ITEM_ID}/tags/{TAG_ID}")))
        .respond_with(ResponseTemplate::new(201))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let mut params = minimal_params();
    params.tags = vec!["積読".to_string()];
    params.create_if_missing = true;

    let result = organize(&api, params).await;

    // 🟡 Intent: 409 競合時、実際に作成したのは別プロセスだが、このツール呼び出しから見れば
    //    「解決できなかった名前を create_if_missing で作成しようとした結果」であり
    //    created_new: true として扱う（タスクファイル実装項目5「エラーにしない」の範囲内の判断）。
    match &result.tags[0] {
        OperationResult::Applied { created_new, .. } => assert!(created_new),
        other => panic!("expected Applied, got {other:?}"),
    }
}
