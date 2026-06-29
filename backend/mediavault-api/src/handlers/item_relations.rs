//! item_relations（関連付け・DLC）ハンドラ
//!
//! TASK-0017: item_relations CRUD実装（handlers/mylists.rsと対称な構造）

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::AppState;
use crate::models::item::{deserialize_request, parse_item_id};
use crate::models::item_relation::{
    CreateItemRelationRequest, ItemRelation, validate_not_self_reference,
};
use crate::models::response::{ApiError, ApiErrorCode, ApiOk};
use crate::repositories::item_relation_repository;

/// 【機能概要】: `POST /item-relations` ハンドラ。item_id, related_item_id, relation_typeを受け取り関連付けを作成する
/// 🔵 信頼性レベル: api-endpoints.md POST /item-relations リクエスト例より
pub async fn create_item_relation_handler(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    let request: CreateItemRelationRequest = deserialize_request(body)?;
    validate_not_self_reference(&request)?;

    let relation = item_relation_repository::create_item_relation(
        &state.db,
        request.item_id,
        request.related_item_id,
        request.relation_type,
    )
    .await?;

    Ok(created_response(relation))
}

/// 【機能概要】: `DELETE /item-relations/:id` ハンドラ。指定IDの関連付けを削除する
/// 🔵 信頼性レベル: api-endpoints.md POST/DELETE /item-relations/:id より
pub async fn delete_item_relation_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    let id = parse_item_id(&id)?;

    let deleted = item_relation_repository::delete_item_relation(&state.db, id).await?;
    if !deleted {
        return Err(ApiError::new(
            ApiErrorCode::ItemNotFound,
            "指定された関連付けが見つかりません",
        ));
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}

/// 【機能概要】: 作成済み関連付けをHTTP 201・統一レスポンス形式で返す
/// 🔵 信頼性レベル: handlers::mylists::created_responseと対称
fn created_response(relation: ItemRelation) -> axum::response::Response {
    (StatusCode::CREATED, Json(ApiOk::new(relation))).into_response()
}
