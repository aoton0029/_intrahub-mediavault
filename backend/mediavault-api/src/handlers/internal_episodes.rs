//! `/internal/groups/:group_id/episodes` ハンドラ（巡回バッチ向けupsert）
//!
//! TASK-0029: 内部REST APIルート群実装（/internal/items等）

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::models::item::{deserialize_request, parse_item_id};
use crate::models::item_episode::{CreateItemEpisodeRequest, ItemEpisode};
use crate::models::item_group::GroupType;
use crate::models::response::{ApiError, ApiErrorCode};
use crate::repositories::item_episode_repository;
use crate::AppState;

/// 【機能概要】: `POST /internal/groups/:group_id/episodes` ハンドラ。巡回バッチからの話数同期用に、
/// 同一(group_id, episode_number)の話数が既に存在する場合は更新、なければ新規作成する
/// （`handlers::item_episodes::create_item_episode_handler`のupsert版）
/// 🔵 信頼性レベル: TASK-0029.md「実装詳細4」・要件定義3.4「Upsert処理」に直接対応
pub async fn upsert_item_episode_handler(
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
        item_episode_repository::upsert_item_episode(&state.db, group_id, request).await?;

    Ok(created_response(episode))
}

fn created_response(episode: ItemEpisode) -> axum::response::Response {
    (StatusCode::CREATED, Json(crate::models::response::ApiOk::new(episode))).into_response()
}
