//! item_streaming_links（配信URL）ハンドラ
//!
//! handlers/item_links.rsと対称な構造。

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::AppState;
use crate::models::item::{deserialize_request, parse_item_id};
use crate::models::item_streaming_link::{
    CreateItemStreamingLinkRequest, ItemStreamingLink, parse_create_item_streaming_link_request,
};
use crate::models::response::{ApiError, ApiErrorCode, ApiOk};
use crate::repositories::item_streaming_link_repository;

/// `POST /items/:id/streaming-links` ハンドラ。platform(必須)/url(必須)を受け取り配信URLを作成する
pub async fn create_item_streaming_link_handler(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let request: CreateItemStreamingLinkRequest = deserialize_request(body)?;
    let request = parse_create_item_streaming_link_request(request)?;

    let link = item_streaming_link_repository::create_item_streaming_link(
        &state.db,
        item_id,
        request.platform,
        request.url,
    )
    .await?;

    Ok(created_response(link))
}

/// `GET /items/:id/streaming-links` ハンドラ。指定itemに紐づく配信URLを一覧取得する
pub async fn list_item_streaming_links_handler(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let links =
        item_streaming_link_repository::list_item_streaming_links(&state.db, item_id).await?;
    Ok(Json(ApiOk::new(links)).into_response())
}

/// `DELETE /items/:id/streaming-links/:link_id` ハンドラ。指定配信URLを削除する
pub async fn delete_item_streaming_link_handler(
    State(state): State<AppState>,
    Path((item_id, link_id)): Path<(String, String)>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let link_id = parse_item_id(&link_id)?;

    let deleted =
        item_streaming_link_repository::delete_item_streaming_link(&state.db, item_id, link_id)
            .await?;
    if !deleted {
        return Err(ApiError::new(
            ApiErrorCode::ItemNotFound,
            "指定された配信URLが見つかりません",
        ));
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}

fn created_response(link: ItemStreamingLink) -> axum::response::Response {
    (StatusCode::CREATED, Json(ApiOk::new(link))).into_response()
}
