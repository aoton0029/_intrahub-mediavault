//! `add_access_link` の統合テスト（wiremock）
//!
//! TASK-0022: add_access_link ツールの実装

mod common;

use mediavault_mcp::result::outcome::Outcome;
use mediavault_mcp::services::add_access_link::add_access_link;
use mediavault_mcp::tools::add_access_link::{AddAccessLinkParams, LinkKind, RegisteredAs};
use serde_json::json;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

const ITEM_ID: &str = "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001";
const LINK_ID: &str = "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0002";

fn params(kind: LinkKind, url: &str) -> AddAccessLinkParams {
    AddAccessLinkParams {
        item_id: ITEM_ID.parse().unwrap(),
        url: url.to_string(),
        kind,
        platform: None,
        label: None,
    }
}

/// 統合テスト1: 対応プラットフォームの配信リンク
#[tokio::test]
async fn registers_streaming_link_for_supported_platform() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!("/api/v1/items/{ITEM_ID}/streaming-links")))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "success": true,
            "data": {"id": LINK_ID, "platform": "netflix", "url": "https://netflix.com/watch/1"},
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let mut request = params(LinkKind::Streaming, "https://netflix.com/watch/1");
    request.platform = Some("netflix".to_string());

    let result = add_access_link(&api, request).await;

    assert_eq!(result.outcome, Outcome::Success);
    assert!(matches!(
        result.registered_as,
        Some(RegisteredAs::StreamingLink)
    ));
    assert!(result.fallback_from.is_none());
    assert_eq!(result.link_id, Some(LINK_ID.parse::<Uuid>().unwrap()));
}

/// 統合テスト2: 非対応プラットフォームのフォールバック
#[tokio::test]
async fn falls_back_to_link_for_unsupported_platform() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!("/api/v1/items/{ITEM_ID}/links")))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "success": true,
            "data": {"id": LINK_ID, "url": "https://video.unext.jp/title/1", "label": "u-next"},
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let mut request = params(LinkKind::Streaming, "https://video.unext.jp/title/1");
    request.platform = Some("u-next".to_string());

    let result = add_access_link(&api, request).await;

    assert_eq!(result.outcome, Outcome::Success);
    assert!(matches!(result.registered_as, Some(RegisteredAs::Link)));
    assert_eq!(result.fallback_from, Some("streaming".to_string()));

    let requests = mock_server.received_requests().await.unwrap();
    assert!(
        requests
            .iter()
            .all(|req| req.url.path() != format!("/api/v1/items/{ITEM_ID}/streaming-links"))
    );
}

/// 統合テスト3: トレーラー登録
#[tokio::test]
async fn registers_trailer() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!("/api/v1/items/{ITEM_ID}/trailers")))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "success": true,
            "data": {"id": LINK_ID, "url": "https://youtube.com/watch?v=1", "label": null},
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = add_access_link(
        &api,
        params(LinkKind::Trailer, "https://youtube.com/watch?v=1"),
    )
    .await;

    assert_eq!(result.outcome, Outcome::Success);
    assert!(matches!(result.registered_as, Some(RegisteredAs::Trailer)));
}

/// 統合テスト4: Jellyfin / Calibre-Web の URL（通常リンク・ラベル付き）
#[tokio::test]
async fn registers_link_with_explicit_label() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!("/api/v1/items/{ITEM_ID}/links")))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "success": true,
            "data": {"id": LINK_ID, "url": "http://jellyfin.local:8096/", "label": "Jellyfin"},
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let mut request = params(LinkKind::Link, "http://jellyfin.local:8096/");
    request.label = Some("Jellyfin".to_string());

    let result = add_access_link(&api, request).await;

    assert_eq!(result.outcome, Outcome::Success);
    assert!(matches!(result.registered_as, Some(RegisteredAs::Link)));
    assert!(result.fallback_from.is_none());
}

/// 統合テスト5: 不正なURL
#[tokio::test]
async fn invalid_url_is_rejected_without_calling_api() {
    let mock_server = common::start_mock_server().await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = add_access_link(&api, params(LinkKind::Link, "javascript:alert(1)")).await;

    let error = result.error.expect("error should be present");
    assert_eq!(error.code, "MCP_INVALID_ARGUMENT");
    let requests = mock_server.received_requests().await.unwrap();
    assert!(requests.is_empty());
}

/// 統合テスト6: 配信リンクの重複
#[tokio::test]
async fn duplicate_streaming_link_is_idempotent() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!("/api/v1/items/{ITEM_ID}/streaming-links")))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "success": false,
            "error": {"code": "DUPLICATE_STREAMING_LINK", "message": "既に登録されています"}
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let mut request = params(LinkKind::Streaming, "https://netflix.com/watch/1");
    request.platform = Some("netflix".to_string());

    let result = add_access_link(&api, request).await;

    assert_eq!(result.outcome, Outcome::Success);
    assert!(result.already_registered);
    assert!(result.link_id.is_none());
    assert!(result.error.is_none());
}

/// 統合テスト7: 存在しない Item
#[tokio::test]
async fn missing_item_returns_not_found() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!("/api/v1/items/{ITEM_ID}/links")))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "success": false,
            "error": {"code": "ITEM_NOT_FOUND", "message": "見つかりません"}
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let mut request = params(LinkKind::Link, "https://example.com");
    request.label = Some("公式サイト".to_string());

    let result = add_access_link(&api, request).await;

    assert_eq!(result.outcome, Outcome::NotFound);
    let error = result.error.expect("error should be present");
    assert_eq!(error.code, "ITEM_NOT_FOUND");
}

/// 統合テスト8: 不正な platform を api が拒否した場合、そのまま透過する
#[tokio::test]
async fn api_validation_error_is_passed_through() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!("/api/v1/items/{ITEM_ID}/streaming-links")))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "success": false,
            "error": {"code": "VALIDATION_ERROR", "message": "urlが空文字です"}
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let mut request = params(LinkKind::Streaming, "https://netflix.com/watch/1");
    request.platform = Some("netflix".to_string());

    let result = add_access_link(&api, request).await;

    assert_eq!(result.outcome, Outcome::Error);
    let error = result.error.expect("error should be present");
    assert_eq!(error.code, "VALIDATION_ERROR");
    assert!(!error.retriable);
}

/// 統合テスト9: リトライしない。POST が1回のみ
#[tokio::test]
async fn does_not_retry_the_write_call() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path(format!("/api/v1/items/{ITEM_ID}/trailers")))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "success": true,
            "data": {"id": LINK_ID, "url": "https://youtube.com/watch?v=1", "label": null},
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let _ = add_access_link(
        &api,
        params(LinkKind::Trailer, "https://youtube.com/watch?v=1"),
    )
    .await;

    let requests = mock_server.received_requests().await.unwrap();
    let post_count = requests
        .iter()
        .filter(|req| req.method.as_str() == "POST" && req.url.path().ends_with("/trailers"))
        .count();
    assert_eq!(post_count, 1);
}
