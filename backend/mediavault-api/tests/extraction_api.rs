//! TASK-0010: Phase 2 文字抽出公開APIの横断統合テスト。

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use mediavault_api::AppState;
use mediavault_api::routes::build_router;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

static FILE_ROOT: OnceLock<PathBuf> = OnceLock::new();

fn file_root() -> &'static Path {
    FILE_ROOT
        .get_or_init(|| {
            let root =
                std::env::temp_dir().join(format!("mediavault-task-0010-{}", std::process::id()));
            std::fs::create_dir_all(root.join("files"))
                .expect("テスト用ファイルルートを作成できること");
            // この統合テストバイナリ内の全ケースで同一ルートを共有する。
            unsafe {
                std::env::set_var("LIBRARY_ROOT", &root);
                std::env::set_var("STORAGE_ROOT", &root);
                std::env::set_var("STORAGE_SUBDIR_FILES", "files");
            }
            root
        })
        .as_path()
}

async fn test_state() -> AppState {
    file_root();
    let database_url =
        std::env::var("DATABASE_URL").expect("TASK-0010統合テストにはDATABASE_URLが必要です");
    AppState {
        db: PgPool::connect(&database_url)
            .await
            .expect("テストDBへ接続できること"),
        internal_api_key: "task-0010-test-key".to_string(),
    }
}

async fn json_response(response: axum::response::Response) -> (StatusCode, Value) {
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("レスポンス本文を読み取れること");
    let body = serde_json::from_slice(&bytes).expect("レスポンスがJSONであること");
    (status, body)
}

async fn call(app: &axum::Router, method: &str, uri: String, body: Body) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();
    json_response(response).await
}

async fn setup_item_with_file(pool: &PgPool, file_type: &str, label: &str) -> (Uuid, Uuid) {
    let item_id: Uuid = sqlx::query_scalar(
        "INSERT INTO items (media_type, title, status, is_favorite, source)
         VALUES ('novel', 'TASK-0010 integration', 'not_started', false, 'manual')
         RETURNING id",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    let extension = if file_type == "video" { "mp4" } else { "pdf" };
    let path = file_root().join(format!("{item_id}-{label}.{extension}"));
    std::fs::write(&path, b"TASK-0010 fixture").unwrap();
    let file_id: Uuid = sqlx::query_scalar(
        "INSERT INTO item_files (item_id, path, label, file_type)
         VALUES ($1, $2, $3, $4::file_type) RETURNING id",
    )
    .bind(item_id)
    .bind(path.to_string_lossy().as_ref())
    .bind(label)
    .bind(file_type)
    .fetch_one(pool)
    .await
    .unwrap();
    (item_id, file_id)
}

async fn insert_text(pool: &PgPool, file_id: Uuid, content: &str, version: &str) {
    sqlx::query(
        "INSERT INTO item_file_texts
           (item_file_id, content, boundaries, extraction_version, extractor, extracted_at)
         VALUES ($1, $2, $3, $4, $5, CURRENT_TIMESTAMP)
         ON CONFLICT (item_file_id) DO UPDATE SET
           content = EXCLUDED.content, boundaries = EXCLUDED.boundaries,
           extraction_version = EXCLUDED.extraction_version",
    )
    .bind(file_id)
    .bind(content)
    .bind(json!([{"start": 0, "end": content.chars().count(), "label": "p.1"}]))
    .bind(version)
    .bind(json!({"method": "embedded_text"}))
    .execute(pool)
    .await
    .unwrap();
}

async fn cleanup(pool: &PgPool, item_id: Uuid) {
    sqlx::query("DELETE FROM items WHERE id = $1")
        .bind(item_id)
        .execute(pool)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore]
async fn extraction_request_status_and_text_form_one_consistent_flow() {
    let state = test_state().await;
    let app = build_router(state.clone());
    let (item_id, file_id) = setup_item_with_file(&state.db, "pdf", "flow").await;

    let (status, body) = call(
        &app,
        "GET",
        format!("/items/{item_id}/text?file_id={file_id}"),
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "TEXT_NOT_EXTRACTED");

    let (status, created) = call(
        &app,
        "POST",
        format!("/items/{item_id}/files/{file_id}/extraction"),
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["data"]["state"], "queued");

    let (status, queued) = call(
        &app,
        "GET",
        format!("/items/{item_id}/files/{file_id}/extraction"),
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(queued["data"]["id"], created["data"]["id"]);

    sqlx::query("UPDATE item_file_extractions SET state = 'succeeded' WHERE item_file_id = $1")
        .bind(file_id)
        .execute(&state.db)
        .await
        .unwrap();
    insert_text(&state.db, file_id, "抽出済み本文", "pdf-v1").await;

    let (status, finished) = call(
        &app,
        "GET",
        format!("/items/{item_id}/files/{file_id}/extraction"),
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(finished["data"]["state"], "succeeded");
    assert!(finished["data"].get("lease_token").is_none());

    let (status, text) = call(
        &app,
        "GET",
        format!("/items/{item_id}/text?file_id={file_id}"),
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(text["data"]["extraction_version"], "pdf-v1");
    assert_eq!(text["data"]["chunk"]["text"], "抽出済み本文");
    cleanup(&state.db, item_id).await;
}

#[tokio::test]
#[ignore]
async fn repeated_requests_converge_and_cancelled_job_can_be_recreated() {
    let state = test_state().await;
    let app = build_router(state.clone());
    let (item_id, file_id) = setup_item_with_file(&state.db, "pdf", "idempotent").await;
    let uri = format!("/items/{item_id}/files/{file_id}/extraction");

    let (first_status, first) = call(&app, "POST", uri.clone(), Body::empty()).await;
    let (second_status, second) = call(&app, "POST", uri.clone(), Body::empty()).await;
    let (third_status, third) = call(&app, "POST", uri.clone(), Body::empty()).await;
    assert_eq!(first_status, StatusCode::CREATED);
    assert_eq!(second_status, StatusCode::OK);
    assert_eq!(third_status, StatusCode::OK);
    assert_eq!(first["data"]["id"], second["data"]["id"]);
    assert_eq!(second["data"]["id"], third["data"]["id"]);

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM item_file_extractions
         WHERE item_file_id = $1 AND state IN ('queued', 'running', 'cancelling')",
    )
    .bind(file_id)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(count, 1);

    let cancel_uri = format!("{uri}/cancel");
    let (status, cancelled) = call(&app, "POST", cancel_uri.clone(), Body::empty()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cancelled["data"]["state"], "cancelled");
    let (status, error) = call(&app, "POST", cancel_uri, Body::empty()).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error["error"]["code"], "EXTRACTION_ALREADY_FINISHED");
    let (status, recreated) = call(&app, "POST", uri, Body::empty()).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_ne!(recreated["data"]["id"], first["data"]["id"]);
    cleanup(&state.db, item_id).await;
}

#[tokio::test]
#[ignore]
async fn cancelling_reextraction_preserves_the_previous_text() {
    let state = test_state().await;
    let app = build_router(state.clone());
    let (item_id, file_id) = setup_item_with_file(&state.db, "pdf", "preserve").await;
    insert_text(&state.db, file_id, "以前の抽出結果", "pdf-v1").await;

    let extraction_uri = format!("/items/{item_id}/files/{file_id}/extraction");
    let (status, _) = call(&app, "POST", extraction_uri.clone(), Body::empty()).await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, _) = call(
        &app,
        "POST",
        format!("{extraction_uri}/cancel"),
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, text) = call(
        &app,
        "GET",
        format!("/items/{item_id}/text?file_id={file_id}"),
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(text["data"]["extraction_version"], "pdf-v1");
    assert_eq!(text["data"]["chunk"]["text"], "以前の抽出結果");
    cleanup(&state.db, item_id).await;
}

#[tokio::test]
#[ignore]
async fn ambiguous_text_response_provides_candidates_for_a_successful_retry() {
    let state = test_state().await;
    let app = build_router(state.clone());
    let (item_id, first_file) = setup_item_with_file(&state.db, "pdf", "main").await;
    let second_path = file_root().join(format!("{item_id}-appendix.pdf"));
    std::fs::write(&second_path, b"appendix").unwrap();
    let second_file: Uuid = sqlx::query_scalar(
        "INSERT INTO item_files (item_id, path, label, file_type)
         VALUES ($1, $2, 'appendix', 'pdf') RETURNING id",
    )
    .bind(item_id)
    .bind(second_path.to_string_lossy().as_ref())
    .fetch_one(&state.db)
    .await
    .unwrap();
    insert_text(&state.db, first_file, "main text", "pdf-v1").await;
    insert_text(&state.db, second_file, "appendix text", "pdf-v1").await;

    let (status, ambiguous) =
        call(&app, "GET", format!("/items/{item_id}/text"), Body::empty()).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(ambiguous["error"]["code"], "AMBIGUOUS_FILE");
    assert_eq!(
        ambiguous["error"]["candidates"].as_array().unwrap().len(),
        2
    );
    let selected = ambiguous["error"]["candidates"][0]["file_id"]
        .as_str()
        .unwrap();

    let (status, resolved) = call(
        &app,
        "GET",
        format!("/items/{item_id}/text?file_id={selected}"),
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resolved["data"]["file_id"], selected);
    cleanup(&state.db, item_id).await;
}

#[tokio::test]
#[ignore]
async fn public_error_codes_have_reachable_paths_and_expected_statuses() {
    let state = test_state().await;
    let app = build_router(state.clone());
    let (item_id, pdf_file) = setup_item_with_file(&state.db, "pdf", "errors-pdf").await;

    let (status, body) = call(
        &app,
        "GET",
        format!("/items/{item_id}/files/{pdf_file}/extraction"),
        Body::empty(),
    )
    .await;
    assert_eq!(
        (status, body["error"]["code"].as_str()),
        (StatusCode::NOT_FOUND, Some("EXTRACTION_NOT_FOUND"))
    );

    let (_, video_file) = setup_item_with_file(&state.db, "video", "errors-video").await;
    let video_item: Uuid = sqlx::query_scalar("SELECT item_id FROM item_files WHERE id = $1")
        .bind(video_file)
        .fetch_one(&state.db)
        .await
        .unwrap();
    let (status, body) = call(
        &app,
        "POST",
        format!("/items/{video_item}/files/{video_file}/extraction"),
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "UNSUPPORTED_FILE_TYPE");

    let (status, body) = call(
        &app,
        "GET",
        format!("/items/{item_id}/text?file_id={pdf_file}"),
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "TEXT_NOT_EXTRACTED");

    let empty_item: Uuid = sqlx::query_scalar(
        "INSERT INTO items (media_type, title, status, is_favorite, source)
         VALUES ('novel', 'TASK-0010 no files', 'not_started', false, 'manual') RETURNING id",
    )
    .fetch_one(&state.db)
    .await
    .unwrap();
    let (status, body) = call(
        &app,
        "GET",
        format!("/items/{empty_item}/text"),
        Body::empty(),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "FILE_NOT_FOUND");
    cleanup(&state.db, item_id).await;
    cleanup(&state.db, video_item).await;
    cleanup(&state.db, empty_item).await;
}

#[tokio::test]
#[ignore]
async fn registering_or_uploading_files_does_not_queue_extraction() {
    let state = test_state().await;
    let app = build_router(state.clone());
    let item_id: Uuid = sqlx::query_scalar(
        "INSERT INTO items (media_type, title, status, is_favorite, source)
         VALUES ('novel', 'TASK-0010 no auto queue', 'not_started', false, 'manual') RETURNING id",
    )
    .fetch_one(&state.db)
    .await
    .unwrap();
    let linked_path = file_root().join(format!("{item_id}-linked.pdf"));
    std::fs::write(&linked_path, b"linked pdf").unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::post(format!("/items/{item_id}/files"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"path": linked_path.to_string_lossy()}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let boundary = "task0010Boundary";
    let multipart = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"upload.pdf\"\r\nContent-Type: application/pdf\r\n\r\n%PDF-1.4 fixture\r\n--{boundary}--\r\n"
    );
    let response = app
        .clone()
        .oneshot(
            Request::post(format!("/items/{item_id}/files/upload"))
                .header(
                    "content-type",
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(multipart))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let extraction_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM item_file_extractions e
         INNER JOIN item_files f ON f.id = e.item_file_id WHERE f.item_id = $1",
    )
    .bind(item_id)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(extraction_count, 0);
    cleanup(&state.db, item_id).await;
}
