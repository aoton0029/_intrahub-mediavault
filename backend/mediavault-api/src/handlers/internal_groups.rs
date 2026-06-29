//! `/internal/items/:id/groups` ハンドラ（巡回バッチ向けupsert）
//!
//! TASK-0029: 内部REST APIルート群実装（/internal/items等）

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::AppState;
use crate::models::item::{deserialize_request, parse_item_id};
use crate::models::item_group::{CreateItemGroupRequest, ItemGroup};
use crate::models::response::ApiError;
use crate::repositories::item_group_repository;

/// 【機能概要】: `POST /internal/items/:id/groups` ハンドラ。巡回バッチからの話数同期用に、
/// 同一(item_id, group_type, number)のグループが既に存在する場合は更新、なければ新規作成する
/// （`handlers::item_groups::create_item_group_handler`のupsert版）
/// 🔵 信頼性レベル: TASK-0029.md「実装詳細4」・テストケース6（TC-018-06）に直接対応
pub async fn upsert_item_group_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&id)?;
    let request: CreateItemGroupRequest = deserialize_request(body)?;

    let group = item_group_repository::upsert_item_group(&state.db, item_id, request).await?;

    Ok(created_response(group))
}

fn created_response(group: ItemGroup) -> axum::response::Response {
    (
        StatusCode::CREATED,
        Json(crate::models::response::ApiOk::new(group)),
    )
        .into_response()
}
