//! TASK-0001: relation_type 6種別拡張の統合テスト
//!
//! 🔵 信頼性レベル: tasks/TASK-0001.md 統合テスト1・2 より

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

/// 【テスト用ヘルパー】: アイテムを1件作成しIDを返す
async fn create_item(app: &axum::Router, title: &str) -> String {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/items")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"media_type": "anime", "title": title}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    body_json(response).await["data"]["id"]
        .as_str()
        .unwrap()
        .to_string()
}

/// 【テスト用ヘルパー】: 関連付け作成リクエストを送る
async fn post_relation(
    app: &axum::Router,
    item_id: &str,
    related_item_id: &str,
    relation_type: &str,
) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/item-relations")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "item_id": item_id,
                        "related_item_id": related_item_id,
                        "relation_type": relation_type
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap()
}

/// 統合テスト1: 6種別の登録と取得
/// 【テスト目的】: 拡張後の relation_type ENUM 6値すべてが DB へ登録でき、取得時に同じ種別で返ることを確認する
/// 【期待される動作】: すべて 201 で登録され、`GET /items/{id}/relations` で6件すべてが同じ種別で返る
/// 🔵 信頼性レベル: TASK-0001 完了条件より
#[tokio::test]
#[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
async fn all_six_relation_types_can_be_created_and_retrieved() {
    let state = test_app_state().await;
    let app = build_full_router(state.clone());

    let item_id = create_item(&app, "TASK-0001 起点アイテム").await;

    let relation_types = [
        "adaptation",
        "sequel",
        "prequel",
        "spinoff",
        "dlc",
        "reference",
    ];

    // 【実際の処理実行】: 種別ごとに別の終点アイテムを用意して登録する 🔵
    for relation_type in relation_types {
        let related_item_id = create_item(&app, &format!("TASK-0001 終点 {relation_type}")).await;

        let response = post_relation(&app, &item_id, &related_item_id, relation_type).await;

        assert_eq!(
            response.status(),
            StatusCode::CREATED,
            "{relation_type} の登録が201になりませんでした"
        ); // 【確認内容】: 6種別すべてが受理されることを確認 🔵
        let json = body_json(response).await;
        assert_eq!(json["data"]["relation_type"], relation_type); // 【確認内容】: レスポンスに登録した種別がそのまま返ることを確認 🔵
    }

    // 【結果検証】: 一覧取得で6件すべてが同じ種別で返ることを確認する 🔵
    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/items/{item_id}/relations"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = body_json(list_response).await;
    let relations = list_json["data"].as_array().unwrap();

    assert_eq!(relations.len(), relation_types.len()); // 【確認内容】: 登録した6件がすべて取得できることを確認 🔵
    let mut retrieved: Vec<&str> = relations
        .iter()
        .map(|r| r["relation_type"].as_str().unwrap())
        .collect();
    retrieved.sort_unstable();
    let mut expected = relation_types;
    expected.sort_unstable();
    assert_eq!(retrieved, expected); // 【確認内容】: 6種別が往復で欠落・変化しないことを確認 🔵
}

/// 統合テスト2: 一意制約の維持
/// 【テスト目的】: 追加した種別でも `uq_item_relations (item_id, related_item_id, relation_type)` が働くことを確認する
/// 【期待される動作】: 同一の (item_id, related_item_id, adaptation) の2回目は 409 `DUPLICATE_RELATION`
/// 🔵 信頼性レベル: init_schema.up.sql の uq_item_relations 制約より
#[tokio::test]
#[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
async fn duplicate_adaptation_relation_returns_409() {
    let state = test_app_state().await;
    let app = build_full_router(state.clone());

    let item_id = create_item(&app, "TASK-0001 重複確認 原作").await;
    let related_item_id = create_item(&app, "TASK-0001 重複確認 映像化").await;

    // 【前提条件設定】: 1回目の登録は成功する 🔵
    let first = post_relation(&app, &item_id, &related_item_id, "adaptation").await;
    assert_eq!(first.status(), StatusCode::CREATED);

    // 【実際の処理実行】: 同じ組み合わせをもう一度登録する 🔵
    let second = post_relation(&app, &item_id, &related_item_id, "adaptation").await;

    // 【結果検証】: 409 DUPLICATE_RELATION で拒否されることを確認する 🔵
    assert_eq!(second.status(), StatusCode::CONFLICT); // 【確認内容】: 追加種別でも一意制約が維持されていることを確認 🔵
    let json = body_json(second).await;
    assert_eq!(json["error"]["code"], "DUPLICATE_RELATION"); // 【確認内容】: エラーコードがDUPLICATE_RELATIONであることを確認 🔵
}

/// 【テスト目的】: ENUM に無い値が 400 VALIDATION_ERROR で拒否されることを確認する
/// 🔵 信頼性レベル: TASK-0001 完了条件「不正な値は 400 VALIDATION_ERROR で拒否される」より
#[tokio::test]
#[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
async fn unknown_relation_type_is_rejected_with_400() {
    let state = test_app_state().await;
    let app = build_full_router(state.clone());

    let item_id = create_item(&app, "TASK-0001 不正値確認 起点").await;
    let related_item_id = create_item(&app, "TASK-0001 不正値確認 終点").await;

    // 【実際の処理実行】: ENUM に存在しない種別を送る 🔵
    let response = post_relation(&app, &item_id, &related_item_id, "inspired_by").await;

    // 【結果検証】: 400 VALIDATION_ERROR で拒否されることを確認する 🔵
    assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: 未定義の種別が受理されないことを確認 🔵
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "VALIDATION_ERROR"); // 【確認内容】: エラーコードがVALIDATION_ERRORであることを確認 🔵
}
