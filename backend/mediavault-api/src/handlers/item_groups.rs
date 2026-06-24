//! item_groups（シーズン/巻/章）ハンドラ
//!
//! TASK-0018: item_groups CRUD実装（handlers/item_relations.rsと対称な構造）

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::models::item::{deserialize_request, parse_item_id};
use crate::models::item_group::{CreateItemGroupRequest, ItemGroup};
use crate::models::response::{ApiError, ApiErrorCode, ApiOk};
use crate::repositories::item_group_repository;
use crate::AppState;

/// 【機能概要】: `POST /items/:id/groups` ハンドラ。group_type, group_name等を受け取りグループを作成する
/// 🔵 信頼性レベル: api-endpoints.md POST /items/:id/groups リクエスト例より
pub async fn create_item_group_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&id)?;
    let request: CreateItemGroupRequest = deserialize_request(body)?;

    if !item_group_repository::item_exists(&state.db, item_id).await? {
        return Err(ApiError::new(
            ApiErrorCode::ItemNotFound,
            "指定されたアイテムが見つかりません",
        ));
    }

    let group = item_group_repository::create_item_group(&state.db, item_id, request).await?;

    Ok(created_response(group))
}

/// 【機能概要】: `GET /items/:id/groups` ハンドラ。item_idに紐づくグループ一覧をdisplay_order昇順で返す
/// 🟡 信頼性レベル: api-endpoints.md REQ-010「一覧表示」からの妥当な推測
pub async fn list_item_groups_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&id)?;

    if !item_group_repository::item_exists(&state.db, item_id).await? {
        return Err(ApiError::new(
            ApiErrorCode::ItemNotFound,
            "指定されたアイテムが見つかりません",
        ));
    }

    let groups = item_group_repository::list_item_groups(&state.db, item_id).await?;

    Ok((StatusCode::OK, Json(ApiOk::new(groups))).into_response())
}

/// 【機能概要】: 作成済みグループをHTTP 201・統一レスポンス形式で返す
/// 🔵 信頼性レベル: handlers::item_relations::created_responseと対称
fn created_response(group: ItemGroup) -> axum::response::Response {
    (StatusCode::CREATED, Json(ApiOk::new(group))).into_response()
}
