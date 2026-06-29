//! タグ・item_tags ハンドラ
//!
//! TASK-0015: タグ・カテゴリCRUD実装

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::AppState;
use crate::models::item::{deserialize_request, parse_item_id};
use crate::models::response::{ApiError, ApiErrorCode, ApiOk};
use crate::models::tag::{CreateTagRequest, Tag, validate_tag_name};
use crate::repositories::tag_repository;

/// 【機能概要】: `POST /tags` ハンドラ。タグ名を受け取りタグを作成する
/// 【実装方針】: items系ハンドラと同様にデシリアライズ→バリデーション→DB登録→201レスポンスの順で処理する
/// 【テスト対応】: タスク仕様テストケース1（タグ作成の正常動作）、テストケース2（重複エラー）に対応
/// 🔵 信頼性レベル: api-endpoints.md「POST /tags」仕様に直接対応
pub async fn create_tag_handler(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    // 【入力値検証】: nameフィールドのデシリアライズと空文字チェックを行う 🔵
    let request: CreateTagRequest = deserialize_request(body)?;
    validate_tag_name(&request.name)?;

    // 【DB登録】: tagsテーブルへINSERTする 🔵
    let tag = tag_repository::create_tag(&state.db, request.name).await?;

    // 【成功レスポンス】: 作成済みタグを201で返す 🔵
    Ok(created_response(tag))
}

/// 【機能概要】: `DELETE /tags/:id` ハンドラ。タグを削除する
/// 【実装方針】: items系のdelete_item_handlerと同様、影響行数0なら404を返す
/// 🔵 信頼性レベル: 既存delete_item_handlerパターンに対応
pub async fn delete_tag_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    // 【パスパラメータ検証】: UUID形式であることを確認する 🔵
    let tag_id = parse_item_id(&id)?;

    // 【DB削除】: ON DELETE CASCADEによりitem_tagsの関連行も同時に削除される 🔵
    let deleted = tag_repository::delete_tag(&state.db, tag_id).await?;
    if !deleted {
        return Err(ApiError::new(
            ApiErrorCode::TagNotFound,
            "指定されたタグが見つかりません",
        ));
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}

/// 【機能概要】: `POST /items/:id/tags/:tag_id` ハンドラ。アイテムへタグを付与する
/// 🟡 信頼性レベル: タスク仕様「item_tagsへの複合キーINSERT」からの妥当な推測
pub async fn attach_tag_handler(
    State(state): State<AppState>,
    Path((item_id, tag_id)): Path<(String, String)>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let tag_id = parse_item_id(&tag_id)?;

    tag_repository::attach_tag_to_item(&state.db, item_id, tag_id).await?;

    Ok(StatusCode::CREATED.into_response())
}

/// 【機能概要】: `DELETE /items/:id/tags/:tag_id` ハンドラ。アイテムからタグの付与を解除する
/// 🟡 信頼性レベル: タスク仕様「複合キーでのDELETE」からの妥当な推測
pub async fn detach_tag_handler(
    State(state): State<AppState>,
    Path((item_id, tag_id)): Path<(String, String)>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let tag_id = parse_item_id(&tag_id)?;

    let detached = tag_repository::detach_tag_from_item(&state.db, item_id, tag_id).await?;
    if !detached {
        return Err(ApiError::new(
            ApiErrorCode::TagNotFound,
            "指定された付与関係が見つかりません",
        ));
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}

/// 【機能概要】: 作成済みタグをHTTP 201・統一レスポンス形式で返す
/// 🔵 信頼性レベル: items系のcreated_responseと同一パターン
fn created_response(tag: Tag) -> axum::response::Response {
    (StatusCode::CREATED, Json(ApiOk::new(tag))).into_response()
}
