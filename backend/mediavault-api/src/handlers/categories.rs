//! カテゴリ・item_categories ハンドラ
//!
//! TASK-0015: タグ・カテゴリCRUD実装（handlers/tags.rsと完全に対称な構造）

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::AppState;
use crate::models::category::{Category, CreateCategoryRequest, validate_category_name};
use crate::models::item::{deserialize_request, parse_item_id};
use crate::models::response::{ApiError, ApiErrorCode, ApiOk};
use crate::repositories::category_repository;

/// 【機能概要】: `POST /categories` ハンドラ。カテゴリ名を受け取りカテゴリを作成する
/// 🔵 信頼性レベル: handlers::tags::create_tag_handlerと完全に対称
pub async fn create_category_handler(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    let request: CreateCategoryRequest = deserialize_request(body)?;
    validate_category_name(&request.name)?;

    let category = category_repository::create_category(&state.db, request.name).await?;

    Ok(created_response(category))
}

/// 【機能概要】: `DELETE /categories/:id` ハンドラ。カテゴリを削除する
/// 🔵 信頼性レベル: handlers::tags::delete_tag_handlerと完全に対称
pub async fn delete_category_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    let category_id = parse_item_id(&id)?;

    let deleted = category_repository::delete_category(&state.db, category_id).await?;
    if !deleted {
        return Err(ApiError::new(
            ApiErrorCode::CategoryNotFound,
            "指定されたカテゴリが見つかりません",
        ));
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}

/// 【機能概要】: `POST /items/:id/categories/:category_id` ハンドラ。アイテムへカテゴリを付与する
/// 🟡 信頼性レベル: handlers::tags::attach_tag_handlerと完全に対称
pub async fn attach_category_handler(
    State(state): State<AppState>,
    Path((item_id, category_id)): Path<(String, String)>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let category_id = parse_item_id(&category_id)?;

    category_repository::attach_category_to_item(&state.db, item_id, category_id).await?;

    Ok(StatusCode::CREATED.into_response())
}

/// 【機能概要】: `DELETE /items/:id/categories/:category_id` ハンドラ。アイテムからカテゴリの付与を解除する
/// 🟡 信頼性レベル: handlers::tags::detach_tag_handlerと完全に対称
pub async fn detach_category_handler(
    State(state): State<AppState>,
    Path((item_id, category_id)): Path<(String, String)>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let category_id = parse_item_id(&category_id)?;

    let detached =
        category_repository::detach_category_from_item(&state.db, item_id, category_id).await?;
    if !detached {
        return Err(ApiError::new(
            ApiErrorCode::CategoryNotFound,
            "指定された付与関係が見つかりません",
        ));
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}

/// 【機能概要】: 作成済みカテゴリをHTTP 201・統一レスポンス形式で返す
/// 🔵 信頼性レベル: handlers::tags::created_responseと完全に対称
fn created_response(category: Category) -> axum::response::Response {
    (StatusCode::CREATED, Json(ApiOk::new(category))).into_response()
}
