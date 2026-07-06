//! item_trailers（トレーラー動画リンク）ハンドラ
//!
//! TASK-0021: item_trailers CRUD実装（handlers/item_links.rsと対称な構造。labelはoptional）

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::AppState;
use crate::models::item::{deserialize_request, parse_item_id};
use crate::models::item_trailer::{
    CreateItemTrailerRequest, ItemTrailer, parse_create_item_trailer_request,
};
use crate::models::response::{ApiError, ApiErrorCode, ApiOk};
use crate::repositories::item_trailer_repository;

/// 【機能概要】: `POST /items/:id/trailers` ハンドラ。url(必須)/label(optional)を受け取りトレーラーを作成する
/// 🔵 信頼性レベル: api-endpoints.md POST /items/:id/trailers・TASK-0021 完了条件3 に直接対応
pub async fn create_item_trailer_handler(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let request: CreateItemTrailerRequest = deserialize_request(body)?;
    let request = parse_create_item_trailer_request(request)?;

    let trailer = item_trailer_repository::create_item_trailer(
        &state.db,
        item_id,
        request.url,
        request.label,
    )
    .await?;

    Ok(created_response(trailer))
}

/// 【機能概要】: `GET /items/:id/trailers` ハンドラ。指定itemに紐づくトレーラーを一覧取得する
/// 🟡 信頼性レベル: handlers::item_groups::list_item_groups_handlerと対称
pub async fn list_item_trailers_handler(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let trailers = item_trailer_repository::list_item_trailers(&state.db, item_id).await?;
    Ok(Json(ApiOk::new(trailers)).into_response())
}

/// 【機能概要】: `DELETE /items/:id/trailers/:trailer_id` ハンドラ。指定トレーラーを削除する
/// 🔵 信頼性レベル: api-endpoints.md DELETE /items/:id/trailers/:trailer_id・TASK-0021 完了条件4 に直接対応
pub async fn delete_item_trailer_handler(
    State(state): State<AppState>,
    Path((item_id, trailer_id)): Path<(String, String)>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let trailer_id = parse_item_id(&trailer_id)?;

    let deleted =
        item_trailer_repository::delete_item_trailer(&state.db, item_id, trailer_id).await?;
    if !deleted {
        return Err(ApiError::new(
            ApiErrorCode::ItemNotFound,
            "指定されたトレーラーが見つかりません",
        ));
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}

/// 【機能概要】: 作成済みトレーラーをHTTP 201・統一レスポンス形式で返す
/// 🔵 信頼性レベル: handlers::item_links::created_responseと対称
fn created_response(trailer: ItemTrailer) -> axum::response::Response {
    (StatusCode::CREATED, Json(ApiOk::new(trailer))).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use sqlx::PgPool;
    use tower::ServiceExt;
    use uuid::Uuid;

    /// 【テスト用ヘルパー】: ルーティング統合テスト用にAppStateを構築する（routes::build_router経由）
    async fn test_app_state() -> AppState {
        let database_url = std::env::var("DATABASE_URL")
            .expect("TASK-0021ルーティング統合テストにはDATABASE_URL環境変数が必要です");
        let db = PgPool::connect(&database_url)
            .await
            .expect("テスト用DBへの接続に失敗しました");
        AppState {
            db,
            internal_api_key: String::new(),
        }
    }

    /// url空文字で400 VALIDATION_ERRORが返る（ルーター経由）
    #[tokio::test]
    #[ignore]
    async fn post_item_trailer_with_empty_url_returns_400() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);
        let item_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/items/{item_id}/trailers"))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::json!({ "url": "" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    /// テストケース3: labelなしでトレーラー作成が成功する（ルーター経由）
    #[tokio::test]
    #[ignore]
    async fn post_item_trailer_without_label_returns_201() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);
        let item_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/items/{item_id}/trailers"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "url": "https://youtube.com/xxx" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    /// 存在しないtrailer_idでのDELETEは404が返る（ルーター経由）
    #[tokio::test]
    #[ignore]
    async fn delete_item_trailer_with_nonexistent_id_returns_404() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);
        let item_id = Uuid::new_v4();
        let trailer_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/items/{item_id}/trailers/{trailer_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
