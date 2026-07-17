//! item_images（画像URL）ハンドラ
//!
//! handlers/item_links.rsと対称な構造。

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::AppState;
use crate::models::item::{deserialize_request, parse_item_id};
use crate::models::item_image::{
    CreateItemImageRequest, ItemImage, parse_create_item_image_request,
};
use crate::models::response::{ApiError, ApiErrorCode, ApiOk};
use crate::repositories::item_image_repository;

/// `POST /items/:id/images` ハンドラ。url(必須)を受け取り画像URLを作成する
pub async fn create_item_image_handler(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let request: CreateItemImageRequest = deserialize_request(body)?;
    let request = parse_create_item_image_request(request)?;

    let image =
        item_image_repository::create_item_image(&state.db, item_id, request.url, request.kind)
            .await?;

    Ok(created_response(image))
}

/// `GET /items/:id/images` ハンドラ。指定itemに紐づく画像URLを一覧取得する
pub async fn list_item_images_handler(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let images = item_image_repository::list_item_images(&state.db, item_id).await?;
    Ok(Json(ApiOk::new(images)).into_response())
}

/// `DELETE /items/:id/images/:image_id` ハンドラ。指定画像URLを削除する
pub async fn delete_item_image_handler(
    State(state): State<AppState>,
    Path((item_id, image_id)): Path<(String, String)>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let image_id = parse_item_id(&image_id)?;

    let deleted = item_image_repository::delete_item_image(&state.db, item_id, image_id).await?;
    if !deleted {
        return Err(ApiError::new(
            ApiErrorCode::ItemNotFound,
            "指定された画像が見つかりません",
        ));
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}

/// 作成済み画像URLをHTTP 201・統一レスポンス形式で返す
fn created_response(image: ItemImage) -> axum::response::Response {
    (StatusCode::CREATED, Json(ApiOk::new(image))).into_response()
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
            .expect("ルーティング統合テストにはDATABASE_URL環境変数が必要です");
        let db = PgPool::connect(&database_url)
            .await
            .expect("テスト用DBへの接続に失敗しました");
        AppState {
            db,
            internal_api_key: String::new(),
        }
    }

    #[tokio::test]
    #[ignore]
    async fn post_item_image_with_empty_url_returns_400() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);
        let item_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/items/{item_id}/images"))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::json!({ "url": "" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[ignore]
    async fn post_item_image_with_nonexistent_item_returns_404() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);
        let item_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/items/{item_id}/images"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "url": "https://example.com/image.jpg" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[ignore]
    async fn delete_item_image_with_nonexistent_id_returns_404() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);
        let item_id = Uuid::new_v4();
        let image_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/items/{item_id}/images/{image_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
