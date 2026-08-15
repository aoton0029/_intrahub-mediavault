//! TASK-0015: PRD 8.8 の競合制御・障害復旧受け入れテスト。

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use mediavault_api::AppState;
use mediavault_api::routes::{build_router, internal::build_internal_router};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

const TEST_KEY: &str = "task-0015-key";
static STORAGE_ROOT: OnceLock<PathBuf> = OnceLock::new();

fn storage_root() -> &'static Path {
    STORAGE_ROOT
        .get_or_init(|| {
            let root =
                std::env::temp_dir().join(format!("mediavault-task-0015-{}", std::process::id()));
            std::fs::create_dir_all(root.join("files")).unwrap();
            unsafe {
                std::env::set_var("STORAGE_ROOT", &root);
                std::env::set_var("STORAGE_SUBDIR_FILES", "files");
                std::env::set_var("LIBRARY_ROOT", &root);
            }
            root
        })
        .as_path()
}

async fn test_app() -> (AppState, Router) {
    storage_root();
    let db = PgPool::connect(
        &std::env::var("DATABASE_URL").expect("TASK-0015統合テストにはDATABASE_URLが必要です"),
    )
    .await
    .unwrap();
    let state = AppState {
        db,
        internal_api_key: TEST_KEY.to_string(),
    };
    let app = Router::new().nest(
        "/api/v1",
        build_router(state.clone()).merge(build_internal_router(state.clone())),
    );
    (state, app)
}

async fn request(
    app: &Router,
    method: &str,
    uri: impl AsRef<str>,
    body: Value,
    authorization: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri.as_ref())
        .header("content-type", "application/json");
    if let Some(key) = authorization {
        builder = builder.header("authorization", format!("Bearer {key}"));
    }
    let response = app
        .clone()
        .oneshot(builder.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, value)
}

async fn fixture(pool: &PgPool, label: &str) -> (Uuid, Uuid) {
    let item_id: Uuid = sqlx::query_scalar(
        "INSERT INTO items (media_type, title, status, is_favorite, source)
         VALUES ('novel', $1, 'not_started', false, 'manual') RETURNING id",
    )
    .bind(format!("TASK-0015 {label}"))
    .fetch_one(pool)
    .await
    .unwrap();
    let relative_path = format!("task-0015-{item_id}.pdf");
    std::fs::write(storage_root().join("files").join(&relative_path), b"pdf").unwrap();
    let file_id = sqlx::query_scalar(
        "INSERT INTO item_files (item_id, path, label, file_type)
         VALUES ($1, $2, $3, 'pdf') RETURNING id",
    )
    .bind(item_id)
    .bind(relative_path)
    .bind(label)
    .fetch_one(pool)
    .await
    .unwrap();
    (item_id, file_id)
}

async fn queue(app: &Router, item_id: Uuid, file_id: Uuid) -> (StatusCode, Value) {
    request(
        app,
        "POST",
        format!("/api/v1/items/{item_id}/files/{file_id}/extraction"),
        json!({}),
        None,
    )
    .await
}

async fn expire_lease(pool: &PgPool, extraction_id: Uuid) {
    sqlx::query(
        "UPDATE item_file_extractions
         SET lease_expires_at = CURRENT_TIMESTAMP - INTERVAL '1 hour' WHERE id = $1",
    )
    .bind(extraction_id)
    .execute(pool)
    .await
    .unwrap();
}

async fn cleanup(pool: &PgPool, item_ids: &[Uuid]) {
    for item_id in item_ids {
        sqlx::query("DELETE FROM items WHERE id = $1")
            .bind(item_id)
            .execute(pool)
            .await
            .unwrap();
    }
}

#[derive(Clone, Debug)]
struct FakeWorker {
    id: String,
    lease_token: Option<Uuid>,
    extraction_id: Option<Uuid>,
}

impl FakeWorker {
    fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            lease_token: None,
            extraction_id: None,
        }
    }

    async fn claim(&mut self, app: &Router) -> Option<Value> {
        let (status, body) = request(
            app,
            "POST",
            "/api/v1/internal/extractions/claim",
            json!({"worker_id": self.id, "lease_seconds": 300}),
            Some(TEST_KEY),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        if body["data"].is_null() {
            return None;
        }
        let data = body["data"].clone();
        self.extraction_id =
            Some(Uuid::parse_str(data["extraction_id"].as_str().unwrap()).unwrap());
        self.lease_token = Some(Uuid::parse_str(data["lease_token"].as_str().unwrap()).unwrap());
        Some(data)
    }

    async fn action(&self, app: &Router, action: &str, extra: Value) -> (StatusCode, Value) {
        let mut body = extra.as_object().cloned().unwrap_or_default();
        body.insert("lease_token".into(), json!(self.lease_token.unwrap()));
        request(
            app,
            "POST",
            format!(
                "/api/v1/internal/extractions/{}/{}",
                self.extraction_id.unwrap(),
                action
            ),
            Value::Object(body),
            Some(TEST_KEY),
        )
        .await
    }

    async fn heartbeat(&self, app: &Router, progress: (i32, i32)) -> (StatusCode, Value) {
        self.action(
            app,
            "heartbeat",
            json!({"progress_current": progress.0, "progress_total": progress.1, "lease_seconds": 300}),
        )
        .await
    }

    async fn complete(&self, app: &Router, content: &str) -> (StatusCode, Value) {
        self.action(
            app,
            "complete",
            json!({
                "content": content,
                "boundaries": [{"start": 0, "end": content.chars().count(), "label": "p.1"}],
                "extraction_version": "pdf-v1",
                "extracted_at": "2026-08-15T12:00:00",
                "extractor": {"method": "embedded_text", "embedded_text_pages": 1, "ocr_pages": 0, "ocr": null}
            }),
        )
        .await
    }

    async fn fail(&self, app: &Router, retryable: bool) -> (StatusCode, Value) {
        self.action(
            app,
            "fail",
            json!({"error": {"kind": "ocr_failed", "message": "worker failure", "retryable": retryable}}),
        )
        .await
    }

    async fn cancelled(&self, app: &Router) -> (StatusCode, Value) {
        self.action(app, "cancelled", json!({})).await
    }
}

// PRD 8.8-1: 同じ対象への未完了ジョブは並列要求でも1件に収束する。
#[tokio::test]
#[ignore]
async fn acceptance_1_parallel_requests_are_idempotent() {
    let (state, app) = test_app().await;
    let (item_id, file_id) = fixture(&state.db, "idempotent").await;
    let futures = (0..10).map(|_| queue(&app, item_id, file_id));
    let results = futures::future::join_all(futures).await;
    assert_eq!(
        results
            .iter()
            .filter(|(s, _)| *s == StatusCode::CREATED)
            .count(),
        1
    );
    assert_eq!(
        results.iter().filter(|(s, _)| *s == StatusCode::OK).count(),
        9
    );
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM item_file_extractions
         WHERE item_file_id = $1 AND state IN ('queued', 'running', 'cancelling')",
    )
    .bind(file_id)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(count, 1);
    cleanup(&state.db, &[item_id]).await;
}

// PRD 8.8-2: SKIP LOCKED により1ジョブを複数workerへ払い出さない。
#[tokio::test]
#[ignore]
async fn acceptance_2_parallel_workers_claim_each_job_once() {
    let (state, app) = test_app().await;
    let mut items = Vec::new();
    for label in ["claim-a", "claim-b", "claim-c"] {
        let (item, file) = fixture(&state.db, label).await;
        queue(&app, item, file).await;
        items.push(item);
    }
    let futures = (0..5).map(|index| {
        let app = app.clone();
        async move {
            let mut worker = FakeWorker::new(format!("parallel-{index}"));
            worker.claim(&app).await
        }
    });
    let claims = futures::future::join_all(futures).await;
    let ids: Vec<_> = claims
        .iter()
        .flatten()
        .map(|data| data["extraction_id"].as_str().unwrap())
        .collect();
    let unique: std::collections::HashSet<_> = ids.iter().copied().collect();
    assert_eq!((ids.len(), unique.len()), (3, 3));
    cleanup(&state.db, &items).await;
}

// PRD 8.8-3: lease切れは新token・attempts増加で再取得できる。
#[tokio::test]
#[ignore]
async fn acceptance_3_expired_lease_is_safely_reclaimed() {
    let (state, app) = test_app().await;
    let (item, file) = fixture(&state.db, "reclaim").await;
    queue(&app, item, file).await;
    let mut old = FakeWorker::new("worker-old");
    let first = old.claim(&app).await.unwrap();
    expire_lease(&state.db, old.extraction_id.unwrap()).await;
    let mut new = FakeWorker::new("worker-new");
    let second = new.claim(&app).await.unwrap();
    assert_eq!(first["extraction_id"], second["extraction_id"]);
    assert_ne!(first["lease_token"], second["lease_token"]);
    assert_eq!(second["attempts"], 2);
    let claimed_by: String =
        sqlx::query_scalar("SELECT claimed_by FROM item_file_extractions WHERE id = $1")
            .bind(new.extraction_id.unwrap())
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(claimed_by, "worker-new");
    cleanup(&state.db, &[item]).await;
}

// PRD 8.8-4: reclaim後の旧worker操作は全て INVALID_LEASE_TOKEN。
#[tokio::test]
#[ignore]
async fn acceptance_4_stale_worker_is_rejected_after_reclaim() {
    let (state, app) = test_app().await;
    let (item, file) = fixture(&state.db, "stale").await;
    queue(&app, item, file).await;
    let mut old = FakeWorker::new("stale-old");
    old.claim(&app).await;
    expire_lease(&state.db, old.extraction_id.unwrap()).await;
    let mut winner = FakeWorker::new("stale-winner");
    winner.claim(&app).await;
    assert_eq!(winner.complete(&app, "winner").await.0, StatusCode::OK);
    for result in [
        old.complete(&app, "stale").await,
        old.heartbeat(&app, (1, 1)).await,
        old.fail(&app, false).await,
        old.cancelled(&app).await,
    ] {
        assert_eq!(result.0, StatusCode::CONFLICT);
        assert_eq!(result.1["error"]["code"], "INVALID_LEASE_TOKEN");
    }
    let content: String =
        sqlx::query_scalar("SELECT content FROM item_file_texts WHERE item_file_id = $1")
            .bind(file)
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(content, "winner");
    cleanup(&state.db, &[item]).await;
}

// PRD 8.8-5: cancel要求はheartbeatで通知され、worker確認で終端化する。
#[tokio::test]
#[ignore]
async fn acceptance_5_cancellation_reaches_worker_and_becomes_terminal() {
    let (state, app) = test_app().await;
    let (item, file) = fixture(&state.db, "cancel").await;
    queue(&app, item, file).await;
    let mut worker = FakeWorker::new("cancel-worker");
    worker.claim(&app).await;
    let (status, _) = request(
        &app,
        "POST",
        format!("/api/v1/items/{item}/files/{file}/extraction/cancel"),
        json!({}),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (_, heartbeat) = worker.heartbeat(&app, (1, 2)).await;
    assert_eq!(heartbeat["data"]["cancel_requested"], true);
    assert_eq!(
        worker.complete(&app, "too late").await.0,
        StatusCode::CONFLICT
    );
    assert_eq!(worker.cancelled(&app).await.0, StatusCode::OK);
    let (state_value, text_count): (String, i64) = sqlx::query_as(
        "SELECT state::text, (SELECT COUNT(*) FROM item_file_texts WHERE item_file_id = $2)
         FROM item_file_extractions WHERE id = $1",
    )
    .bind(worker.extraction_id.unwrap())
    .bind(file)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!((state_value.as_str(), text_count), ("cancelled", 0));
    cleanup(&state.db, &[item]).await;
}

// PRD 8.8-6/7: completeは本文と成功状態を原子的に保存しText APIへ公開する。
#[tokio::test]
#[ignore]
async fn acceptance_6_and_7_complete_is_atomic_and_text_api_returns_metadata() {
    let (state, app) = test_app().await;
    let (item, file) = fixture(&state.db, "complete").await;
    queue(&app, item, file).await;
    let mut worker = FakeWorker::new("complete-worker");
    worker.claim(&app).await;
    assert_eq!(
        worker.complete(&app, "extracted text").await.0,
        StatusCode::OK
    );
    let (job_state, text_count): (String, i64) = sqlx::query_as(
        "SELECT state::text, (SELECT COUNT(*) FROM item_file_texts WHERE item_file_id = $2)
         FROM item_file_extractions WHERE id = $1",
    )
    .bind(worker.extraction_id.unwrap())
    .bind(file)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!((job_state.as_str(), text_count), ("succeeded", 1));
    let (status, text) = request(
        &app,
        "GET",
        format!("/api/v1/items/{item}/text?file_id={file}"),
        json!({}),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(text["data"]["extraction_version"], "pdf-v1");
    assert_eq!(text["data"]["chunk"]["index"], 0);
    assert_eq!(text["data"]["chunk"]["total_chunks"], 1);
    assert_eq!(text["data"]["chunk"]["label"], "p.1");

    // extraction_version のDB制約違反を起こし、本文UPSERTとstate更新が共に戻ることを確認する。
    let (rollback_item, rollback_file) = fixture(&state.db, "rollback").await;
    queue(&app, rollback_item, rollback_file).await;
    let mut rollback_worker = FakeWorker::new("rollback-worker");
    rollback_worker.claim(&app).await;
    let (status, _) = rollback_worker
        .action(
            &app,
            "complete",
            json!({
                "content": "rollback",
                "boundaries": [{"start": 0, "end": 8, "label": "p.1"}],
                "extraction_version": "v".repeat(65),
                "extracted_at": "2026-08-15T12:00:00",
                "extractor": {"method": "embedded_text", "embedded_text_pages": 1, "ocr_pages": 0, "ocr": null}
            }),
        )
        .await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    let (rollback_state, rollback_texts): (String, i64) = sqlx::query_as(
        "SELECT state::text, (SELECT COUNT(*) FROM item_file_texts WHERE item_file_id = $2)
         FROM item_file_extractions WHERE id = $1",
    )
    .bind(rollback_worker.extraction_id.unwrap())
    .bind(rollback_file)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!((rollback_state.as_str(), rollback_texts), ("running", 0));
    cleanup(&state.db, &[item, rollback_item]).await;
}

// PRD 8.8-8: 未抽出・ファイル不在・複数候補を別のエラーとして返す。
#[tokio::test]
#[ignore]
async fn acceptance_8_text_api_distinguishes_error_conditions() {
    let (state, app) = test_app().await;
    let (unextracted_item, unextracted_file) = fixture(&state.db, "unextracted").await;
    let (status, body) = request(
        &app,
        "GET",
        format!("/api/v1/items/{unextracted_item}/text?file_id={unextracted_file}"),
        json!({}),
        None,
    )
    .await;
    assert_eq!(
        (status, body["error"]["code"].as_str()),
        (StatusCode::UNPROCESSABLE_ENTITY, Some("TEXT_NOT_EXTRACTED"))
    );

    let no_file: Uuid = sqlx::query_scalar(
        "INSERT INTO items (media_type, title, status, is_favorite, source)
         VALUES ('novel', 'TASK-0015 no file', 'not_started', false, 'manual') RETURNING id",
    )
    .fetch_one(&state.db)
    .await
    .unwrap();
    let (status, body) = request(
        &app,
        "GET",
        format!("/api/v1/items/{no_file}/text"),
        json!({}),
        None,
    )
    .await;
    assert_eq!(
        (status, body["error"]["code"].as_str()),
        (StatusCode::NOT_FOUND, Some("FILE_NOT_FOUND"))
    );

    let (ambiguous_item, first_file) = fixture(&state.db, "ambiguous-a").await;
    let second_file: Uuid = sqlx::query_scalar(
        "INSERT INTO item_files (item_id, path, label, file_type)
         VALUES ($1, $2, 'ambiguous-b', 'pdf') RETURNING id",
    )
    .bind(ambiguous_item)
    .bind(format!("task-0015-{ambiguous_item}-b.pdf"))
    .fetch_one(&state.db)
    .await
    .unwrap();
    for file in [first_file, second_file] {
        sqlx::query(
            "INSERT INTO item_file_texts
             (item_file_id, content, boundaries, extraction_version, extractor, extracted_at)
             VALUES ($1, 'text', '[{\"start\":0,\"end\":4,\"label\":\"p.1\"}]', 'pdf-v1',
                     '{\"method\":\"embedded_text\",\"embedded_text_pages\":1,\"ocr_pages\":0,\"ocr\":null}', CURRENT_TIMESTAMP)",
        )
        .bind(file)
        .execute(&state.db)
        .await
        .unwrap();
    }
    let (status, body) = request(
        &app,
        "GET",
        format!("/api/v1/items/{ambiguous_item}/text"),
        json!({}),
        None,
    )
    .await;
    assert_eq!(
        (status, body["error"]["code"].as_str()),
        (StatusCode::CONFLICT, Some("AMBIGUOUS_FILE"))
    );
    cleanup(&state.db, &[unextracted_item, no_file, ambiguous_item]).await;
}

// PRD 8.8-9: 全worker APIは認証境界内にあり、公開aliasを持たない。
#[tokio::test]
#[ignore]
async fn acceptance_9_internal_worker_api_requires_authentication() {
    let (_, app) = test_app().await;
    let id = Uuid::new_v4();
    let token = Uuid::new_v4();
    let endpoints = [
        (
            "/api/v1/internal/extractions/claim".to_string(),
            json!({"worker_id":"x","lease_seconds":300}),
        ),
        (
            format!("/api/v1/internal/extractions/{id}/heartbeat"),
            json!({"lease_token":token}),
        ),
        (
            format!("/api/v1/internal/extractions/{id}/complete"),
            json!({"lease_token":token}),
        ),
        (
            format!("/api/v1/internal/extractions/{id}/fail"),
            json!({"lease_token":token}),
        ),
        (
            format!("/api/v1/internal/extractions/{id}/cancelled"),
            json!({"lease_token":token}),
        ),
    ];
    for (uri, body) in endpoints {
        assert_eq!(
            request(&app, "POST", &uri, body.clone(), None).await.0,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            request(&app, "POST", &uri, body, Some("wrong-key")).await.0,
            StatusCode::UNAUTHORIZED
        );
    }
    assert_eq!(
        request(
            &app,
            "POST",
            "/api/v1/extractions/claim",
            json!({}),
            Some(TEST_KEY)
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
}

// EDGE-008: heartbeat途絶後も別workerが回収して完遂できる。
#[tokio::test]
#[ignore]
async fn heartbeat_loss_recovers_without_stale_artifacts() {
    let (state, app) = test_app().await;
    let (item, file) = fixture(&state.db, "heartbeat-loss").await;
    queue(&app, item, file).await;
    let mut abandoned = FakeWorker::new("abandoned");
    abandoned.claim(&app).await;
    expire_lease(&state.db, abandoned.extraction_id.unwrap()).await;
    let mut recovery = FakeWorker::new("recovery");
    recovery.claim(&app).await;
    assert_eq!(recovery.complete(&app, "recovered").await.0, StatusCode::OK);
    let row: (String, String) = sqlx::query_as(
        "SELECT e.state::text, t.content FROM item_file_extractions e
         JOIN item_file_texts t ON t.item_file_id = e.item_file_id WHERE e.id = $1",
    )
    .bind(recovery.extraction_id.unwrap())
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!((row.0.as_str(), row.1.as_str()), ("succeeded", "recovered"));
    cleanup(&state.db, &[item]).await;
}

// REQ-111: 上限到達済みの期限切れrunning行はclaim時にfailedへ掃除される。
#[tokio::test]
#[ignore]
async fn exhausted_expired_job_is_swept_to_failed() {
    let (state, app) = test_app().await;
    let (item, file) = fixture(&state.db, "exhausted").await;
    queue(&app, item, file).await;
    let mut worker = FakeWorker::new("exhausted-worker");
    worker.claim(&app).await;
    sqlx::query(
        "UPDATE item_file_extractions SET attempts = max_attempts,
         lease_expires_at = CURRENT_TIMESTAMP - INTERVAL '1 hour' WHERE id = $1",
    )
    .bind(worker.extraction_id.unwrap())
    .execute(&state.db)
    .await
    .unwrap();
    let mut sweeper = FakeWorker::new("sweeper");
    assert!(sweeper.claim(&app).await.is_none());
    let (state_value, error): (String, Value) =
        sqlx::query_as("SELECT state::text, error FROM item_file_extractions WHERE id = $1")
            .bind(worker.extraction_id.unwrap())
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(state_value, "failed");
    assert_eq!(error["kind"], "lease_expired");
    cleanup(&state.db, &[item]).await;
}
