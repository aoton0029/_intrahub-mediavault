//! `MasterResolver` の統合テスト（wiremock）
//!
//! TASK-0012: ItemSummary 縮約と名前→ID解決サービス

use mediavault_mcp::services::resolve::{MasterResolver, Resolution};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

mod common;

fn tags_response(names: &[&str]) -> serde_json::Value {
    let data: Vec<_> = names
        .iter()
        .map(|name| {
            json!({
                "id": uuid::Uuid::new_v4().to_string(),
                "name": name,
                "item_count": 0
            })
        })
        .collect();
    json!({"success": true, "data": data})
}

/// 統合テスト1: 呼び出し内メモ化
#[tokio::test]
async fn resolves_multiple_tags_with_single_get_tags_call() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/tags"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(tags_response(&["積読", "SF", "映画"])),
        )
        .expect(1)
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");
    let mut resolver = MasterResolver::new(&api);

    for name in ["積読", "SF", "映画"] {
        let resolution = resolver.resolve_tag(name).await.unwrap();
        assert!(matches!(resolution, Resolution::Found(_)));
    }

    // `expect(1)` により wiremock がドロップ時に呼び出し回数を検証する
}

/// 統合テスト2: 呼び出しをまたぐキャッシュがない
#[tokio::test]
async fn does_not_cache_resolution_across_resolver_instances() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(tags_response(&["積読"])))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    let api = common::build_client(&mock_server, "internal-key");
    let mut resolver = MasterResolver::new(&api);
    let first = resolver.resolve_tag("新タグ").await.unwrap();
    assert!(matches!(first, Resolution::NotFound { .. }));

    // 1回目と2回目の間でモックの一覧を変更する（別経路で作られたタグを模す）
    Mock::given(method("GET"))
        .and(path("/api/v1/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(tags_response(&["新タグ"])))
        .mount(&mock_server)
        .await;

    // 新しい MasterResolver（= 新しい呼び出し）を生成する
    let mut resolver = MasterResolver::new(&api);
    let second = resolver.resolve_tag("新タグ").await.unwrap();
    assert!(matches!(second, Resolution::Found(_)));

    let requests = mock_server.received_requests().await.unwrap();
    let tag_requests = requests
        .iter()
        .filter(|req| req.url.path() == "/api/v1/tags")
        .count();
    assert_eq!(tag_requests, 2);
}

/// 統合テスト3: マスタ取得の失敗
#[tokio::test]
async fn master_fetch_failure_propagates_as_api_client_error_not_resolution() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/tags"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "success": false,
            "error": {"code": "INTERNAL_ERROR", "message": "サーバーエラー"}
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");
    let mut resolver = MasterResolver::new(&api);

    let result = resolver.resolve_tag("積読").await;

    assert!(result.is_err());
}

/// カテゴリ・マイリストも同様に解決できる
#[tokio::test]
async fn resolves_category_and_mylist() {
    let mock_server = common::start_mock_server().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/categories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(tags_response(&["2026年視聴"])))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/mylists"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [{"id": uuid::Uuid::new_v4().to_string(), "name": "お気に入り"}]
        })))
        .mount(&mock_server)
        .await;
    let api = common::build_client(&mock_server, "internal-key");
    let mut resolver = MasterResolver::new(&api);

    let category = resolver.resolve_category("2026年視聴").await.unwrap();
    let mylist = resolver.resolve_mylist("お気に入り").await.unwrap();

    assert!(matches!(category, Resolution::Found(_)));
    assert!(matches!(mylist, Resolution::Found(_)));
}
