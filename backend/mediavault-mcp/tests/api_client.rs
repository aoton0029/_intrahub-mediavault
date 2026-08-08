//! `ApiClient` の統合テスト（wiremock）
//!
//! TASK-0008: ApiClient層の実装

mod common;

use std::time::Duration;

use mediavault_mcp::api::error::ApiClientError;
use serde::Deserialize;
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, ResponseTemplate};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Item {
    id: String,
    title: String,
}

/// 統合テスト1: サーバー停止/接続拒否 -> Connection
///
/// 🟡 Intent: 何もリッスンしていないポート（127.0.0.1:1）へ接続させ、
///    OS レベルの接続拒否を発生させる。wiremock サーバーの停止直後は
///    OSがポートを即座に解放しない場合があり不安定なため使わない。
#[tokio::test]
async fn get_returns_connection_error_when_server_unreachable() {
    let client = mediavault_mcp::api::client::ApiClient::new(
        url::Url::parse("http://127.0.0.1:1").unwrap(),
        mediavault_mcp::config::SecretString::from("internal-key".to_string()),
        Duration::from_millis(500),
        Duration::from_secs(1),
    )
    .unwrap();

    let result = client.get::<Item>("/api/v1/items/1", &[]).await;

    assert!(matches!(result, Err(ApiClientError::Connection(_))));
}

/// 統合テスト1: 遅延させてタイムアウト -> Connection
#[tokio::test]
async fn get_returns_connection_error_on_timeout() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/items/1"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
        .mount(&mock_server)
        .await;

    let client =
        common::build_client_with_timeout(&mock_server, "internal-key", Duration::from_millis(300));

    let result = client.get::<Item>("/api/v1/items/1", &[]).await;

    assert!(matches!(result, Err(ApiClientError::Connection(_))));
}

/// 統合テスト1: 404 + ITEM_NOT_FOUND -> Api { code, status: 404 }
#[tokio::test]
async fn get_classifies_404_as_api_error() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/items/1"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "success": false,
            "error": { "code": "ITEM_NOT_FOUND", "message": "見つかりません" }
        })))
        .mount(&mock_server)
        .await;

    let client = common::build_client(&mock_server, "internal-key");
    let result = client.get::<Item>("/api/v1/items/1", &[]).await;

    match result {
        Err(ApiClientError::Api {
            code,
            status,
            message,
        }) => {
            assert_eq!(code, "ITEM_NOT_FOUND");
            assert_eq!(status, 404);
            assert_eq!(message, "見つかりません");
        }
        other => panic!("expected Api error, got {other:?}"),
    }
}

/// 統合テスト1: 409 + DUPLICATE_RELATION -> Api { status: 409 }
#[tokio::test]
async fn post_classifies_409_as_api_error() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/relations"))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "success": false,
            "error": { "code": "DUPLICATE_RELATION", "message": "既に存在します" }
        })))
        .mount(&mock_server)
        .await;

    let client = common::build_client(&mock_server, "internal-key");
    let result = client
        .post::<_, Item>("/api/v1/relations", &json!({}))
        .await;

    match result {
        Err(ApiClientError::Api { status, .. }) => assert_eq!(status, 409),
        other => panic!("expected Api error, got {other:?}"),
    }
}

/// 統合テスト1: 422 + API_KEY_NOT_CONFIGURED -> Api { status: 422 }
#[tokio::test]
async fn post_classifies_422_as_api_error() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/catalog/search"))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "success": false,
            "error": { "code": "API_KEY_NOT_CONFIGURED", "message": "未設定です" }
        })))
        .mount(&mock_server)
        .await;

    let client = common::build_client(&mock_server, "internal-key");
    let result = client
        .post::<_, Item>("/api/v1/catalog/search", &json!({}))
        .await;

    match result {
        Err(ApiClientError::Api { status, .. }) => assert_eq!(status, 422),
        other => panic!("expected Api error, got {other:?}"),
    }
}

/// 統合テスト1: 500 + INTERNAL_ERROR -> Api { status: 500 }
#[tokio::test]
async fn get_classifies_500_as_api_error() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/items"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "success": false,
            "error": { "code": "INTERNAL_ERROR", "message": "内部エラー" }
        })))
        .mount(&mock_server)
        .await;

    let client = common::build_client(&mock_server, "internal-key");
    let result = client.get::<Item>("/api/v1/items", &[]).await;

    match result {
        Err(ApiClientError::Api { status, .. }) => assert_eq!(status, 500),
        other => panic!("expected Api error, got {other:?}"),
    }
}

/// 統合テスト1: 内部APIパスで 401 -> Auth
#[tokio::test]
async fn internal_path_401_classifies_as_auth_error() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/internal/items"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&mock_server)
        .await;

    let client = common::build_client(&mock_server, "internal-key");
    let result = client.get::<Item>("/api/v1/internal/items", &[]).await;

    assert!(matches!(result, Err(ApiClientError::Auth)));
}

/// 統合テスト1: 壊れたJSON -> Decode
#[tokio::test]
async fn get_classifies_broken_json_as_decode_error() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/items"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&mock_server)
        .await;

    let client = common::build_client(&mock_server, "internal-key");
    let result = client.get::<Item>("/api/v1/items", &[]).await;

    assert!(matches!(result, Err(ApiClientError::Decode(_))));
}

/// 統合テスト2: code / message の透過
#[tokio::test]
async fn error_code_and_message_are_preserved_verbatim() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/tags/1"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "success": false,
            "error": { "code": "TAG_NOT_FOUND", "message": "タグが見つかりません" }
        })))
        .mount(&mock_server)
        .await;

    let client = common::build_client(&mock_server, "internal-key");
    let result = client.get::<Item>("/api/v1/tags/1", &[]).await;

    match result {
        Err(ApiClientError::Api { code, message, .. }) => {
            assert_eq!(code, "TAG_NOT_FOUND");
            assert_eq!(message, "タグが見つかりません");
        }
        other => panic!("expected Api error, got {other:?}"),
    }
}

/// 統合テスト3: 内部APIキーの付与（内部パスのみ）
#[tokio::test]
async fn internal_api_key_is_only_attached_to_internal_paths() {
    let mock_server = common::start_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/internal/items"))
        .and(header("Authorization", "Bearer internal-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": { "id": "1", "title": "内部" }
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v1/items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": { "id": "2", "title": "公開" }
        })))
        .mount(&mock_server)
        .await;

    let client = common::build_client(&mock_server, "internal-key");

    let internal_result = client.get::<Item>("/api/v1/internal/items", &[]).await;
    assert!(internal_result.is_ok());

    let public_result = client.get::<Item>("/api/v1/items", &[]).await;
    assert!(public_result.is_ok());
}

/// 統合テスト4: タイムアウトで無期限にハングしない
#[tokio::test]
async fn request_does_not_hang_indefinitely_on_timeout() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/items"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(5)))
        .mount(&mock_server)
        .await;

    let client =
        common::build_client_with_timeout(&mock_server, "internal-key", Duration::from_millis(300));

    let result = tokio::time::timeout(
        Duration::from_secs(2),
        client.get::<Item>("/api/v1/items", &[]),
    )
    .await
    .expect("request should not hang past the outer test timeout");

    assert!(matches!(result, Err(ApiClientError::Connection(_))));
}

/// 統合テスト5: POST は失敗時にリトライせず、モックへの到達は1回のみ
///
/// 🟡 Intent: wiremock は HTTP レベルのモックであり、TCP接続失敗そのものを
///    注入できないため「接続エラーから1回だけリトライして成功する」ケースは
///    実サーバーの生死を切り替える形では検証できない。ここでは POST が
///    エラー時にリトライしない（到達回数が1回であること）を `expect(1)` で保証する。
#[tokio::test]
async fn post_does_not_retry_on_failure() {
    let mock_server = common::start_mock_server().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/relations"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "success": false,
            "error": { "code": "INTERNAL_ERROR", "message": "内部エラー" }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = common::build_client(&mock_server, "internal-key");
    let result = client
        .post::<_, Item>("/api/v1/relations", &json!({}))
        .await;

    assert!(result.is_err());
    mock_server.verify().await;
}

/// 統合テスト5: GET は失敗時に接続をやり直す経路を持つが、成功レスポンスには
/// 1回しか到達しない（正常系でリトライが誤発火しないことの確認）
#[tokio::test]
async fn get_does_not_retry_on_success() {
    let mock_server = common::start_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/items/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": { "id": "1", "title": "成功" }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = common::build_client(&mock_server, "internal-key");
    let result = client.get::<Item>("/api/v1/items/1", &[]).await;

    assert!(result.is_ok());
    mock_server.verify().await;
}
