//! citations（引用）ハンドラ

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::AppState;
use crate::models::citation::{
    Citation, CreateCitationRequest, UpdateCitationRequest, parse_create_citation_request,
    parse_update_citation_request,
};
use crate::models::item::{deserialize_request, parse_item_id};
use crate::models::response::{ApiError, ApiErrorCode, ApiOk};
use crate::repositories::citation_repository;

/// `POST /items/:id/citations` ハンドラ。quote_text(必須)/locator_type(必須)等を受け取り引用を作成する
pub async fn create_citation_handler(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let request: CreateCitationRequest = deserialize_request(body)?;
    let request = parse_create_citation_request(request)?;

    let citation = citation_repository::create_citation(
        &state.db,
        item_id,
        request.quote_text,
        request.note,
        request.locator_type,
        request.page_number,
        request.timestamp_seconds,
        request.location_number,
        request.chapter,
    )
    .await?;

    Ok(created_response(citation))
}

/// `GET /items/:id/citations` ハンドラ。指定itemに紐づく引用を一覧取得する
pub async fn list_citations_handler(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let citations = citation_repository::list_citations(&state.db, item_id).await?;
    Ok(Json(ApiOk::new(citations)).into_response())
}

/// `PATCH /citations/:id` ハンドラ。指定引用を部分更新する
pub async fn update_citation_handler(
    State(state): State<AppState>,
    Path(citation_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    let citation_id = parse_item_id(&citation_id)?;
    let request: UpdateCitationRequest = deserialize_request(body)?;
    let request = parse_update_citation_request(request)?;

    let citation = citation_repository::update_citation(&state.db, citation_id, request).await?;

    match citation {
        Some(citation) => Ok(Json(ApiOk::new(citation)).into_response()),
        None => Err(ApiError::new(
            ApiErrorCode::CitationNotFound,
            "指定された引用が見つかりません",
        )),
    }
}

/// `DELETE /citations/:id` ハンドラ。指定引用を削除する
pub async fn delete_citation_handler(
    State(state): State<AppState>,
    Path(citation_id): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    let citation_id = parse_item_id(&citation_id)?;

    let deleted = citation_repository::delete_citation(&state.db, citation_id).await?;
    if !deleted {
        return Err(ApiError::new(
            ApiErrorCode::CitationNotFound,
            "指定された引用が見つかりません",
        ));
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}

/// 作成済み引用をHTTP 201・統一レスポンス形式で返す
fn created_response(citation: Citation) -> axum::response::Response {
    (StatusCode::CREATED, Json(ApiOk::new(citation))).into_response()
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
            .expect("citationsルーティング統合テストにはDATABASE_URL環境変数が必要です");
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
    async fn post_citation_with_empty_quote_text_returns_400() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);
        let item_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/items/{item_id}/citations"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "quote_text": "", "locator_type": "none" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[ignore]
    async fn post_citation_with_nonexistent_item_returns_404() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);
        let item_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/items/{item_id}/citations"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "quote_text": "引用文",
                            "locator_type": "page",
                            "page_number": 10
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[ignore]
    async fn patch_citation_with_nonexistent_id_returns_404() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);
        let citation_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/citations/{citation_id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "page_number": 5 }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[ignore]
    async fn delete_citation_with_nonexistent_id_returns_404() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);
        let citation_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/citations/{citation_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
