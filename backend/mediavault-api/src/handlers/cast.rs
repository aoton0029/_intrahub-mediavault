//! cast（キャスト管理）ハンドラ
//!
//! handlers/staff.rsと対称な構造（roleを持たない点のみ異なる）。

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::AppState;
use crate::models::cast::{
    Cast, CastListQuery, CreateCastRequest, CreateItemCastRequest, ItemCast,
    parse_create_cast_request, parse_create_item_cast_request,
};
use crate::models::item::{deserialize_request, parse_item_id};
use crate::models::response::{ApiError, ApiErrorCode, ApiOk};
use crate::repositories::cast_repository;

/// `GET /cast?q=...` ハンドラ。氏名部分一致でキャストを検索する
/// qが空/未指定の場合はDBへ問い合わせず空配列を返す（一覧全件取得を避ける）
pub async fn list_cast_handler(
    State(state): State<AppState>,
    Query(query): Query<CastListQuery>,
) -> Result<axum::response::Response, ApiError> {
    let q = query.q.unwrap_or_default();
    let trimmed = q.trim();
    if trimmed.is_empty() {
        return Ok(Json(ApiOk::new(
            Vec::<crate::models::cast::CastSearchResult>::new(),
        ))
        .into_response());
    }

    let results = cast_repository::search_cast(&state.db, trimmed, 20).await?;
    Ok(Json(ApiOk::new(results)).into_response())
}

/// `POST /cast` ハンドラ。name(必須)/external_id(optional)/image_url(optional)を受け取りキャストを作成する
pub async fn create_cast_handler(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    let request: CreateCastRequest = deserialize_request(body)?;
    let request = parse_create_cast_request(request)?;

    let cast = cast_repository::create_cast(
        &state.db,
        request.name,
        request.external_id,
        request.image_url,
    )
    .await?;

    Ok(created_cast_response(cast))
}

/// `POST /items/:id/cast` ハンドラ。cast_id(必須)/character_name(optional)を受け取り紐付けを作成する
pub async fn create_item_cast_handler(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let request: CreateItemCastRequest = deserialize_request(body)?;
    let request = parse_create_item_cast_request(request)?;

    let item_cast =
        cast_repository::link_cast(&state.db, item_id, request.cast_id, request.character_name)
            .await?;

    Ok(created_item_cast_response(item_cast))
}

/// `GET /items/:id/cast` ハンドラ。指定itemに紐づくキャスト紐付けを一覧取得する
pub async fn list_item_cast_handler(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let cast = cast_repository::list_item_cast(&state.db, item_id).await?;
    Ok(Json(ApiOk::new(cast)).into_response())
}

/// `DELETE /items/:id/cast/:item_cast_id` ハンドラ。指定item_cast_idの紐付けを削除する
pub async fn delete_item_cast_handler(
    State(state): State<AppState>,
    Path((item_id, item_cast_id)): Path<(String, String)>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let item_cast_id = parse_item_id(&item_cast_id)?;

    let deleted = cast_repository::unlink_cast(&state.db, item_id, item_cast_id).await?;

    if !deleted {
        return Err(ApiError::new(
            ApiErrorCode::ItemNotFound,
            "指定された紐付けが見つかりません",
        ));
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}

fn created_cast_response(cast: Cast) -> axum::response::Response {
    (StatusCode::CREATED, Json(ApiOk::new(cast))).into_response()
}

fn created_item_cast_response(item_cast: ItemCast) -> axum::response::Response {
    (StatusCode::CREATED, Json(ApiOk::new(item_cast))).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use sqlx::PgPool;
    use tower::ServiceExt;
    use uuid::Uuid;

    async fn test_app_state() -> AppState {
        let database_url = std::env::var("DATABASE_URL")
            .expect("castルーティング統合テストにはDATABASE_URL環境変数が必要です");
        let db = PgPool::connect(&database_url)
            .await
            .expect("テスト用DBへの接続に失敗しました");
        AppState {
            db,
            internal_api_key: String::new(),
        }
    }

    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn post_cast_with_required_fields_only_returns_201() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/cast")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "name": "声優A" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    #[ignore]
    async fn post_cast_with_empty_name_returns_400() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/cast")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::json!({ "name": "" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[ignore]
    async fn post_item_cast_with_nonexistent_cast_id_returns_404_cast_not_found() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);
        let item_id = Uuid::new_v4();
        let nonexistent_cast_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/items/{item_id}/cast"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "cast_id": nonexistent_cast_id }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[ignore]
    async fn delete_item_cast_with_nonexistent_id_returns_404() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);
        let item_id = Uuid::new_v4();
        let nonexistent_item_cast_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/items/{item_id}/cast/{nonexistent_item_cast_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
