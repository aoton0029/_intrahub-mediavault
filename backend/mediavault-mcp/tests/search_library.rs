//! `search_library` の統合テスト（wiremock）
//!
//! TASK-0013: search_library ツールの実装

use mediavault_mcp::services::cursor::Cursor;
use mediavault_mcp::services::search_library::search;
use mediavault_mcp::tools::search_library::SearchLibraryParams;
use serde_json::json;
use uuid::Uuid;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

mod common;

fn item_json(
    id: Uuid,
    title: &str,
    media_type: &str,
    status: &str,
    year: i32,
) -> serde_json::Value {
    json!({
        "id": id.to_string(),
        "media_type": media_type,
        "title": title,
        "original_title": null,
        "release_date": format!("{year}-04-01"),
        "status": status,
        "rating": null,
        "is_favorite": false,
        "tags": [],
        "categories": []
    })
}

fn items_response(items: Vec<serde_json::Value>, has_more: bool, total: u64) -> serde_json::Value {
    json!({
        "success": true,
        "data": items,
        "pagination": {
            "limit": 20,
            "has_more": has_more,
            "next_after_created_at": if has_more { Some("2026-07-01T12:00:00") } else { None },
            "next_after_id": if has_more { Some(Uuid::new_v4().to_string()) } else { None },
            "total": total
        }
    })
}

async fn empty_tags_and_categories(mock_server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/api/v1/tags"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"success": true, "data": []})),
        )
        .mount(mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/categories"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"success": true, "data": []})),
        )
        .mount(mock_server)
        .await;
}

/// 統合テスト1: タイトル検索で1件
#[tokio::test]
async fn title_search_returns_single_match_with_item_id() {
    let mock_server = common::start_mock_server().await;
    let id = Uuid::new_v4();
    Mock::given(method("GET"))
        .and(path("/api/v1/items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(items_response(
            vec![item_json(id, "作品A", "anime", "in_progress", 2023)],
            false,
            1,
        )))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = search(
        &api,
        SearchLibraryParams {
            title: Some("作品A".to_string()),
            ..Default::default()
        },
    )
    .await;

    assert_eq!(result.total_count, 1);
    assert_eq!(result.items.len(), 1);
    assert_eq!(result.items[0].item_id, id);
    assert_eq!(result.source, "mediavault_library");
}

/// 統合テスト2: 同名作品3件の区別
#[tokio::test]
async fn distinguishes_three_items_with_same_title() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(items_response(
            vec![
                item_json(Uuid::new_v4(), "同名作", "anime", "not_started", 2020),
                item_json(Uuid::new_v4(), "同名作", "movie", "in_progress", 2021),
                item_json(Uuid::new_v4(), "同名作", "manga", "completed", 2022),
            ],
            false,
            3,
        )))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = search(
        &api,
        SearchLibraryParams {
            title: Some("同名作".to_string()),
            ..Default::default()
        },
    )
    .await;

    assert_eq!(result.items.len(), 3);
    let media_types: Vec<_> = result
        .items
        .iter()
        .map(|item| serde_json::to_value(item.media_type).unwrap())
        .collect();
    assert_eq!(
        media_types,
        vec![json!("anime"), json!("movie"), json!("manga")]
    );
    let years: Vec<_> = result.items.iter().map(|item| item.release_year).collect();
    assert_eq!(years, vec![Some(2020), Some(2021), Some(2022)]);
}

/// 統合テスト3: 原題・別名でのヒット（MCPはクエリを正しく渡し結果を扱うことのみ確認）
#[tokio::test]
async fn passes_title_query_and_returns_hit_from_original_title_or_alias() {
    let mock_server = common::start_mock_server().await;
    let id = Uuid::new_v4();
    Mock::given(method("GET"))
        .and(path("/api/v1/items"))
        .and(query_param("title", "Original Title"))
        .respond_with(ResponseTemplate::new(200).set_body_json(items_response(
            vec![item_json(id, "作品A", "anime", "not_started", 2023)],
            false,
            1,
        )))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = search(
        &api,
        SearchLibraryParams {
            title: Some("Original Title".to_string()),
            ..Default::default()
        },
    )
    .await;

    assert_eq!(result.items.len(), 1);
    assert_eq!(result.items[0].item_id, id);
}

/// 統合テスト4: タグ名の解決
#[tokio::test]
async fn resolves_tag_name_to_id_before_searching_items() {
    let mock_server = common::start_mock_server().await;
    let tag_id = Uuid::new_v4();
    Mock::given(method("GET"))
        .and(path("/api/v1/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [{"id": tag_id.to_string(), "name": "積読", "item_count": 3}]
        })))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/items"))
        .and(query_param("tag_id", tag_id.to_string()))
        .respond_with(ResponseTemplate::new(200).set_body_json(items_response(vec![], false, 0)))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = search(
        &api,
        SearchLibraryParams {
            tag: Some("積読".to_string()),
            ..Default::default()
        },
    )
    .await;

    assert_eq!(
        result.outcome,
        mediavault_mcp::result::outcome::Outcome::Success
    );
    assert_eq!(result.applied_filters["tag_id"], json!(tag_id));
}

/// 統合テスト5: タグ名が解決できない
#[tokio::test]
async fn tag_name_not_resolved_returns_not_found_and_does_not_call_items() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [{"id": Uuid::new_v4().to_string(), "name": "SF", "item_count": 1}]
        })))
        .mount(&mock_server)
        .await;
    // /api/v1/items へのリクエストが来たら失敗させる（呼ばれないことを検証する）
    Mock::given(method("GET"))
        .and(path("/api/v1/items"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = search(
        &api,
        SearchLibraryParams {
            tag: Some("存在しないタグ".to_string()),
            ..Default::default()
        },
    )
    .await;

    assert_eq!(
        result.outcome,
        mediavault_mcp::result::outcome::Outcome::NotFound
    );
    let error = result.error.unwrap();
    assert_eq!(error.code, "MCP_NAME_NOT_RESOLVED");
    assert_eq!(result.applied_filters["available_names"], json!(["SF"]));
}

/// 統合テスト6: 0件時の source 明示
#[tokio::test]
async fn zero_results_still_carries_source_field() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(items_response(vec![], false, 0)))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = search(
        &api,
        SearchLibraryParams {
            title: Some("存在しない作品".to_string()),
            ..Default::default()
        },
    )
    .await;

    assert_eq!(result.total_count, 0);
    assert!(result.items.is_empty());
    assert_eq!(result.source, "mediavault_library");
}

/// 統合テスト7: ページネーション
#[tokio::test]
async fn next_cursor_carries_after_created_at_and_after_id_to_next_page() {
    let mock_server = common::start_mock_server().await;
    empty_tags_and_categories(&mock_server).await;

    let cursor_created_at =
        chrono::NaiveDateTime::parse_from_str("2026-07-01T12:00:00", "%Y-%m-%dT%H:%M:%S").unwrap();
    let cursor_id = Uuid::new_v4();

    Mock::given(method("GET"))
        .and(path("/api/v1/items"))
        .and(wiremock::matchers::query_param_is_missing("after_id"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [item_json(Uuid::new_v4(), "作品A", "anime", "not_started", 2023)],
            "pagination": {
                "limit": 20,
                "has_more": true,
                "next_after_created_at": "2026-07-01T12:00:00",
                "next_after_id": cursor_id.to_string(),
                "total": 2
            }
        })))
        .mount(&mock_server)
        .await;

    let api = common::build_client(&mock_server, "internal-key");
    let first_page = search(&api, SearchLibraryParams::default()).await;
    let next_cursor = first_page
        .next_cursor
        .expect("1ページ目には next_cursor があるはず");

    Mock::given(method("GET"))
        .and(path("/api/v1/items"))
        .and(query_param("after_id", cursor_id.to_string()))
        .and(query_param("after_created_at", "2026-07-01T12:00:00"))
        .respond_with(ResponseTemplate::new(200).set_body_json(items_response(vec![], false, 2)))
        .mount(&mock_server)
        .await;

    let second_page = search(
        &api,
        SearchLibraryParams {
            cursor: Some(next_cursor),
            ..Default::default()
        },
    )
    .await;

    assert_eq!(
        second_page.outcome,
        mediavault_mcp::result::outcome::Outcome::Success
    );
    let _ = cursor_created_at; // ドキュメント用途、比較は wiremock 側のクエリ一致で行う
}

/// 統合テスト8: 接続エラー
#[tokio::test]
async fn connection_failure_returns_api_unreachable_and_retriable() {
    let mock_server = common::start_mock_server().await;
    drop(mock_server); // サーバーを即座に停止し、接続不能な状態を再現する
    let api = mediavault_mcp::api::client::ApiClient::new(
        url::Url::parse("http://127.0.0.1:1").unwrap(),
        mediavault_mcp::config::SecretString::from("internal-key".to_string()),
        std::time::Duration::from_millis(200),
        std::time::Duration::from_millis(200),
    )
    .unwrap();

    let result = search(&api, SearchLibraryParams::default()).await;

    assert_eq!(
        result.outcome,
        mediavault_mcp::result::outcome::Outcome::Error
    );
    let error = result.error.unwrap();
    assert_eq!(error.code, "MCP_API_UNREACHABLE");
    assert!(error.retriable);
}

/// 統合テスト9: description が返らない
#[tokio::test]
async fn response_json_does_not_contain_description_field() {
    let mock_server = common::start_mock_server().await;
    let mut item = item_json(Uuid::new_v4(), "作品A", "anime", "not_started", 2023);
    item["description"] = json!("とても長いあらすじ".repeat(50));
    Mock::given(method("GET"))
        .and(path("/api/v1/items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(items_response(
            vec![item],
            false,
            1,
        )))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = search(&api, SearchLibraryParams::default()).await;
    let value = serde_json::to_value(&result).unwrap();

    assert!(value["items"][0].get("description").is_none());
    assert!(value["items"][0].get("details").is_none());
}

/// テストケース1: limit のバリデーション（統合経路）
#[tokio::test]
async fn limit_51_returns_invalid_argument_without_calling_api() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/items"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = search(
        &api,
        SearchLibraryParams {
            limit: Some(51),
            ..Default::default()
        },
    )
    .await;

    assert_eq!(
        result.outcome,
        mediavault_mcp::result::outcome::Outcome::Error
    );
    assert_eq!(result.error.unwrap().code, "MCP_INVALID_ARGUMENT");
}

/// テストケース2: 空白のみの検索語（統合経路）
#[tokio::test]
async fn whitespace_only_title_returns_invalid_argument_without_calling_api() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/items"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = search(
        &api,
        SearchLibraryParams {
            title: Some("   ".to_string()),
            ..Default::default()
        },
    )
    .await;

    assert_eq!(
        result.outcome,
        mediavault_mcp::result::outcome::Outcome::Error
    );
    assert_eq!(result.error.unwrap().code, "MCP_INVALID_ARGUMENT");
}

/// テストケース4: 不正なカーソル（統合経路）
#[tokio::test]
async fn invalid_cursor_returns_invalid_argument_without_calling_api() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/items"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");

    let result = search(
        &api,
        SearchLibraryParams {
            cursor: Some("not-base64".to_string()),
            ..Default::default()
        },
    )
    .await;

    assert_eq!(
        result.outcome,
        mediavault_mcp::result::outcome::Outcome::Error
    );
    assert_eq!(result.error.unwrap().code, "MCP_INVALID_ARGUMENT");
}

#[allow(dead_code)]
fn unused_cursor_type_reference(_c: Cursor) {}
