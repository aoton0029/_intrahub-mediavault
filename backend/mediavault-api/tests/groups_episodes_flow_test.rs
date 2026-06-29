//! TASK-0032: 主要フロー統合テスト — EDGE-101再確認（IT-007）
//!
//! 🔵 信頼性レベル: main-flow-integration-test-testcases.md IT-007
//! （acceptance-criteria.md TC-EDGE-101-01、handlers/item_episodes.rs:21-49で確認済みの
//! エラーコードベース）

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use common::{build_full_router, test_app_state};

/// 【テスト用ヘルパー】: レスポンスボディをJSON Valueへ変換する
async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("レスポンスボディの読み取りに失敗しました");
    serde_json::from_slice(&bytes).expect("レスポンスボディのJSONパースに失敗しました")
}

/// IT-007 (EDGE-101): volume配下へのepisode登録拒否
/// 【テスト目的】: `group_type=volume`の`item_groups`に対し`POST /groups/:group_id/episodes`を
/// 呼ぶと拒否されることを確認する（シーズン/話数の構造的整合性を守るための重要なバリデーション）
/// 【テスト内容】: 事前に`group_type="volume"`のグループを作成し、そのgroup_idに対し
/// episode作成リクエストを送る
/// 【期待される動作】: 400、`error.code="INVALID_GROUP_TYPE_FOR_EPISODES"`。
/// `item_episodes`にレコードが作成されないことをDB直接SELECTで確認する
/// 🔵 信頼性レベル: main-flow-integration-test-testcases.md IT-007
/// （acceptance-criteria.md TC-EDGE-101-01、handlers/item_episodes.rs:21-49ベース）
#[tokio::test]
#[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
async fn it_007_creating_episode_under_volume_group_returns_400_and_creates_no_record() {
    let state = test_app_state().await;
    let app = build_full_router(state.clone());

    // 【テストデータ準備】: 親アイテムを作成する
    let create_item_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/items")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"media_type":"manga","title":"EDGE-101確認用マンガ"})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_item_response.status(), StatusCode::CREATED);
    let item_created = body_json(create_item_response).await;
    let item_id = item_created["data"]["id"].as_str().unwrap().to_string();

    // 【前提条件設定】: group_type="volume"のグループを作成する 🔵
    let create_group_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/items/{item_id}/groups"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "group_type": "volume",
                        "group_name": "第1巻",
                        "number": 1
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_group_response.status(), StatusCode::CREATED); // 【確認内容】: volumeグループの作成が201で成功することを確認 🔵
    let group_created = body_json(create_group_response).await;
    let group_id = group_created["data"]["id"].as_str().unwrap().to_string();
    let group_uuid: uuid::Uuid = group_id.parse().unwrap();

    // 【実際の処理実行】: volumeグループに対しepisode作成リクエストを送る 🔵
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/groups/{group_id}/episodes"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"episode_number": 1, "title": "誤って登録される話数"})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // 【結果検証】: 400・error.code="INVALID_GROUP_TYPE_FOR_EPISODES"であることを確認する 🔵
    assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: volume配下へのepisode登録が400で拒否されることを確認 🔵
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "INVALID_GROUP_TYPE_FOR_EPISODES"); // 【確認内容】: エラーコードがINVALID_GROUP_TYPE_FOR_EPISODESであることを確認 🔵

    // 【結果検証】: item_episodesにレコードが作成されていないことをDB直接SELECTで確認する（最重要） 🔵
    let episode_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM item_episodes WHERE group_id = $1")
            .bind(group_uuid)
            .fetch_one(&state.db)
            .await
            .expect("item_episodes件数取得に失敗しました");
    assert_eq!(episode_count, 0); // 【確認内容】: バリデーション拒否時にitem_episodesへレコードが作成されていないことを確認 🔵
}
