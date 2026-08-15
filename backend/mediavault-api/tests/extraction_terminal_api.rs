use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use mediavault_api::AppState;
use mediavault_api::routes::{build_router, internal::build_internal_router};
use tower::ServiceExt;
use uuid::Uuid;

const TEST_KEY: &str = "task-0014-key";

async fn app() -> (AppState, Router) {
    unsafe { std::env::set_var("INTERNAL_API_KEY", TEST_KEY) };
    let db = sqlx::PgPool::connect(
        &std::env::var("DATABASE_URL").expect("TASK-0014統合テストにはDATABASE_URLが必要です"),
    )
    .await
    .unwrap();
    let state = AppState {
        db,
        internal_api_key: TEST_KEY.to_string(),
    };
    let router = Router::new().nest(
        "/api/v1",
        build_router(state.clone()).merge(build_internal_router(state.clone())),
    );
    (state, router)
}

async fn running_fixture(state: &AppState, attempts: i32) -> (Uuid, Uuid, Uuid) {
    let item_id: Uuid = sqlx::query_scalar(
        "INSERT INTO items (media_type, title, status, is_favorite, source)
         VALUES ('novel', 'TASK-0014 terminal', 'not_started', false, 'manual') RETURNING id",
    )
    .fetch_one(&state.db)
    .await
    .unwrap();
    let file_id: Uuid = sqlx::query_scalar(
        "INSERT INTO item_files (item_id, path, file_type)
         VALUES ($1, $2, 'pdf') RETURNING id",
    )
    .bind(item_id)
    .bind(format!("task-0014-{item_id}.pdf"))
    .fetch_one(&state.db)
    .await
    .unwrap();
    let lease_token = Uuid::new_v4();
    let extraction_id: Uuid = sqlx::query_scalar(
        "INSERT INTO item_file_extractions
             (item_file_id, state, attempts, lease_token, lease_expires_at)
         VALUES ($1, 'running', $2, $3, CURRENT_TIMESTAMP + INTERVAL '5 minutes')
         RETURNING id",
    )
    .bind(file_id)
    .bind(attempts)
    .bind(lease_token)
    .fetch_one(&state.db)
    .await
    .unwrap();
    (item_id, extraction_id, lease_token)
}

async fn post(
    router: &Router,
    extraction_id: Uuid,
    action: &str,
    body: String,
) -> axum::response::Response {
    router
        .clone()
        .oneshot(
            Request::post(format!(
                "/api/v1/internal/extractions/{extraction_id}/{action}"
            ))
            .header("authorization", format!("Bearer {TEST_KEY}"))
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap(),
        )
        .await
        .unwrap()
}

fn fail_body(lease_token: Uuid, retryable: bool) -> String {
    format!(
        r#"{{"lease_token":"{lease_token}","error":{{"kind":"ocr_failed","message":"OCR失敗","retryable":{retryable}}}}}"#
    )
}

async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn cleanup(state: &AppState, item_id: Uuid) {
    sqlx::query("DELETE FROM items WHERE id = $1")
        .bind(item_id)
        .execute(&state.db)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore]
async fn fail_retries_below_limit_without_incrementing_attempts() {
    let (state, router) = app().await;
    let (item_id, extraction_id, token) = running_fixture(&state, 1).await;
    let response = post(&router, extraction_id, "fail", fail_body(token, true)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body["data"]["state"], "queued");
    assert_eq!(body["data"]["attempts"], 1);
    assert_eq!(body["data"]["error"]["kind"], "ocr_failed");
    let lease: Option<Uuid> =
        sqlx::query_scalar("SELECT lease_token FROM item_file_extractions WHERE id = $1")
            .bind(extraction_id)
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert!(lease.is_none());
    cleanup(&state, item_id).await;
}

#[tokio::test]
#[ignore]
async fn fail_is_terminal_when_permanent_or_attempt_limit_reached() {
    for (attempts, retryable) in [(0, false), (3, true)] {
        let (state, router) = app().await;
        let (item_id, extraction_id, token) = running_fixture(&state, attempts).await;
        let response = post(&router, extraction_id, "fail", fail_body(token, retryable)).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body_json(response).await["data"]["state"], "failed");
        cleanup(&state, item_id).await;
    }
}

#[tokio::test]
#[ignore]
async fn fail_and_cancelled_reject_wrong_or_ineligible_lease() {
    let (state, router) = app().await;
    let (item_id, extraction_id, token) = running_fixture(&state, 1).await;
    let wrong_fail = post(
        &router,
        extraction_id,
        "fail",
        fail_body(Uuid::new_v4(), true),
    )
    .await;
    assert_eq!(wrong_fail.status(), StatusCode::CONFLICT);

    let direct_cancel = post(
        &router,
        extraction_id,
        "cancelled",
        format!(r#"{{"lease_token":"{token}"}}"#),
    )
    .await;
    assert_eq!(direct_cancel.status(), StatusCode::CONFLICT);
    cleanup(&state, item_id).await;
}

#[tokio::test]
#[ignore]
async fn cancelling_can_be_confirmed_without_creating_text() {
    let (state, router) = app().await;
    let (item_id, extraction_id, token) = running_fixture(&state, 1).await;
    sqlx::query("UPDATE item_file_extractions SET state = 'cancelling' WHERE id = $1")
        .bind(extraction_id)
        .execute(&state.db)
        .await
        .unwrap();
    let before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM item_file_texts")
        .fetch_one(&state.db)
        .await
        .unwrap();

    let response = post(
        &router,
        extraction_id,
        "cancelled",
        format!(r#"{{"lease_token":"{token}"}}"#),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_json(response).await["data"]["state"], "cancelled");
    let after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM item_file_texts")
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(after, before);
    cleanup(&state, item_id).await;
}

#[tokio::test]
async fn fail_rejects_oversized_message_before_db_access() {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgresql://postgres:postgres@localhost/mediavault_test")
        .unwrap();
    unsafe { std::env::set_var("INTERNAL_API_KEY", TEST_KEY) };
    let state = AppState {
        db: pool,
        internal_api_key: TEST_KEY.to_string(),
    };
    let router = Router::new().nest("/api/v1", build_internal_router(state));
    let body = format!(
        r#"{{"lease_token":"{}","error":{{"kind":"internal","message":"{}","retryable":false}}}}"#,
        Uuid::new_v4(),
        "a".repeat(4_001)
    );
    let response = post(&router, Uuid::new_v4(), "fail", body).await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
