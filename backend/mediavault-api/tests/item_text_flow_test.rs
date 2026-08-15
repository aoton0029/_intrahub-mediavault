use axum::body::Body;
use axum::http::{Request, StatusCode};
use mediavault_api::AppState;
use mediavault_api::routes::build_router;
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

fn test_app() -> axum::Router {
    let db = PgPoolOptions::new()
        .connect_lazy("postgres://postgres:postgres@127.0.0.1:1/mediavault")
        .expect("lazy test pool should be valid");
    build_router(AppState {
        db,
        internal_api_key: "test-key".to_string(),
    })
}

async fn response_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    serde_json::from_slice(&bytes).expect("response should be JSON")
}

#[tokio::test]
async fn get_item_text_rejects_invalid_item_uuid_before_db_access() {
    let response = test_app()
        .oneshot(
            Request::get("/items/not-a-uuid/text")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_json(response).await;
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}
