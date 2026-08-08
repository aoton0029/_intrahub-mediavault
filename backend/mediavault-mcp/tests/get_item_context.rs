//! `get_item_context` の統合テスト（wiremock）
//!
//! TASK-0014: get_item_context ツールの実装

use mediavault_mcp::result::outcome::Outcome;
use mediavault_mcp::services::context::get_context;
use serde_json::json;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

mod common;

fn item_detail_json(id: Uuid) -> serde_json::Value {
    json!({
        "id": id.to_string(),
        "media_type": "anime",
        "title": "作品A",
        "original_title": null,
        "description": "あらすじ",
        "cover_image_url": null,
        "release_date": "2023-04-01",
        "homepage_url": null,
        "status": "in_progress",
        "consumed_date": null,
        "rating": 8.5,
        "is_favorite": true,
        "source": "api",
        "external_id": "12345",
        "created_at": "2026-07-01T12:00:00",
        "updated_at": "2026-07-01T12:00:00",
        "detail": {"episodes": 26},
        "tags": [{"id": Uuid::new_v4().to_string(), "name": "お気に入り原作"}],
        "categories": [],
        "calibre_links": [],
        "streaming_links": [],
        "images": []
    })
}

fn ok(data: serde_json::Value) -> serde_json::Value {
    json!({"success": true, "data": data})
}

fn err(code: &str, message: &str) -> serde_json::Value {
    json!({"success": false, "error": {"code": code, "message": message}})
}

async fn mount_item_detail(mock_server: &MockServer, item_id: Uuid) {
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{item_id}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok(item_detail_json(item_id))))
        .mount(mock_server)
        .await;
}

async fn mount_empty_section(mock_server: &MockServer, item_id: Uuid, section: &str) {
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{item_id}/{section}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok(json!([]))))
        .mount(mock_server)
        .await;
}

const SECTIONS: [&str; 8] = [
    "relations",
    "mylists",
    "groups",
    "cast",
    "staff",
    "files",
    "links",
    "trailers",
];

/// 統合テスト1: すべて揃った Item
#[tokio::test]
async fn all_sections_loaded_returns_success() {
    let mock_server = common::start_mock_server().await;
    let item_id = Uuid::new_v4();
    mount_item_detail(&mock_server, item_id).await;

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{item_id}/relations")))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok(json!([{
            "id": Uuid::new_v4().to_string(),
            "item_id": item_id.to_string(),
            "related_item_id": Uuid::new_v4().to_string(),
            "relation_type": "sequel"
        }]))))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{item_id}/mylists")))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok(json!([{
            "id": Uuid::new_v4().to_string(),
            "name": "積読"
        }]))))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{item_id}/groups")))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok(json!([{
            "id": Uuid::new_v4().to_string(),
            "group_type": "season",
            "group_name": "season 1",
            "number": 1
        }]))))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{item_id}/cast")))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok(json!([{
            "id": Uuid::new_v4().to_string(),
            "item_id": item_id.to_string(),
            "cast_id": Uuid::new_v4().to_string(),
            "character_name": "主人公"
        }]))))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{item_id}/staff")))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok(json!([{
            "id": Uuid::new_v4().to_string(),
            "item_id": item_id.to_string(),
            "staff_id": Uuid::new_v4().to_string(),
            "role": "監督"
        }]))))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{item_id}/files")))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok(json!([{
            "id": Uuid::new_v4().to_string(),
            "item_id": item_id.to_string(),
            "path": "/srv/anime/ep1.mkv",
            "file_type": "video"
        }]))))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{item_id}/links")))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok(json!([{
            "id": Uuid::new_v4().to_string(),
            "item_id": item_id.to_string(),
            "url": "https://example.com",
            "label": "公式"
        }]))))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{item_id}/trailers")))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok(json!([{
            "id": Uuid::new_v4().to_string(),
            "item_id": item_id.to_string(),
            "url": "https://youtube.com/watch?v=xxx",
            "label": null
        }]))))
        .mount(&mock_server)
        .await;

    let api = common::build_client(&mock_server, "internal-key");
    let result = get_context(&api, item_id).await;

    assert_eq!(result.outcome, Outcome::Success);
    let value = serde_json::to_value(&result).unwrap();
    for section in SECTIONS {
        assert_eq!(
            value[section]["state"],
            json!("loaded"),
            "section={section}"
        );
    }
    assert_eq!(value["item"]["item_id"], json!(item_id));
}

/// 統合テスト2: 並列実行の確認（逐次ではなく並行して到達する）
#[tokio::test]
async fn parallel_requests_complete_within_one_round_trip() {
    let mock_server = common::start_mock_server().await;
    let item_id = Uuid::new_v4();
    mount_item_detail(&mock_server, item_id).await;

    for section in SECTIONS {
        Mock::given(method("GET"))
            .and(path(format!("/api/v1/items/{item_id}/{section}")))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_delay(std::time::Duration::from_millis(100))
                    .set_body_json(ok(json!([]))),
            )
            .mount(&mock_server)
            .await;
    }

    let api = common::build_client(&mock_server, "internal-key");
    let start = std::time::Instant::now();
    let result = get_context(&api, item_id).await;
    let elapsed = start.elapsed();

    assert_eq!(result.outcome, Outcome::Success);
    assert!(
        elapsed < std::time::Duration::from_millis(500),
        "並列実行されていれば500ms未満に収まるはず: {elapsed:?}"
    );
}

/// 統合テスト3: 一部未登録
#[tokio::test]
async fn some_sections_empty_still_reports_success() {
    let mock_server = common::start_mock_server().await;
    let item_id = Uuid::new_v4();
    mount_item_detail(&mock_server, item_id).await;

    for section in SECTIONS {
        mount_empty_section(&mock_server, item_id, section).await;
    }

    let api = common::build_client(&mock_server, "internal-key");
    let result = get_context(&api, item_id).await;

    assert_eq!(result.outcome, Outcome::Success);
    let value = serde_json::to_value(&result).unwrap();
    assert_eq!(value["mylists"], json!({"state": "empty"}));
    assert_eq!(value["relations"], json!({"state": "empty"}));
}

/// 統合テスト4: 一部取得失敗
#[tokio::test]
async fn one_section_failure_returns_partial_and_keeps_others() {
    let mock_server = common::start_mock_server().await;
    let item_id = Uuid::new_v4();
    mount_item_detail(&mock_server, item_id).await;

    for section in SECTIONS {
        if section == "files" {
            Mock::given(method("GET"))
                .and(path(format!("/api/v1/items/{item_id}/files")))
                .respond_with(
                    ResponseTemplate::new(500)
                        .set_body_json(err("INTERNAL_ERROR", "サーバーエラー")),
                )
                .mount(&mock_server)
                .await;
        } else {
            mount_empty_section(&mock_server, item_id, section).await;
        }
    }

    let api = common::build_client(&mock_server, "internal-key");
    let result = get_context(&api, item_id).await;

    assert_eq!(result.outcome, Outcome::Partial);
    let value = serde_json::to_value(&result).unwrap();
    assert_eq!(value["files"]["state"], json!("failed"));
    assert_eq!(value["files"]["error"]["code"], json!("INTERNAL_ERROR"));
    assert_eq!(value["links"], json!({"state": "empty"}));
}

/// 統合テスト5: Item が存在しない
#[tokio::test]
async fn item_not_found_returns_not_found_and_skips_other_calls() {
    let mock_server = common::start_mock_server().await;
    let item_id = Uuid::new_v4();

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{item_id}")))
        .respond_with(
            ResponseTemplate::new(404).set_body_json(err("ITEM_NOT_FOUND", "見つかりません")),
        )
        .mount(&mock_server)
        .await;
    for section in SECTIONS {
        Mock::given(method("GET"))
            .and(path(format!("/api/v1/items/{item_id}/{section}")))
            .respond_with(ResponseTemplate::new(500))
            .expect(0)
            .mount(&mock_server)
            .await;
    }

    let api = common::build_client(&mock_server, "internal-key");
    let result = get_context(&api, item_id).await;

    assert_eq!(result.outcome, Outcome::NotFound);
    assert!(result.item.is_none());
    let error = result.error.unwrap();
    assert_eq!(error.code, "ITEM_NOT_FOUND");
}

/// 統合テスト6: タグ等を追加取得しない
#[tokio::test]
async fn does_not_fetch_tags_or_streaming_links_separately() {
    let mock_server = common::start_mock_server().await;
    let item_id = Uuid::new_v4();
    mount_item_detail(&mock_server, item_id).await;
    for section in SECTIONS {
        mount_empty_section(&mock_server, item_id, section).await;
    }
    Mock::given(method("GET"))
        .and(path("/api/v1/tags"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{item_id}/streaming-links")))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&mock_server)
        .await;

    let api = common::build_client(&mock_server, "internal-key");
    let result = get_context(&api, item_id).await;

    assert_eq!(result.outcome, Outcome::Success);
}

/// 統合テスト: 関係の向き（related_item_id 側 = incoming）
#[tokio::test]
async fn relation_direction_is_incoming_when_item_is_the_target() {
    let mock_server = common::start_mock_server().await;
    let item_id = Uuid::new_v4();
    let origin_id = Uuid::new_v4();
    mount_item_detail(&mock_server, item_id).await;

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{item_id}/relations")))
        .respond_with(ResponseTemplate::new(200).set_body_json(ok(json!([{
            "id": Uuid::new_v4().to_string(),
            "item_id": origin_id.to_string(),
            "related_item_id": item_id.to_string(),
            "relation_type": "adaptation"
        }]))))
        .mount(&mock_server)
        .await;
    for section in [
        "mylists", "groups", "cast", "staff", "files", "links", "trailers",
    ] {
        mount_empty_section(&mock_server, item_id, section).await;
    }

    let api = common::build_client(&mock_server, "internal-key");
    let result = get_context(&api, item_id).await;

    let value = serde_json::to_value(&result).unwrap();
    assert_eq!(
        value["relations"]["items"][0]["direction"],
        json!("incoming")
    );
    assert_eq!(
        value["relations"]["items"][0]["related_item_id"],
        json!(origin_id)
    );
}
