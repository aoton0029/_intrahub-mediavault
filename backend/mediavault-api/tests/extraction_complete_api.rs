use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use mediavault_api::AppState;
use mediavault_api::routes::{build_router, internal::build_internal_router};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;
use uuid::Uuid;

const TEST_KEY: &str = "task-0013-key";

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

fn complete_body(lease_token: Uuid, content: &str, boundary_end: i64) -> String {
    serde_json::json!({
        "lease_token": lease_token,
        "content": content,
        "boundaries": [{"start": 0, "end": boundary_end, "label": "p.1"}],
        "extraction_version": "pdf-v1",
        "extracted_at": "2026-08-15T12:00:00",
        "extractor": {
            "method": "mixed",
            "embedded_text_pages": 1,
            "ocr_pages": 1,
            "ocr": {"engine": "yomitoku", "device": "cpu", "model": "v1"}
        }
    })
    .to_string()
}

#[tokio::test]
async fn complete_without_authentication_returns_401() {
    let response = app()
        .oneshot(
            Request::post(format!(
                "/api/v1/internal/extractions/{}/complete",
                Uuid::new_v4()
            ))
            .header("content-type", "application/json")
            .body(Body::from(complete_body(Uuid::new_v4(), "abc", 3)))
            .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn complete_rejects_invalid_boundaries_before_db_access() {
    let response = app()
        .oneshot(
            Request::post(format!(
                "/api/v1/internal/extractions/{}/complete",
                Uuid::new_v4()
            ))
            .header("authorization", format!("Bearer {TEST_KEY}"))
            .header("content-type", "application/json")
            .body(Body::from(complete_body(Uuid::new_v4(), "abc", 4)))
            .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn complete_rejects_oversized_content_before_db_access() {
    let content = "a".repeat(5_000_001);
    let response = app()
        .oneshot(
            Request::post(format!(
                "/api/v1/internal/extractions/{}/complete",
                Uuid::new_v4()
            ))
            .header("authorization", format!("Bearer {TEST_KEY}"))
            .header("content-type", "application/json")
            .body(Body::from(complete_body(
                Uuid::new_v4(),
                &content,
                content.len() as i64,
            )))
            .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

async fn db_app() -> (AppState, Router) {
    unsafe {
        std::env::set_var("INTERNAL_API_KEY", TEST_KEY);
    }
    let db = sqlx::PgPool::connect(
        &std::env::var("DATABASE_URL").expect("TASK-0013統合テストにはDATABASE_URLが必要です"),
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

async fn running_fixture(state: &AppState) -> (Uuid, Uuid, Uuid, Uuid) {
    let item_id: Uuid = sqlx::query_scalar(
        "INSERT INTO items (media_type, title, status, is_favorite, source)
         VALUES ('novel', 'TASK-0013 complete', 'not_started', false, 'manual') RETURNING id",
    )
    .fetch_one(&state.db)
    .await
    .unwrap();
    let file_id: Uuid = sqlx::query_scalar(
        "INSERT INTO item_files (item_id, path, file_type)
         VALUES ($1, $2, 'pdf') RETURNING id",
    )
    .bind(item_id)
    .bind(format!("task-0013-{item_id}.pdf"))
    .fetch_one(&state.db)
    .await
    .unwrap();
    let lease_token = Uuid::new_v4();
    let extraction_id: Uuid = sqlx::query_scalar(
        "INSERT INTO item_file_extractions
             (item_file_id, state, attempts, lease_token, lease_expires_at,
              progress_current, progress_total)
         VALUES ($1, 'running', 1, $2, CURRENT_TIMESTAMP + INTERVAL '5 minutes', 1, 2)
         RETURNING id",
    )
    .bind(file_id)
    .bind(lease_token)
    .fetch_one(&state.db)
    .await
    .unwrap();
    (item_id, file_id, extraction_id, lease_token)
}

async fn post_complete(
    router: &Router,
    extraction_id: Uuid,
    lease_token: Uuid,
    content: &str,
    version: &str,
) -> axum::response::Response {
    let mut body: serde_json::Value = serde_json::from_str(&complete_body(
        lease_token,
        content,
        content.chars().count() as i64,
    ))
    .unwrap();
    body["extraction_version"] = version.into();
    router
        .clone()
        .oneshot(
            Request::post(format!(
                "/api/v1/internal/extractions/{extraction_id}/complete"
            ))
            .header("authorization", format!("Bearer {TEST_KEY}"))
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
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
async fn complete_atomically_saves_all_fields_and_succeeds() {
    let (state, router) = db_app().await;
    let (item_id, file_id, extraction_id, lease_token) = running_fixture(&state).await;
    let response = post_complete(&router, extraction_id, lease_token, "日本語本文", "pdf-v1").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body["data"]["state"], "succeeded");
    assert_eq!(body["data"]["progress_current"], 2);

    let (content, boundaries, version, extractor): (
        String,
        serde_json::Value,
        String,
        serde_json::Value,
    ) = sqlx::query_as(
        "SELECT content, boundaries, extraction_version, extractor
         FROM item_file_texts WHERE item_file_id = $1",
    )
    .bind(file_id)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(content, "日本語本文");
    assert_eq!(boundaries[0]["end"], 5);
    assert_eq!(version, "pdf-v1");
    assert_eq!(extractor["method"], "mixed");
    assert_eq!(extractor["ocr"]["engine"], "yomitoku");
    cleanup(&state, item_id).await;
}

#[tokio::test]
#[ignore]
async fn complete_rejects_stale_token_and_cancelling_without_writing_text() {
    let (state, router) = db_app().await;
    let (item_id, file_id, extraction_id, lease_token) = running_fixture(&state).await;
    let stale = post_complete(&router, extraction_id, Uuid::new_v4(), "stale", "pdf-v1").await;
    assert_eq!(stale.status(), StatusCode::CONFLICT);

    sqlx::query("UPDATE item_file_extractions SET state = 'cancelling' WHERE id = $1")
        .bind(extraction_id)
        .execute(&state.db)
        .await
        .unwrap();
    let cancelling = post_complete(&router, extraction_id, lease_token, "cancel", "pdf-v1").await;
    assert_eq!(cancelling.status(), StatusCode::CONFLICT);
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM item_file_texts WHERE item_file_id = $1")
            .bind(file_id)
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(count, 0);
    cleanup(&state, item_id).await;
}

#[tokio::test]
#[ignore]
async fn reextraction_replaces_text_but_preserves_extraction_history() {
    let (state, router) = db_app().await;
    let (item_id, file_id, first_id, first_token) = running_fixture(&state).await;
    assert_eq!(
        post_complete(&router, first_id, first_token, "first", "pdf-v1")
            .await
            .status(),
        StatusCode::OK
    );
    let second_token = Uuid::new_v4();
    let second_id: Uuid = sqlx::query_scalar(
        "INSERT INTO item_file_extractions
             (item_file_id, state, attempts, lease_token, lease_expires_at)
         VALUES ($1, 'running', 1, $2, CURRENT_TIMESTAMP + INTERVAL '5 minutes') RETURNING id",
    )
    .bind(file_id)
    .bind(second_token)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(
        post_complete(&router, second_id, second_token, "second", "pdf-v2")
            .await
            .status(),
        StatusCode::OK
    );

    let (text_count, extraction_count): (i64, i64) = sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM item_file_texts WHERE item_file_id = $1),
                (SELECT COUNT(*) FROM item_file_extractions WHERE item_file_id = $1)",
    )
    .bind(file_id)
    .fetch_one(&state.db)
    .await
    .unwrap();
    let (content, version): (String, String) = sqlx::query_as(
        "SELECT content, extraction_version FROM item_file_texts WHERE item_file_id = $1",
    )
    .bind(file_id)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!((text_count, extraction_count), (1, 2));
    assert_eq!((content.as_str(), version.as_str()), ("second", "pdf-v2"));
    cleanup(&state, item_id).await;
}

#[tokio::test]
#[ignore]
async fn failed_upsert_rolls_back_and_parallel_complete_has_one_winner() {
    let (state, router) = db_app().await;
    let (item_id, file_id, extraction_id, lease_token) = running_fixture(&state).await;
    let failed = post_complete(
        &router,
        extraction_id,
        lease_token,
        "rollback",
        &"v".repeat(65),
    )
    .await;
    assert_eq!(failed.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let state_value: String =
        sqlx::query_scalar("SELECT state::text FROM item_file_extractions WHERE id = $1")
            .bind(extraction_id)
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(state_value, "running");

    let (left, right) = tokio::join!(
        post_complete(&router, extraction_id, lease_token, "left", "pdf-v1"),
        post_complete(&router, extraction_id, lease_token, "right", "pdf-v1")
    );
    let statuses = [left.status(), right.status()];
    assert_eq!(statuses.iter().filter(|s| **s == StatusCode::OK).count(), 1);
    assert_eq!(
        statuses
            .iter()
            .filter(|s| **s == StatusCode::CONFLICT)
            .count(),
        1
    );
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM item_file_texts WHERE item_file_id = $1")
            .bind(file_id)
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(count, 1);
    cleanup(&state, item_id).await;
}
