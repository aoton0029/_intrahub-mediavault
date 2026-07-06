//! item_links（参考リンク）ハンドラ
//!
//! TASK-0021: item_links CRUD実装（handlers/item_relations.rsと対称な構造）

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::AppState;
use crate::models::item::{deserialize_request, parse_item_id};
use crate::models::item_link::{CreateItemLinkRequest, ItemLink, parse_create_item_link_request};
use crate::models::response::{ApiError, ApiErrorCode, ApiOk};
use crate::repositories::item_link_repository;

/// 【機能概要】: `POST /items/:id/links` ハンドラ。url(必須)/label(必須)を受け取りリンクを作成する
/// 🔵 信頼性レベル: api-endpoints.md POST /items/:id/links・TASK-0021 完了条件1 に直接対応
pub async fn create_item_link_handler(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let request: CreateItemLinkRequest = deserialize_request(body)?;
    let request = parse_create_item_link_request(request)?;

    let link =
        item_link_repository::create_item_link(&state.db, item_id, request.url, request.label)
            .await?;

    Ok(created_response(link))
}

/// 【機能概要】: `GET /items/:id/links` ハンドラ。指定itemに紐づく参考リンクを一覧取得する
/// 🟡 信頼性レベル: handlers::item_groups::list_item_groups_handlerと対称
pub async fn list_item_links_handler(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let links = item_link_repository::list_item_links(&state.db, item_id).await?;
    Ok(Json(ApiOk::new(links)).into_response())
}

/// 【機能概要】: `DELETE /items/:id/links/:link_id` ハンドラ。指定リンクを削除する
/// 🔵 信頼性レベル: api-endpoints.md DELETE /items/:id/links/:link_id・TASK-0021 完了条件2 に直接対応
pub async fn delete_item_link_handler(
    State(state): State<AppState>,
    Path((item_id, link_id)): Path<(String, String)>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let link_id = parse_item_id(&link_id)?;

    let deleted = item_link_repository::delete_item_link(&state.db, item_id, link_id).await?;
    if !deleted {
        return Err(ApiError::new(
            ApiErrorCode::ItemNotFound,
            "指定されたリンクが見つかりません",
        ));
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}

/// 【機能概要】: 作成済みリンクをHTTP 201・統一レスポンス形式で返す
/// 🔵 信頼性レベル: handlers::item_relations::created_responseと対称
fn created_response(link: ItemLink) -> axum::response::Response {
    (StatusCode::CREATED, Json(ApiOk::new(link))).into_response()
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

    /// テストケース4: url空文字で400 VALIDATION_ERRORが返る（ルーター経由）
    #[tokio::test]
    #[ignore]
    async fn post_item_link_with_empty_url_returns_400() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);
        let item_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/items/{item_id}/links"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "url": "", "label": "公式サイト" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    /// テストケース: 存在しないitemへのPOSTで404が返る（ルーター経由）
    #[tokio::test]
    #[ignore]
    async fn post_item_link_with_nonexistent_item_returns_404() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);
        let item_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/items/{item_id}/links"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "url": "https://example.com", "label": "公式サイト" })
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    /// テストケース: 存在しないlink_idでのDELETEは404が返る（ルーター経由）
    #[tokio::test]
    #[ignore]
    async fn delete_item_link_with_nonexistent_id_returns_404() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);
        let item_id = Uuid::new_v4();
        let link_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/items/{item_id}/links/{link_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
