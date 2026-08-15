//! worker専用の文字抽出内部APIハンドラ。

use axum::Json;
use axum::extract::{Path, State};
use uuid::Uuid;

use crate::AppState;
use crate::models::item_extraction::{
    CancelledRequest, ClaimRequest, ClaimResponse, CompleteRequest, ExtractionResponse,
    FailRequest, HeartbeatRequest, HeartbeatResponse, cancel_requested, validate_claim_request,
    validate_complete_request, validate_fail_request, validate_heartbeat_request,
};
use crate::models::response::{ApiError, ApiErrorCode, ApiOk};
use crate::repositories::item_extraction_repository;
use crate::services::file_ref::{self, FileRefConfig};

/// 🔵 Intent: 実行可能な抽出を排他的に取得し、worker用leaseと安全なFileRefを返す。
pub async fn claim_handler(
    State(state): State<AppState>,
    Json(request): Json<ClaimRequest>,
) -> Result<ApiOk<Option<ClaimResponse>>, ApiError> {
    validate_claim_request(&request)?;
    item_extraction_repository::sweep_exhausted_leases(&state.db).await?;

    loop {
        let Some(extraction) = item_extraction_repository::claim_next(
            &state.db,
            request.worker_id.trim(),
            request.lease_seconds,
        )
        .await?
        else {
            return Ok(ApiOk::new(None));
        };

        let Some(file) =
            item_extraction_repository::find_claimed_file(&state.db, extraction.item_file_id)
                .await?
        else {
            item_extraction_repository::fail_unavailable_claim(&state.db, extraction.id).await?;
            continue;
        };

        let path = file.path;
        let resolved = tokio::task::spawn_blocking(move || {
            file_ref::resolve(&path, &FileRefConfig::from_env())
        })
        .await
        .map_err(|error| {
            tracing::error!(%error, "claim file_ref resolution task failed");
            ApiError::new(ApiErrorCode::InternalError, "抽出処理の取得に失敗しました")
        })?;

        let resolved = match resolved {
            Ok(resolved) => resolved,
            Err(_) => {
                item_extraction_repository::fail_unavailable_claim(&state.db, extraction.id)
                    .await?;
                continue;
            }
        };
        let lease_token = extraction.lease_token.ok_or_else(invalid_claim_state)?;
        let lease_expires_at = extraction
            .lease_expires_at
            .ok_or_else(invalid_claim_state)?;

        return Ok(ApiOk::new(Some(ClaimResponse {
            extraction_id: extraction.id,
            item_file_id: extraction.item_file_id,
            item_id: file.item_id,
            file_type: file.file_type,
            size_bytes: resolved.size_bytes,
            attempts: extraction.attempts,
            lease_token,
            lease_expires_at,
            file_ref: resolved.file_ref,
        })));
    }
}

/// 🔵 Intent: lease延長・進捗更新・キャンセル通知を認証済みworkerの1リクエストで処理する。
pub async fn heartbeat_handler(
    State(state): State<AppState>,
    Path(extraction_id): Path<Uuid>,
    Json(request): Json<HeartbeatRequest>,
) -> Result<ApiOk<HeartbeatResponse>, ApiError> {
    validate_heartbeat_request(&request)?;
    // 🟡 既定値300秒はworker設定の設計上の暫定値と揃え、省略時もleaseを延長する。
    let lease_seconds = request.lease_seconds.unwrap_or(300);
    let updated = item_extraction_repository::heartbeat(
        &state.db,
        extraction_id,
        request.lease_token,
        lease_seconds,
        request.progress_current,
        request.progress_total,
    )
    .await?;

    Ok(ApiOk::new(HeartbeatResponse {
        state: updated.state,
        cancel_requested: cancel_requested(updated.state),
        lease_expires_at: updated.lease_expires_at,
    }))
}

/// 🔵 Intent: 内容検証をロック取得前に済ませ、結果保存と成功遷移を単一トランザクションへ委譲する。
pub async fn complete_handler(
    State(state): State<AppState>,
    Path(extraction_id): Path<Uuid>,
    Json(request): Json<CompleteRequest>,
) -> Result<ApiOk<ExtractionResponse>, ApiError> {
    validate_complete_request(&request)?;
    let completed = item_extraction_repository::complete_extraction(
        &state.db,
        extraction_id,
        request.lease_token,
        &request,
    )
    .await?;
    Ok(ApiOk::new(completed.into()))
}

/// 🔵 Intent: workerの構造化失敗を検証し、有効なleaseに限って再投入または終端化する。
pub async fn fail_handler(
    State(state): State<AppState>,
    Path(extraction_id): Path<Uuid>,
    Json(request): Json<FailRequest>,
) -> Result<ApiOk<ExtractionResponse>, ApiError> {
    validate_fail_request(&request)?;
    let failed = item_extraction_repository::fail_extraction(
        &state.db,
        extraction_id,
        request.lease_token,
        &request.error,
    )
    .await?;
    Ok(ApiOk::new(failed.into()))
}

/// 🔵 Intent: workerのキャンセル完了確認を受け、cancelling状態だけを終端化する。
pub async fn cancelled_handler(
    State(state): State<AppState>,
    Path(extraction_id): Path<Uuid>,
    Json(request): Json<CancelledRequest>,
) -> Result<ApiOk<ExtractionResponse>, ApiError> {
    let cancelled = item_extraction_repository::cancel_extraction(
        &state.db,
        extraction_id,
        request.lease_token,
    )
    .await?;
    Ok(ApiOk::new(cancelled.into()))
}

fn invalid_claim_state() -> ApiError {
    ApiError::new(
        ApiErrorCode::InternalError,
        "抽出処理のlease発行に失敗しました",
    )
}
