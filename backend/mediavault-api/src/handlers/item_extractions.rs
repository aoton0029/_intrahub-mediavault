//! item_file_extractions 公開APIハンドラ。

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::AppState;
use crate::models::item::parse_item_id;
use crate::models::item_extraction::{ExtractionResponse, is_extractable};
use crate::models::item_file::{ItemFile, parse_file_id};
use crate::models::response::{ApiError, ApiErrorCode, ApiOk};
use crate::repositories::item_extraction_repository::{self, CreateOutcome};
use crate::repositories::{item_file_repository, item_repository};
use crate::services::file_ref::{self, FileRefConfig};

/// 🔵 Intent: TASK-0007所定の検証順序を守り、同一ファイルのactive抽出を冪等に作成する。
pub async fn request_extraction_handler(
    State(state): State<AppState>,
    Path((item_id, file_id)): Path<(String, String)>,
) -> Result<axum::response::Response, ApiError> {
    // 1. DBアクセスより先に両方のUUID形式を検証する。
    let item_id = parse_item_id(&item_id)?;
    let file_id = parse_file_id(&file_id)?;

    // 2. 既存item_filesハンドラと同様、Item不存在とFile不存在を区別する。
    let file = find_owned_file(&state, item_id, file_id).await?;

    // 3. 実体確認より先に非対応形式を拒否し、再試行不要であることを明示する。
    if !is_extractable(file.file_type) {
        return Err(ApiError::new(
            ApiErrorCode::UnsupportedFileType,
            "このファイル形式は文字抽出に対応していません",
        ));
    }

    // 4. ファイルシステムの同期I/Oでasync executorを塞がないようblocking poolへ分離する。
    let path = file.path;
    tokio::task::spawn_blocking(move || file_ref::resolve(&path, &FileRefConfig::from_env()))
        .await
        .map_err(|error| {
            tracing::error!(%error, "file_ref resolution task failed");
            ApiError::new(
                ApiErrorCode::UnprocessableEntity,
                "ファイルの実体を確認できません",
            )
        })??;

    // 5. 部分UNIQUE制約を使うrepositoryに並列リクエストの収束を委ねる。
    match item_extraction_repository::create_extraction(&state.db, file_id).await? {
        CreateOutcome::Created(row) => Ok((
            StatusCode::CREATED,
            Json(ApiOk::new(ExtractionResponse::from(row))),
        )
            .into_response()),
        CreateOutcome::Existing(row) => {
            Ok(Json(ApiOk::new(ExtractionResponse::from(row))).into_response())
        }
    }
}

/// 🔵 Intent: ファイル存在・帰属の検証順序を3つの公開APIで統一する。
async fn find_owned_file(
    state: &AppState,
    item_id: uuid::Uuid,
    file_id: uuid::Uuid,
) -> Result<ItemFile, ApiError> {
    if item_repository::get_item_by_id(&state.db, item_id)
        .await?
        .is_none()
    {
        return Err(ApiError::new(
            ApiErrorCode::ItemNotFound,
            "指定されたアイテムが見つかりません",
        ));
    }

    item_file_repository::find_item_file(&state.db, item_id, file_id)
        .await?
        .ok_or_else(|| {
            ApiError::new(
                ApiErrorCode::FileNotFound,
                "指定されたファイルが見つかりません",
            )
        })
}

/// 🔵 Intent: 同一ファイルの抽出履歴から最新状態だけを公開DTOとして返す。
pub async fn get_extraction_handler(
    State(state): State<AppState>,
    Path((item_id, file_id)): Path<(String, String)>,
) -> Result<ApiOk<ExtractionResponse>, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let file_id = parse_file_id(&file_id)?;
    find_owned_file(&state, item_id, file_id).await?;

    let extraction = item_extraction_repository::find_latest_by_file(&state.db, file_id).await?;
    Ok(ApiOk::new(ExtractionResponse::from(extraction)))
}

/// 🔵 Intent: queuedは即時停止、runningはworker確認待ちのcancellingへ遷移させる。
pub async fn cancel_extraction_handler(
    State(state): State<AppState>,
    Path((item_id, file_id)): Path<(String, String)>,
) -> Result<ApiOk<ExtractionResponse>, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let file_id = parse_file_id(&file_id)?;
    find_owned_file(&state, item_id, file_id).await?;

    let latest = item_extraction_repository::find_latest_by_file(&state.db, file_id).await?;
    let extraction =
        item_extraction_repository::request_cancel(&state.db, latest.id, file_id).await?;
    Ok(ApiOk::new(ExtractionResponse::from(extraction)))
}
