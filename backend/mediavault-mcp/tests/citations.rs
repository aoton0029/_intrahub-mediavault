//! `list_citations` / `add_citation` の統合テスト（wiremock）
//!
//! 第2段階。設計決定 D-11・REQ-903 / REQ-904 / REQ-905 より。

use mediavault_mcp::api::models::LocatorType;
use mediavault_mcp::result::outcome::Outcome;
use mediavault_mcp::services::citations::{add_citation, list_citations};
use mediavault_mcp::tools::citations::{AddCitationParams, ListCitationsParams};
use serde_json::json;
use uuid::Uuid;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

mod common;

fn ok(data: serde_json::Value) -> serde_json::Value {
    json!({"success": true, "data": data})
}

fn err(code: &str, message: &str) -> serde_json::Value {
    json!({"success": false, "error": {"code": code, "message": message}})
}

fn citation_json(quote: &str) -> serde_json::Value {
    json!({
        "id": Uuid::new_v4().to_string(),
        "item_id": Uuid::new_v4().to_string(),
        "quote_text": quote,
        "note": "第3章の議論のまとめとして引用",
        "locator_type": "page",
        "page_number": 128,
        "timestamp_seconds": null,
        "location_number": null,
        "chapter": null,
        "created_at": "2026-07-01T12:00:00",
        "updated_at": "2026-07-01T12:00:00"
    })
}

async fn mount_citations(mock_server: &MockServer, item_id: Uuid, count: usize) {
    let body: Vec<serde_json::Value> = (0..count)
        .map(|i| citation_json(&format!("引用{i}")))
        .collect();
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{item_id}/citations")))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok(json!(body))))
        .mount(mock_server)
        .await;
}

fn list_params(item_id: Uuid) -> ListCitationsParams {
    ListCitationsParams {
        item_id,
        limit: None,
        cursor: None,
    }
}

// ============================================================
// list_citations
// ============================================================

/// 位置情報は api の値をそのまま透過する。「p.128」のような表示文字列へ整形しない（REQ-146）。
#[tokio::test]
async fn returns_locator_fields_untouched() {
    let mock_server = common::start_mock_server().await;
    let item_id = Uuid::new_v4();
    mount_citations(&mock_server, item_id, 1).await;

    let api = common::build_client(&mock_server, "internal-key");
    let result = list_citations(&api, list_params(item_id)).await;

    assert_eq!(result.outcome, Outcome::Success);
    let value = serde_json::to_value(&result).unwrap();
    let citation = &value["citations"][0];
    assert_eq!(citation["locator_type"], json!("page"));
    assert_eq!(citation["page_number"], json!(128));
    assert_eq!(citation["timestamp_seconds"], json!(null));

    let serialized = serde_json::to_string(&result).unwrap();
    assert!(
        !serialized.contains("p.128"),
        "表示用に整形してはならない（利用側が位置種別ごとに処理できなくなる）"
    );
}

/// api にページネーションが無いため、MCP 側で切り出す。`total_count` は全件数を返す。
#[tokio::test]
async fn slices_locally_and_reports_full_total_count() {
    let mock_server = common::start_mock_server().await;
    let item_id = Uuid::new_v4();
    mount_citations(&mock_server, item_id, 30).await;

    let api = common::build_client(&mock_server, "internal-key");
    let result = list_citations(
        &api,
        ListCitationsParams {
            item_id,
            limit: Some(10),
            cursor: None,
        },
    )
    .await;

    assert_eq!(result.citations.len(), 10);
    assert_eq!(result.total_count, 30, "総件数は切り出し前の全件数");
    assert!(result.next_cursor.is_some());
}

/// カーソルで続きを取得できる。ページをまたいで重複・欠落しない。
#[tokio::test]
async fn cursor_advances_without_gaps_or_duplicates() {
    let mock_server = common::start_mock_server().await;
    let item_id = Uuid::new_v4();
    mount_citations(&mock_server, item_id, 25).await;
    let api = common::build_client(&mock_server, "internal-key");

    let mut seen: Vec<String> = Vec::new();
    let mut cursor = None;
    loop {
        let result = list_citations(
            &api,
            ListCitationsParams {
                item_id,
                limit: Some(10),
                cursor: cursor.clone(),
            },
        )
        .await;
        seen.extend(result.citations.iter().map(|c| c.quote_text.clone()));
        match result.next_cursor {
            Some(next) => cursor = Some(next),
            None => break,
        }
    }

    assert_eq!(seen.len(), 25, "全件を過不足なく取得できる");
    let unique: std::collections::HashSet<_> = seen.iter().collect();
    assert_eq!(unique.len(), 25, "ページ間で重複しない");
}

/// 最終ページでは `next_cursor` を返さない。
#[tokio::test]
async fn last_page_has_no_next_cursor() {
    let mock_server = common::start_mock_server().await;
    let item_id = Uuid::new_v4();
    mount_citations(&mock_server, item_id, 5).await;

    let api = common::build_client(&mock_server, "internal-key");
    let result = list_citations(
        &api,
        ListCitationsParams {
            item_id,
            limit: Some(10),
            cursor: None,
        },
    )
    .await;

    assert_eq!(result.citations.len(), 5);
    assert!(result.next_cursor.is_none());
}

/// 引用が0件でもエラーにせず success を返す。
#[tokio::test]
async fn zero_citations_is_success_not_error() {
    let mock_server = common::start_mock_server().await;
    let item_id = Uuid::new_v4();
    mount_citations(&mock_server, item_id, 0).await;

    let api = common::build_client(&mock_server, "internal-key");
    let result = list_citations(&api, list_params(item_id)).await;

    assert_eq!(result.outcome, Outcome::Success);
    assert_eq!(result.total_count, 0);
    assert!(result.citations.is_empty());
    assert!(result.next_cursor.is_none());
}

/// 存在しない Item は `not_found` と api のエラーコードをそのまま返す（REQ-146）。
#[tokio::test]
async fn missing_item_returns_not_found_with_api_code() {
    let mock_server = common::start_mock_server().await;
    let item_id = Uuid::new_v4();
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{item_id}/citations")))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(err("ITEM_NOT_FOUND", "アイテムが存在しません")),
        )
        .mount(&mock_server)
        .await;

    let api = common::build_client(&mock_server, "internal-key");
    let result = list_citations(&api, list_params(item_id)).await;

    assert_eq!(result.outcome, Outcome::NotFound);
    assert_eq!(result.error.unwrap().code, "ITEM_NOT_FOUND");
}

/// 不正なカーソルは api を呼ばずに `MCP_INVALID_ARGUMENT` で弾く。
#[tokio::test]
async fn invalid_cursor_does_not_call_api() {
    let mock_server = common::start_mock_server().await;
    let item_id = Uuid::new_v4();

    let api = common::build_client(&mock_server, "internal-key");
    let result = list_citations(
        &api,
        ListCitationsParams {
            item_id,
            limit: None,
            cursor: Some("not-a-valid-cursor".to_string()),
        },
    )
    .await;

    assert_eq!(result.outcome, Outcome::Error);
    assert_eq!(result.error.unwrap().code, "MCP_INVALID_ARGUMENT");
    assert!(
        mock_server.received_requests().await.unwrap().is_empty(),
        "バリデーション失敗時は api を呼ばない"
    );
}

// ============================================================
// add_citation
// ============================================================

fn add_params(item_id: Uuid) -> AddCitationParams {
    AddCitationParams {
        item_id,
        quote_text: "人は見たいものしか見ようとしない。".to_string(),
        locator_type: LocatorType::Page,
        note: Some("第3章より".to_string()),
        page_number: Some(128),
        timestamp_seconds: None,
        location_number: None,
        chapter: None,
    }
}

/// 位置情報を含めて POST し、作成された引用を返す。
#[tokio::test]
async fn posts_citation_with_locator_and_returns_created() {
    let mock_server = common::start_mock_server().await;
    let item_id = Uuid::new_v4();

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/items/{item_id}/citations")))
        .and(body_json(json!({
            "quote_text": "人は見たいものしか見ようとしない。",
            "locator_type": "page",
            "note": "第3章より",
            "page_number": 128
        })))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(ok(citation_json("人は見たいものしか見ようとしない。"))),
        )
        .mount(&mock_server)
        .await;

    let api = common::build_client(&mock_server, "internal-key");
    let result = add_citation(&api, add_params(item_id)).await;

    assert_eq!(result.outcome, Outcome::Success);
    let citation = result.citation.expect("作成された引用が返る");
    assert_eq!(citation.page_number, Some(128));
}

/// 未指定の位置フィールドはボディに含めない（api 側で null 上書きが起きないようにする）。
#[tokio::test]
async fn omits_unset_locator_fields_from_body() {
    let mock_server = common::start_mock_server().await;
    let item_id = Uuid::new_v4();

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/items/{item_id}/citations")))
        .and(body_json(json!({
            "quote_text": "位置不明の引用",
            "locator_type": "none"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(ok(citation_json("位置不明の引用"))))
        .mount(&mock_server)
        .await;

    let api = common::build_client(&mock_server, "internal-key");
    let result = add_citation(
        &api,
        AddCitationParams {
            item_id,
            quote_text: "位置不明の引用".to_string(),
            locator_type: LocatorType::None,
            note: None,
            page_number: None,
            timestamp_seconds: None,
            location_number: None,
            chapter: None,
        },
    )
    .await;

    assert_eq!(result.outcome, Outcome::Success);
}

/// **REQ-904 の中核**: `locator_type` と位置フィールドが不整合なら api を呼ばずに弾く。
/// api 側は必須バリデーションしないため、ここで止めないと出典不明の引用が保存される。
#[tokio::test]
async fn locator_mismatch_is_rejected_before_calling_api() {
    let mock_server = common::start_mock_server().await;
    let item_id = Uuid::new_v4();

    let api = common::build_client(&mock_server, "internal-key");
    let result = add_citation(
        &api,
        AddCitationParams {
            page_number: None, // locator_type: Page なのに未指定
            ..add_params(item_id)
        },
    )
    .await;

    assert_eq!(result.outcome, Outcome::Error);
    assert_eq!(result.error.unwrap().code, "MCP_INVALID_ARGUMENT");
    assert!(
        mock_server.received_requests().await.unwrap().is_empty(),
        "不整合な引用を api へ送ってはならない"
    );
}

/// 存在しない Item への追加は `not_found`。
#[tokio::test]
async fn missing_item_on_add_returns_not_found() {
    let mock_server = common::start_mock_server().await;
    let item_id = Uuid::new_v4();
    Mock::given(method("POST"))
        .and(path(format!("/api/v1/items/{item_id}/citations")))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(err("ITEM_NOT_FOUND", "アイテムが存在しません")),
        )
        .mount(&mock_server)
        .await;

    let api = common::build_client(&mock_server, "internal-key");
    let result = add_citation(&api, add_params(item_id)).await;

    assert_eq!(result.outcome, Outcome::NotFound);
    assert_eq!(result.error.unwrap().code, "ITEM_NOT_FOUND");
}

/// api のバリデーションエラーはコード・メッセージを保ったまま透過する（REQ-146）。
#[tokio::test]
async fn api_validation_error_is_passed_through() {
    let mock_server = common::start_mock_server().await;
    let item_id = Uuid::new_v4();
    Mock::given(method("POST"))
        .and(path(format!("/api/v1/items/{item_id}/citations")))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_json(err("VALIDATION_ERROR", "quote_text は必須です")),
        )
        .mount(&mock_server)
        .await;

    let api = common::build_client(&mock_server, "internal-key");
    let result = add_citation(&api, add_params(item_id)).await;

    assert_eq!(result.outcome, Outcome::Error);
    let error = result.error.unwrap();
    assert_eq!(error.code, "VALIDATION_ERROR");
    assert_eq!(error.message, "quote_text は必須です");
}

/// **冪等ではない**（D-03 の明示的な例外）。失敗しても再送しない。
/// 再送すると api に重複検出が無いため引用が二重登録されうる。
#[tokio::test]
async fn does_not_retry_the_post_on_failure() {
    let mock_server = common::start_mock_server().await;
    let item_id = Uuid::new_v4();
    Mock::given(method("POST"))
        .and(path(format!("/api/v1/items/{item_id}/citations")))
        .respond_with(
            ResponseTemplate::new(500).set_body_json(err("INTERNAL_ERROR", "サーバエラー")),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let api = common::build_client(&mock_server, "internal-key");
    let result = add_citation(&api, add_params(item_id)).await;

    assert_eq!(result.outcome, Outcome::Error);
    // `expect(1)` が MockServer の drop 時に検証される
}
