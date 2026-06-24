//! item_episodes（season/chapter配下の話数）ハンドラ
//!
//! TASK-0019: item_episodes CRUD + EDGE-101検証実装（handlers/item_groups.rsと対称な構造）

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::models::item::{deserialize_request, parse_item_id};
use crate::models::item_episode::{CreateItemEpisodeRequest, ItemEpisode};
use crate::models::item_group::GroupType;
use crate::models::response::{ApiError, ApiErrorCode, ApiOk};
use crate::repositories::item_episode_repository;
use crate::AppState;

/// 【機能概要】: `POST /groups/:group_id/episodes` ハンドラ。episode_number等を受け取り話数を作成する
/// 【実装方針】: 対象グループのgroup_typeを事前にSELECTし、`volume`の場合はDBへのINSERT前に
/// INVALID_GROUP_TYPE_FOR_EPISODES（400）を返す（EDGE-101のアプリケーション層チェック）
/// 🔵 信頼性レベル: TASK-0019.md 実装詳細2・テストケース1/2に直接対応
pub async fn create_item_episode_handler(
    State(state): State<AppState>,
    Path(group_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    let group_id = parse_item_id(&group_id)?;
    let request: CreateItemEpisodeRequest = deserialize_request(body)?;

    let group_type = item_episode_repository::get_group_type(&state.db, group_id)
        .await?
        .ok_or_else(|| {
            ApiError::new(
                ApiErrorCode::GroupNotFound,
                "指定されたグループが見つかりません",
            )
        })?;

    if group_type == GroupType::Volume {
        return Err(ApiError::new(
            ApiErrorCode::InvalidGroupTypeForEpisodes,
            "volume配下のグループには話数を登録できません",
        ));
    }

    let episode =
        item_episode_repository::create_item_episode(&state.db, group_id, request).await?;

    Ok(created_response(episode))
}

/// 【機能概要】: `GET /groups/:group_id/episodes` ハンドラ。group_idに紐づく話数一覧をepisode_number昇順で返す
/// 🔵 信頼性レベル: TASK-0019.md 実装詳細4・テストケース4に直接対応
pub async fn list_item_episodes_handler(
    State(state): State<AppState>,
    Path(group_id): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    let group_id = parse_item_id(&group_id)?;

    if item_episode_repository::get_group_type(&state.db, group_id)
        .await?
        .is_none()
    {
        return Err(ApiError::new(
            ApiErrorCode::GroupNotFound,
            "指定されたグループが見つかりません",
        ));
    }

    let episodes = item_episode_repository::list_item_episodes(&state.db, group_id).await?;

    Ok((StatusCode::OK, Json(ApiOk::new(episodes))).into_response())
}

/// 【機能概要】: 作成済み話数をHTTP 201・統一レスポンス形式で返す
/// 🔵 信頼性レベル: handlers::item_groups::created_responseと対称
fn created_response(episode: ItemEpisode) -> axum::response::Response {
    (StatusCode::CREATED, Json(ApiOk::new(episode))).into_response()
}
