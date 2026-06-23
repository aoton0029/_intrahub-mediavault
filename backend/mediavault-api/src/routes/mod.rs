//! ルーター構築
//!
//! TASK-0007: Axumルーター骨格・DB接続プール設定・main.rs実装

use axum::routing::get;
use axum::Router;

use crate::handlers::health::health_handler;
use crate::handlers::items::{
    create_item_handler, get_item_handler, list_items_handler, update_item_handler,
};
use crate::AppState;

/// アプリケーション全体のRouterを構築する。
/// Phase 2以降のルートはこのRouterに `.merge()` や `.nest()` で追加していく。
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        // 【TASK-0010】: GET /items（一覧・絞り込み）を既存POST /itemsと同一パスに追加 🔵
        .route("/items", get(list_items_handler).post(create_item_handler))
        // 【TASK-0011】: GET /items/:id（個別詳細取得） 🟡
        // 【TASK-0012】: PATCH /items/:id（部分更新）を同一パスに追加 🔵
        .route(
            "/items/:id",
            get(get_item_handler).patch(update_item_handler),
        )
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    /// 【テスト用ヘルパー】: GET /itemsルーティング統合テスト用にAppStateを構築する。
    /// DATABASE_URL環境変数（テスト用Postgres）への接続が必要なため、本ヘルパーを使う
    /// テストはすべて#[ignore]とし、`cargo test -- --ignored` で実行する 🟡
    async fn test_app_state() -> AppState {
        let database_url = std::env::var("DATABASE_URL")
            .expect("TASK-0010ルーティング統合テストにはDATABASE_URL環境変数が必要です");
        let db = sqlx::PgPool::connect(&database_url)
            .await
            .expect("テスト用DBへの接続に失敗しました");
        AppState {
            db,
            internal_api_key: String::new(),
        }
    }

    /// TC-0010-E01: 不正なmedia_type値 → 400（ルーター経由）
    /// 🔵 信頼性レベル: 要件 EC-1・TASK-0010 注意事項に直接対応
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn get_items_with_invalid_media_type_returns_400() {
        // 【テスト目的】: media_type=invalidのようなenum外の値がAxumのQuery抽出段階で
        // 400 Bad Requestとして拒否されることを確認する
        // 【テスト内容】: GET /items?media_type=invalid をルーター経由で実行する
        // 【期待される動作】: レスポンスステータスが400であること
        let state = test_app_state().await;
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/items?media_type=invalid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: 不正なenum値が400で拒否されることを確認 🔵
    }

    /// TC-0010-E02: 不正なpage値（非数値）→ 400（ルーター経由）
    /// 🔵 信頼性レベル: 要件 EC-1・note.md 6章 技術的制約に直接対応
    #[tokio::test]
    #[ignore]
    async fn get_items_with_non_numeric_page_returns_400() {
        // 【テスト目的】: page=abcのようなu32にパースできない値が400で拒否されることを確認する
        let state = test_app_state().await;
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/items?page=abc")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: 数値以外のpageが400で拒否されることを確認 🔵
    }

    /// TC-0010-E03: 不正なis_favorite値（bool以外）→ 400（ルーター経由）
    /// 🟡 信頼性レベル: 要件 入力仕様表（is_favorite: true/false）からの妥当な推測
    #[tokio::test]
    #[ignore]
    async fn get_items_with_non_boolean_is_favorite_returns_400() {
        // 【テスト目的】: is_favorite=yesのようなbool以外の値が400で拒否されることを確認する
        let state = test_app_state().await;
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/items?is_favorite=yes")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: bool以外のis_favoriteが400で拒否されることを確認 🟡
    }

    /// TC-0011-E01: 存在しないitemで404（ルーター経由、実DB必要）
    /// 🔵 信頼性レベル: タスクファイル テストケース2に直接対応
    #[tokio::test]
    #[ignore]
    async fn get_item_with_nonexistent_id_returns_404() {
        let state = test_app_state().await;
        let app = build_router(state);
        let id = uuid::Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/items/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND); // 【確認内容】: 存在しないitemで404が返ることを確認 🔵
    }

    /// TC-0011-E02: 不正なUUID形式で400（ルーター経由、実DB必要）
    /// 🟡 信頼性レベル: タスクファイル テストケース3に直接対応
    #[tokio::test]
    #[ignore]
    async fn get_item_with_invalid_uuid_returns_400() {
        let state = test_app_state().await;
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/items/not-a-uuid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: 不正なUUID形式が400で拒否されることを確認 🟡
    }
}
