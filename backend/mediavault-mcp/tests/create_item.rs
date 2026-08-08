//! `create_item` の統合テスト（wiremock）
//!
//! TASK-0018: create_item ツールの実装

use mediavault_mcp::api::models::MediaType;
use mediavault_mcp::result::operation::OperationResult;
use mediavault_mcp::result::outcome::Outcome;
use mediavault_mcp::services::create_item::create;
use mediavault_mcp::tools::create_item::CreateItemParams;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

mod common;

fn item_json(id: &str, media_type: &str, title: &str) -> serde_json::Value {
    json!({
        "id": id,
        "media_type": media_type,
        "title": title,
        "original_title": null,
        "release_date": null,
        "status": "not_started",
        "rating": null,
        "is_favorite": false,
        "source": "manual"
    })
}

fn minimal_params(media_type: MediaType, title: &str) -> CreateItemParams {
    CreateItemParams {
        media_type,
        title: title.to_string(),
        original_title: None,
        description: None,
        release_date: None,
        homepage_url: None,
        rating: None,
        is_favorite: None,
        file_paths: Vec::new(),
        urls: Vec::new(),
        tags: Vec::new(),
        mylists: Vec::new(),
        create_if_missing: false,
    }
}

const ITEM_ID: &str = "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001";

/// 統合テスト1: 最小構成での登録
#[tokio::test]
async fn creates_item_with_media_type_and_title_only() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/items"))
        .respond_with(ResponseTemplate::new(201).set_body_json(
            json!({"success": true, "data": item_json(ITEM_ID, "manga", "同人誌A")}),
        ))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = create(&api, minimal_params(MediaType::Manga, "同人誌A")).await;

    assert_eq!(result.outcome, Outcome::Success);
    let item = result.item.expect("item should be present");
    assert_eq!(item.title, "同人誌A");
    assert_eq!(item.media_type, MediaType::Manga);
    assert!(result.tags.is_empty());
    assert!(result.mylists.is_empty());
    assert!(result.files.is_empty());
    assert!(result.links.is_empty());
}

/// 統合テスト2: 全項目の関連付け
#[tokio::test]
async fn associates_files_urls_tags_and_mylists_in_one_call() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/items"))
        .respond_with(ResponseTemplate::new(201).set_body_json(
            json!({"success": true, "data": item_json(ITEM_ID, "manga", "同人誌A")}),
        ))
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("/api/v1/items/{ITEM_ID}/files")))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "success": true,
            "data": {"id": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0002", "path": "/srv/manga/a.pdf", "file_type": "pdf"}
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("/api/v1/items/{ITEM_ID}/links")))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "success": true,
            "data": {"id": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0003", "url": "https://example.com", "label": "https://example.com"}
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [{"id": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0004", "name": "同人", "item_count": 1}]
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!(
            "/api/v1/items/{ITEM_ID}/tags/b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0004"
        )))
        .respond_with(ResponseTemplate::new(201))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/mylists"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [{"id": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0005", "name": "夏コミ"}]
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path(
            "/api/v1/mylists/b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0005/items",
        ))
        .respond_with(ResponseTemplate::new(201))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let mut params = minimal_params(MediaType::Manga, "同人誌A");
    params.file_paths = vec!["/srv/manga/a.pdf".to_string()];
    params.urls = vec!["https://example.com".to_string()];
    params.tags = vec!["同人".to_string()];
    params.mylists = vec!["夏コミ".to_string()];

    let result = create(&api, params).await;

    assert_eq!(result.outcome, Outcome::Success);
    assert!(result.item.is_some());
    assert!(matches!(result.files[0], OperationResult::Applied { .. }));
    assert!(matches!(result.links[0], OperationResult::Applied { .. }));
    assert!(matches!(result.tags[0], OperationResult::Applied { .. }));
    assert!(matches!(result.mylists[0], OperationResult::Applied { .. }));
}

/// 統合テスト3: Item 作成後にタグ付与が失敗
#[tokio::test]
async fn returns_item_id_even_when_tag_attach_fails() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/items"))
        .respond_with(ResponseTemplate::new(201).set_body_json(
            json!({"success": true, "data": item_json(ITEM_ID, "manga", "同人誌A")}),
        ))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [{"id": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0004", "name": "同人", "item_count": 1}]
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!(
            "/api/v1/items/{ITEM_ID}/tags/b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0004"
        )))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "success": false,
            "error": {"code": "INTERNAL_ERROR", "message": "サーバーエラー"}
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let mut params = minimal_params(MediaType::Manga, "同人誌A");
    params.tags = vec!["同人".to_string()];

    let result = create(&api, params).await;

    let item = result.item.expect("item id should not be lost");
    assert_eq!(item.item_id.to_string(), ITEM_ID);
    assert!(matches!(result.tags[0], OperationResult::Failed { .. }));
    assert_eq!(result.outcome, Outcome::Partial);

    let requests = mock_server.received_requests().await.unwrap();
    assert!(requests.iter().all(|req| req.method.as_str() != "DELETE"));
}

/// 統合テスト4: create_if_missing 未指定
#[tokio::test]
async fn tag_not_found_without_create_if_missing_does_not_create_tag() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/items"))
        .respond_with(ResponseTemplate::new(201).set_body_json(
            json!({"success": true, "data": item_json(ITEM_ID, "manga", "同人誌A")}),
        ))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": []
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let mut params = minimal_params(MediaType::Manga, "同人誌A");
    params.tags = vec!["存在しないタグ".to_string()];

    let result = create(&api, params).await;

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

/// 統合テスト5: create_if_missing = true
#[tokio::test]
async fn creates_missing_tag_when_create_if_missing_true() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/items"))
        .respond_with(ResponseTemplate::new(201).set_body_json(
            json!({"success": true, "data": item_json(ITEM_ID, "manga", "同人誌A")}),
        ))
        .mount(&mock_server)
        .await;
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
            "data": {"id": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0004", "name": "新タグ"}
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!(
            "/api/v1/items/{ITEM_ID}/tags/b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0004"
        )))
        .respond_with(ResponseTemplate::new(201))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let mut params = minimal_params(MediaType::Manga, "同人誌A");
    params.tags = vec!["新タグ".to_string()];
    params.create_if_missing = true;

    let result = create(&api, params).await;

    match &result.tags[0] {
        OperationResult::Applied { created_new, .. } => assert!(created_new),
        other => panic!("expected Applied, got {other:?}"),
    }
}

/// 統合テスト6: Item 作成自体が失敗
#[tokio::test]
async fn item_creation_failure_stops_before_associations() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/items"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "success": false,
            "error": {"code": "VALIDATION_ERROR", "message": "titleは必須です"}
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let mut params = minimal_params(MediaType::Manga, "同人誌A");
    params.tags = vec!["同人".to_string()];
    params.mylists = vec!["夏コミ".to_string()];
    params.urls = vec!["https://example.com".to_string()];
    params.file_paths = vec!["/srv/manga/a.pdf".to_string()];

    let result = create(&api, params).await;

    assert_eq!(result.outcome, Outcome::Error);
    assert!(result.item.is_none());
    assert!(result.tags.is_empty());
    assert!(result.mylists.is_empty());
    assert!(result.files.is_empty());
    assert!(result.links.is_empty());

    let requests = mock_server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1, "関連付けの api が一切呼ばれないこと");
}

/// 統合テスト7: タイトルが空白のみ
#[tokio::test]
async fn blank_title_validation_error_is_passed_through() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/items"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "success": false,
            "error": {"code": "VALIDATION_ERROR", "message": "titleは空白のみ不可です"}
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = create(&api, minimal_params(MediaType::Manga, "   ")).await;

    assert_eq!(result.outcome, Outcome::Error);
    let error = result.error.unwrap();
    assert_eq!(error.code, "VALIDATION_ERROR");
}

/// 統合テスト8: 削除系が呼ばれない（あらゆる失敗パターンで確認）
#[tokio::test]
async fn never_calls_delete_across_failure_modes() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/items"))
        .respond_with(ResponseTemplate::new(201).set_body_json(
            json!({"success": true, "data": item_json(ITEM_ID, "manga", "同人誌A")}),
        ))
        .mount(&mock_server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("/api/v1/items/{ITEM_ID}/files")))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "success": false,
            "error": {"code": "INTERNAL_ERROR", "message": "サーバーエラー"}
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/tags"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "success": false,
            "error": {"code": "INTERNAL_ERROR", "message": "サーバーエラー"}
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/mylists"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "success": false,
            "error": {"code": "INTERNAL_ERROR", "message": "サーバーエラー"}
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let mut params = minimal_params(MediaType::Manga, "同人誌A");
    params.file_paths = vec!["/srv/manga/a.pdf".to_string()];
    params.urls = vec!["not-a-url".to_string(), "ftp://x".to_string()];
    params.tags = vec!["同人".to_string()];
    params.mylists = vec!["夏コミ".to_string()];

    let result = create(&api, params).await;

    assert!(result.item.is_some());
    assert_eq!(result.outcome, Outcome::Partial);
    assert!(matches!(result.files[0], OperationResult::Failed { .. }));
    assert!(matches!(result.links[0], OperationResult::Failed { .. }));
    assert!(matches!(result.links[1], OperationResult::Failed { .. }));
    assert!(matches!(result.tags[0], OperationResult::Failed { .. }));
    assert!(matches!(result.mylists[0], OperationResult::Failed { .. }));

    let requests = mock_server.received_requests().await.unwrap();
    assert!(requests.iter().all(|req| req.method.as_str() != "DELETE"));
}
