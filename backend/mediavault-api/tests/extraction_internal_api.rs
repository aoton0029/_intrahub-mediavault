use std::path::PathBuf;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use mediavault_api::AppState;
use mediavault_api::routes::{build_router, internal::build_internal_router};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;
use uuid::Uuid;

const TEST_KEY: &str = "task-0011-key";

fn app() -> Router {
    unsafe {
        std::env::set_var("INTERNAL_API_KEY", TEST_KEY);
    }
    let db = PgPoolOptions::new()
        .connect_lazy("postgres://postgres:postgres@127.0.0.1:1/mediavault")
        .unwrap();
    let state = AppState {
        db,
        internal_api_key: TEST_KEY.to_string(),
    };
    Router::new().nest(
        "/api/v1",
        build_router(state.clone()).merge(build_internal_router(state)),
    )
}

#[tokio::test]
async fn claim_without_authentication_returns_401() {
    let response = app()
        .oneshot(
            Request::post("/api/v1/internal/extractions/claim")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"worker_id":"worker-1","lease_seconds":300}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn claim_rejects_non_positive_lease_before_db_access() {
    let response = app()
        .oneshot(
            Request::post("/api/v1/internal/extractions/claim")
                .header("authorization", format!("Bearer {TEST_KEY}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"worker_id":"worker-1","lease_seconds":0}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn claim_is_not_exposed_on_the_public_route() {
    let response = app()
        .oneshot(
            Request::post("/api/v1/extractions/claim")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"worker_id":"worker-1","lease_seconds":300}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn heartbeat_without_authentication_returns_401() {
    let response = app()
        .oneshot(
            Request::post(format!(
                "/api/v1/internal/extractions/{}/heartbeat",
                Uuid::new_v4()
            ))
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"lease_token":"{}","lease_seconds":300}}"#,
                Uuid::new_v4()
            )))
            .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn heartbeat_rejects_non_positive_lease_before_db_access() {
    let response = app()
        .oneshot(
            Request::post(format!(
                "/api/v1/internal/extractions/{}/heartbeat",
                Uuid::new_v4()
            ))
            .header("authorization", format!("Bearer {TEST_KEY}"))
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"lease_token":"{}","lease_seconds":0}}"#,
                Uuid::new_v4()
            )))
            .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

async fn db_app() -> (AppState, Router) {
    unsafe {
        std::env::set_var("INTERNAL_API_KEY", TEST_KEY);
    }
    let root = std::env::temp_dir().join(format!("mediavault-task-0011-{}", std::process::id()));
    std::fs::create_dir_all(root.join("files")).unwrap();
    unsafe {
        std::env::set_var("STORAGE_ROOT", &root);
        std::env::set_var("STORAGE_SUBDIR_FILES", "files");
    }
    let db = sqlx::PgPool::connect(
        &std::env::var("DATABASE_URL").expect("TASK-0011統合テストにはDATABASE_URLが必要です"),
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

async fn queued_fixture(state: &AppState, write_file: bool) -> (Uuid, Uuid, Uuid) {
    let item_id: Uuid = sqlx::query_scalar(
        "INSERT INTO items (media_type, title, status, is_favorite, source)
         VALUES ('novel', 'TASK-0011 claim', 'not_started', false, 'manual') RETURNING id",
    )
    .fetch_one(&state.db)
    .await
    .unwrap();
    let relative_path = format!("{item_id}.pdf");
    if write_file {
        let root = std::env::var("STORAGE_ROOT").unwrap();
        std::fs::write(
            PathBuf::from(root).join("files").join(&relative_path),
            b"pdf",
        )
        .unwrap();
    }
    let file_id: Uuid = sqlx::query_scalar(
        "INSERT INTO item_files (item_id, path, file_type)
         VALUES ($1, $2, 'pdf') RETURNING id",
    )
    .bind(item_id)
    .bind(relative_path)
    .fetch_one(&state.db)
    .await
    .unwrap();
    let extraction_id: Uuid = sqlx::query_scalar(
        "INSERT INTO item_file_extractions (item_file_id, created_at)
         VALUES ($1, CURRENT_TIMESTAMP - INTERVAL '100 years') RETURNING id",
    )
    .bind(file_id)
    .fetch_one(&state.db)
    .await
    .unwrap();
    (item_id, file_id, extraction_id)
}

async fn claim(router: &Router, worker: &str) -> axum::response::Response {
    router
        .clone()
        .oneshot(
            Request::post("/api/v1/internal/extractions/claim")
                .header("authorization", format!("Bearer {TEST_KEY}"))
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"worker_id":"{worker}","lease_seconds":300}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn heartbeat(
    router: &Router,
    extraction_id: Uuid,
    lease_token: Uuid,
    body: &str,
) -> axum::response::Response {
    router
        .clone()
        .oneshot(
            Request::post(format!(
                "/api/v1/internal/extractions/{extraction_id}/heartbeat"
            ))
            .header("authorization", format!("Bearer {TEST_KEY}"))
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"lease_token":"{lease_token}",{body}}}"#
            )))
            .unwrap(),
        )
        .await
        .unwrap()
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
async fn claim_returns_lease_file_ref_and_increments_attempts() {
    let (state, router) = db_app().await;
    let (item_id, file_id, extraction_id) = queued_fixture(&state, true).await;
    let response = claim(&router, "worker-1").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body["data"]["extraction_id"], extraction_id.to_string());
    assert_eq!(body["data"]["item_file_id"], file_id.to_string());
    assert_eq!(body["data"]["attempts"], 1);
    assert_eq!(body["data"]["file_ref"]["root"], "storage");
    assert!(body["data"]["lease_token"].as_str().is_some());
    cleanup(&state, item_id).await;
}

#[tokio::test]
#[ignore]
async fn parallel_workers_do_not_receive_the_same_extraction() {
    let (state, router) = db_app().await;
    let (item_id, _, extraction_id) = queued_fixture(&state, true).await;
    let (first, second) = tokio::join!(claim(&router, "worker-a"), claim(&router, "worker-b"));
    let first = body_json(first).await;
    let second = body_json(second).await;
    let claimed = [
        first["data"]["extraction_id"].as_str(),
        second["data"]["extraction_id"].as_str(),
    ]
    .into_iter()
    .flatten()
    .filter(|id| *id == extraction_id.to_string())
    .count();
    assert_eq!(claimed, 1);
    cleanup(&state, item_id).await;
}

#[tokio::test]
#[ignore]
async fn expired_lease_is_reclaimed_and_exhausted_lease_is_swept() {
    let (state, router) = db_app().await;
    let (item_id, _, extraction_id) = queued_fixture(&state, true).await;
    sqlx::query(
        "UPDATE item_file_extractions SET state = 'running', attempts = 1,
         lease_token = gen_random_uuid(), lease_expires_at = CURRENT_TIMESTAMP - INTERVAL '1 minute'
         WHERE id = $1",
    )
    .bind(extraction_id)
    .execute(&state.db)
    .await
    .unwrap();
    let old_token: Uuid =
        sqlx::query_scalar("SELECT lease_token FROM item_file_extractions WHERE id = $1")
            .bind(extraction_id)
            .fetch_one(&state.db)
            .await
            .unwrap();
    let body = body_json(claim(&router, "worker-retry").await).await;
    assert_eq!(body["data"]["attempts"], 2);
    assert_ne!(body["data"]["lease_token"], old_token.to_string());

    sqlx::query(
        "UPDATE item_file_extractions SET attempts = max_attempts,
         lease_expires_at = CURRENT_TIMESTAMP - INTERVAL '1 minute' WHERE id = $1",
    )
    .bind(extraction_id)
    .execute(&state.db)
    .await
    .unwrap();
    let _ = claim(&router, "worker-sweeper").await;
    let state_value: String =
        sqlx::query_scalar("SELECT state::text FROM item_file_extractions WHERE id = $1")
            .bind(extraction_id)
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(state_value, "failed");
    cleanup(&state, item_id).await;
}

#[tokio::test]
#[ignore]
async fn missing_file_is_failed_instead_of_returned_to_worker() {
    let (state, router) = db_app().await;
    let (item_id, _, extraction_id) = queued_fixture(&state, false).await;
    let body = body_json(claim(&router, "worker-missing").await).await;
    assert!(body["data"].is_null());
    let (state_value, error): (String, serde_json::Value) =
        sqlx::query_as("SELECT state::text, error FROM item_file_extractions WHERE id = $1")
            .bind(extraction_id)
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(state_value, "failed");
    assert_eq!(error["kind"], "file_not_found");
    cleanup(&state, item_id).await;
}

#[tokio::test]
#[ignore]
async fn heartbeat_extends_lease_and_partially_updates_public_progress() {
    let (state, router) = db_app().await;
    let (item_id, file_id, extraction_id) = queued_fixture(&state, true).await;
    let claimed = body_json(claim(&router, "worker-heartbeat").await).await;
    let lease_token = Uuid::parse_str(claimed["data"]["lease_token"].as_str().unwrap()).unwrap();
    let previous_expiry: chrono::NaiveDateTime = sqlx::query_scalar(
        "UPDATE item_file_extractions SET progress_total = 10 WHERE id = $1
         RETURNING lease_expires_at",
    )
    .bind(extraction_id)
    .fetch_one(&state.db)
    .await
    .unwrap();

    let response = heartbeat(
        &router,
        extraction_id,
        lease_token,
        r#""progress_current":5,"lease_seconds":600"#,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body["data"]["state"], "running");
    assert_eq!(body["data"]["cancel_requested"], false);

    let public_response = router
        .clone()
        .oneshot(
            Request::get(format!(
                "/api/v1/items/{item_id}/files/{file_id}/extraction"
            ))
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
    let public_body = body_json(public_response).await;
    assert_eq!(public_body["data"]["progress_current"], 5);
    assert_eq!(public_body["data"]["progress_total"], 10);
    let new_expiry: chrono::NaiveDateTime =
        sqlx::query_scalar("SELECT lease_expires_at FROM item_file_extractions WHERE id = $1")
            .bind(extraction_id)
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert!(new_expiry > previous_expiry);
    cleanup(&state, item_id).await;
}

#[tokio::test]
#[ignore]
async fn cancelling_extraction_returns_cancel_requested() {
    let (state, router) = db_app().await;
    let (item_id, file_id, extraction_id) = queued_fixture(&state, true).await;
    let claimed = body_json(claim(&router, "worker-cancel").await).await;
    let lease_token = Uuid::parse_str(claimed["data"]["lease_token"].as_str().unwrap()).unwrap();

    let cancel_response = router
        .clone()
        .oneshot(
            Request::post(format!(
                "/api/v1/items/{item_id}/files/{file_id}/extraction/cancel"
            ))
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cancel_response.status(), StatusCode::OK);

    let response = heartbeat(
        &router,
        extraction_id,
        lease_token,
        r#""lease_seconds":300"#,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body["data"]["state"], "cancelling");
    assert_eq!(body["data"]["cancel_requested"], true);
    cleanup(&state, item_id).await;
}

#[tokio::test]
#[ignore]
async fn heartbeat_rejects_wrong_token_and_terminal_state() {
    let (state, router) = db_app().await;
    let (item_id, _, extraction_id) = queued_fixture(&state, true).await;
    let claimed = body_json(claim(&router, "worker-token").await).await;
    let lease_token = Uuid::parse_str(claimed["data"]["lease_token"].as_str().unwrap()).unwrap();

    let wrong = heartbeat(
        &router,
        extraction_id,
        Uuid::new_v4(),
        r#""lease_seconds":300"#,
    )
    .await;
    assert_eq!(wrong.status(), StatusCode::CONFLICT);
    assert_eq!(
        body_json(wrong).await["error"]["code"],
        "INVALID_LEASE_TOKEN"
    );

    sqlx::query(
        "UPDATE item_file_extractions SET state = 'succeeded', lease_token = NULL,
         lease_expires_at = NULL WHERE id = $1",
    )
    .bind(extraction_id)
    .execute(&state.db)
    .await
    .unwrap();
    let terminal = heartbeat(
        &router,
        extraction_id,
        lease_token,
        r#""lease_seconds":300"#,
    )
    .await;
    assert_eq!(terminal.status(), StatusCode::CONFLICT);
    assert_eq!(
        body_json(terminal).await["error"]["code"],
        "INVALID_LEASE_TOKEN"
    );
    cleanup(&state, item_id).await;
}

#[tokio::test]
#[ignore]
async fn heartbeat_rejects_token_replaced_by_reclaim() {
    let (state, router) = db_app().await;
    let (item_id, _, extraction_id) = queued_fixture(&state, true).await;
    let first = body_json(claim(&router, "worker-old").await).await;
    let old_token = Uuid::parse_str(first["data"]["lease_token"].as_str().unwrap()).unwrap();
    sqlx::query(
        "UPDATE item_file_extractions SET lease_expires_at = CURRENT_TIMESTAMP - INTERVAL '1 minute'
         WHERE id = $1",
    )
    .bind(extraction_id)
    .execute(&state.db)
    .await
    .unwrap();
    let _ = claim(&router, "worker-new").await;

    let response = heartbeat(&router, extraction_id, old_token, r#""lease_seconds":300"#).await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert_eq!(
        body_json(response).await["error"]["code"],
        "INVALID_LEASE_TOKEN"
    );
    cleanup(&state, item_id).await;
}
