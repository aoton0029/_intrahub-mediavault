//! マイリスト・mylist_items ハンドラ
//!
//! TASK-0016: マイリストCRUD実装（handlers/categories.rsと対称な構造）

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::AppState;
use crate::models::item::{deserialize_request, parse_item_id};
use crate::models::mylist::{
    AddMylistItemRequest, CreateMylistRequest, Mylist, MylistWithCovers, UpdateMylistRequest,
    validate_mylist_name,
};
use crate::models::response::{ApiError, ApiErrorCode, ApiOk};
use crate::repositories::mylist_repository;

/// 【機能概要】: `GET /mylists` ハンドラ。全マイリストをカバー画像候補・所属item数付きで一覧取得する
/// 🟡 信頼性レベル: handlers::categories::list_categories_handlerと対称
pub async fn list_mylists_handler(
    State(state): State<AppState>,
) -> Result<ApiOk<Vec<MylistWithCovers>>, ApiError> {
    let mylists = mylist_repository::list_mylists_with_covers(&state.db).await?;
    Ok(ApiOk::new(mylists))
}

/// 【機能概要】: `PATCH /mylists/:id` ハンドラ。マイリスト名を変更する
/// 🟡 信頼性レベル: category系にrenameが無いため、他ハンドラと対称な形で妥当な推測
pub async fn update_mylist_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<ApiOk<Mylist>, ApiError> {
    let mylist_id = parse_item_id(&id)?;
    let request: UpdateMylistRequest = deserialize_request(body)?;
    validate_mylist_name(&request.name)?;

    let mylist = mylist_repository::update_mylist_name(&state.db, mylist_id, request.name).await?;
    match mylist {
        Some(mylist) => Ok(ApiOk::new(mylist)),
        None => Err(ApiError::new(
            ApiErrorCode::MylistNotFound,
            "指定されたマイリストが見つかりません",
        )),
    }
}

/// 【機能概要】: `DELETE /mylists/:id` ハンドラ。マイリストを削除する
/// 🔵 信頼性レベル: handlers::categories::delete_category_handlerと完全に対称
pub async fn delete_mylist_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    let mylist_id = parse_item_id(&id)?;

    let deleted = mylist_repository::delete_mylist(&state.db, mylist_id).await?;
    if !deleted {
        return Err(ApiError::new(
            ApiErrorCode::MylistNotFound,
            "指定されたマイリストが見つかりません",
        ));
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}

/// 【機能概要】: `GET /items/:id/mylists` ハンドラ。指定アイテムが所属するマイリストを一覧取得する
/// 🟡 信頼性レベル: mylist_repository::list_mylists_for_itemの逆引きJOINに対応
pub async fn list_item_mylists_handler(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
) -> Result<ApiOk<Vec<Mylist>>, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let mylists = mylist_repository::list_mylists_for_item(&state.db, item_id).await?;
    Ok(ApiOk::new(mylists))
}

/// 【機能概要】: `POST /mylists` ハンドラ。マイリスト名を受け取りマイリストを作成する
/// 🔵 信頼性レベル: handlers::categories::create_category_handlerと対称
pub async fn create_mylist_handler(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    let request: CreateMylistRequest = deserialize_request(body)?;
    validate_mylist_name(&request.name)?;

    let mylist = mylist_repository::create_mylist(&state.db, request.name).await?;

    Ok(created_response(mylist))
}

/// 【機能概要】: `POST /mylists/:id/items` ハンドラ。マイリストへitemを追加する
/// 🔵 信頼性レベル: タスク仕様「事前に存在チェックを行いMYLIST_NOT_FOUNDを返す」に対応
pub async fn add_mylist_item_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    let mylist_id = parse_item_id(&id)?;
    let request: AddMylistItemRequest = deserialize_request(body)?;

    if !mylist_repository::mylist_exists(&state.db, mylist_id).await? {
        return Err(ApiError::new(
            ApiErrorCode::MylistNotFound,
            "指定されたマイリストが見つかりません",
        ));
    }

    mylist_repository::add_item_to_mylist(&state.db, mylist_id, request.item_id).await?;

    Ok(StatusCode::CREATED.into_response())
}

/// 【機能概要】: `DELETE /mylists/:id/items/:item_id` ハンドラ。マイリストからitemを削除する
/// 🔵 信頼性レベル: handlers::categories::detach_category_handlerと対称
pub async fn remove_mylist_item_handler(
    State(state): State<AppState>,
    Path((id, item_id)): Path<(String, String)>,
) -> Result<axum::response::Response, ApiError> {
    let mylist_id = parse_item_id(&id)?;
    let item_id = parse_item_id(&item_id)?;

    let removed = mylist_repository::remove_item_from_mylist(&state.db, mylist_id, item_id).await?;
    if !removed {
        return Err(ApiError::new(
            ApiErrorCode::ItemNotFound,
            "指定されたマイリストへの追加が見つかりません",
        ));
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}

/// 【機能概要】: 作成済みマイリストをHTTP 201・統一レスポンス形式で返す
/// 🔵 信頼性レベル: handlers::categories::created_responseと対称
fn created_response(mylist: Mylist) -> axum::response::Response {
    (StatusCode::CREATED, Json(ApiOk::new(mylist))).into_response()
}
