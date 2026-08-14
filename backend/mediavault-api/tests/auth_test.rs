//! TASK-0032: 主要フロー統合テスト — 内部APIキー認証（IT-006, IT-008, IT-009）
//!
//! 正しいキー／キーなし／誤りキーの3パターンで`/api/v1/internal/*`エンドポイントへのアクセス結果
//! （201系 vs 401）を検証する。
//! 🔵 信頼性レベル: main-flow-integration-test-testcases.md IT-006/IT-008/IT-009
//! （acceptance-criteria.md TC-018-01、middleware/api_key_auth.rsベース）

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use common::{TEST_INTERNAL_API_KEY, build_full_router, internal_api_key, test_app_state};

/// 【テスト用ヘルパー】: 有効な最小CreateItemRequest相当のJSONボディを返す
fn valid_create_item_body() -> String {
    serde_json::json!({"title": "内部APIキー認証テスト用アイテム", "media_type": "anime"})
        .to_string()
}

/// IT-006: 内部APIキー認証（正しいキー）
/// 【テスト目的】: 正しい`INTERNAL_API_KEY`を`Authorization: Bearer <key>`で送信し、
/// `/api/v1/internal/items`へのPOSTが成功することを確認する
/// 【テスト内容】: 正しいAPIキー、POST /api/v1/internal/itemsの有効なボディを送信する
/// 【期待される動作】: 201
/// 🔵 信頼性レベル: main-flow-integration-test-testcases.md IT-006
/// （acceptance-criteria.md TC-018-01、middleware/api_key_auth.rsベース）
#[tokio::test]
#[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
async fn it_006_internal_items_post_with_valid_key_succeeds() {
    // 【テストデータ準備】: 認証通過させるためINTERNAL_API_KEY環境変数を既知値に設定する 🔵
    internal_api_key(TEST_INTERNAL_API_KEY);
    let state = test_app_state().await;
    let app = build_full_router(state);

    // 【実際の処理実行】: 正しいAuthorizationヘッダー付きでPOST /api/v1/internal/itemsを実行する 🔵
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/internal/items")
                .header("authorization", format!("Bearer {TEST_INTERNAL_API_KEY}"))
                .header("content-type", "application/json")
                .body(Body::from(valid_create_item_body()))
                .unwrap(),
        )
        .await
        .unwrap();

    // 【結果検証】: 認証通過後にハンドラが実行され201が返ることを確認する 🔵
    assert_eq!(response.status(), StatusCode::CREATED); // 【確認内容】: 正しいキーでの内部API呼び出しが201で成功することを確認 🔵
}

/// IT-008: 内部APIキー認証（キーなし）
/// 【エラーケースの概要】: `Authorization`ヘッダーなしで内部エンドポイントへアクセスする
/// 【テスト内容】: ヘッダーなしでPOST /api/v1/internal/itemsを実行する
/// 【期待される結果】: 401 Unauthorized
/// 🟡 信頼性レベル: main-flow-integration-test-testcases.md IT-008（TC-018-E01相当。
/// acceptance-criteria.mdに直接の項番記載はないが、タスクファイル完了条件およびmiddleware/
/// api_key_auth.rsの実装から妥当に推測）
#[tokio::test]
#[ignore]
async fn it_008_internal_items_post_without_auth_header_returns_401() {
    internal_api_key(TEST_INTERNAL_API_KEY);
    let state = test_app_state().await;
    let app = build_full_router(state);

    // 【実際の処理実行】: Authorizationヘッダーなしでリクエストする 🟡
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/internal/items")
                .header("content-type", "application/json")
                .body(Body::from(valid_create_item_body()))
                .unwrap(),
        )
        .await
        .unwrap();

    // 【結果検証】: 認証ミドルウェアで遮断され401が返ることを確認する 🟡
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED); // 【確認内容】: 無認証アクセスが401で拒否されることを確認 🟡
}

/// IT-009: 内部APIキー認証（誤ったキー）
/// 【エラーケースの概要】: 誤った値を`Authorization: Bearer wrong-key`として送信する
/// 【テスト内容】: 誤ったAPIキー文字列でPOST /api/v1/internal/itemsを実行する
/// 【期待される結果】: 401 Unauthorized
/// 🟡 信頼性レベル: main-flow-integration-test-testcases.md IT-009（TC-018-E02相当。
/// middleware実装の比較ロジックから妥当に推測）
#[tokio::test]
#[ignore]
async fn it_009_internal_items_post_with_wrong_key_returns_401() {
    internal_api_key(TEST_INTERNAL_API_KEY);
    let state = test_app_state().await;
    let app = build_full_router(state);

    // 【実際の処理実行】: 不一致のAPIキーでリクエストする 🟡
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/internal/items")
                .header("authorization", "Bearer wrong-key")
                .header("content-type", "application/json")
                .body(Body::from(valid_create_item_body()))
                .unwrap(),
        )
        .await
        .unwrap();

    // 【結果検証】: キー不一致が401で拒否されることを確認する 🟡
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED); // 【確認内容】: 誤ったキーでのアクセスが401で拒否されることを確認 🟡
}
