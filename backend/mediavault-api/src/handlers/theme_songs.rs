//! theme_songs（テーマソング）ハンドラ
//!
//! 曲マスタ（/theme-songs）とアイテムへの紐付け（/items/:id/theme-songs）の
//! 両系統をまとめて扱う。handlers/staff.rsと対称な構造。

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::AppState;
use crate::handlers::items::normalize_limit;
use crate::models::item::{deserialize_request, parse_item_id};
use crate::models::response::{ApiError, ApiErrorCode, ApiOk};
use crate::models::theme_song::{
    CreateItemThemeSongRequest, CreateThemeSongLinkRequest, CreateThemeSongRequest,
    ListThemeSongsQuery, UpdateThemeSongRequest, parse_create_theme_song_link_request,
    parse_create_theme_song_request, parse_update_theme_song_request,
};
use crate::repositories::theme_song_repository;

fn theme_song_not_found() -> ApiError {
    ApiError::new(
        ApiErrorCode::ThemeSongNotFound,
        "指定された曲が見つかりません",
    )
}

/// `GET /theme-songs` ハンドラ。曲名・artistの部分一致で曲を一覧・検索する
pub async fn list_theme_songs_handler(
    State(state): State<AppState>,
    Query(query): Query<ListThemeSongsQuery>,
) -> Result<axum::response::Response, ApiError> {
    // 【limit正規化】: GET /itemsと同じくnormalize_limitで未指定20件・上限100件へ揃える
    let limit = normalize_limit(query.limit);
    let songs =
        theme_song_repository::list_theme_songs(&state.db, query.q.as_deref(), limit.into())
            .await?;

    Ok(Json(ApiOk::new(songs)).into_response())
}

/// `POST /theme-songs` ハンドラ。曲と（指定があれば）リンクを単一トランザクションで作成する
pub async fn create_theme_song_handler(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    let request: CreateThemeSongRequest = deserialize_request(body)?;
    let request = parse_create_theme_song_request(request)?;

    let song = theme_song_repository::create_theme_song(&state.db, request).await?;

    Ok((StatusCode::CREATED, Json(ApiOk::new(song))).into_response())
}

/// `GET /theme-songs/:id` ハンドラ。リンクとその曲が使われているアイテム一覧を含めて返す
pub async fn get_theme_song_handler(
    State(state): State<AppState>,
    Path(theme_song_id): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    let theme_song_id = parse_item_id(&theme_song_id)?;

    let detail = theme_song_repository::get_theme_song_detail(&state.db, theme_song_id)
        .await?
        .ok_or_else(theme_song_not_found)?;

    Ok(Json(ApiOk::new(detail)).into_response())
}

/// `PATCH /theme-songs/:id` ハンドラ。指定されたフィールドのみ更新する
pub async fn update_theme_song_handler(
    State(state): State<AppState>,
    Path(theme_song_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    let theme_song_id = parse_item_id(&theme_song_id)?;
    let request: UpdateThemeSongRequest = deserialize_request(body)?;
    let request = parse_update_theme_song_request(request)?;

    let song = theme_song_repository::update_theme_song(&state.db, theme_song_id, request)
        .await?
        .ok_or_else(theme_song_not_found)?;

    Ok(Json(ApiOk::new(song)).into_response())
}

/// `DELETE /theme-songs/:id` ハンドラ。リンク・全アイテムへの紐付けはCASCADEで削除される
pub async fn delete_theme_song_handler(
    State(state): State<AppState>,
    Path(theme_song_id): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    let theme_song_id = parse_item_id(&theme_song_id)?;

    let deleted = theme_song_repository::delete_theme_song(&state.db, theme_song_id).await?;
    if !deleted {
        return Err(theme_song_not_found());
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}

/// `GET /theme-songs/:id/links` ハンドラ。sort_order昇順でリンクを一覧取得する
pub async fn list_theme_song_links_handler(
    State(state): State<AppState>,
    Path(theme_song_id): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    let theme_song_id = parse_item_id(&theme_song_id)?;
    let links = theme_song_repository::list_theme_song_links(&state.db, theme_song_id).await?;

    Ok(Json(ApiOk::new(links)).into_response())
}

/// `POST /theme-songs/:id/links` ハンドラ。指定曲へリンクを追加する
pub async fn create_theme_song_link_handler(
    State(state): State<AppState>,
    Path(theme_song_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    let theme_song_id = parse_item_id(&theme_song_id)?;
    let request: CreateThemeSongLinkRequest = deserialize_request(body)?;
    let request = parse_create_theme_song_link_request(request)?;

    let link =
        theme_song_repository::create_theme_song_link(&state.db, theme_song_id, request).await?;

    Ok((StatusCode::CREATED, Json(ApiOk::new(link))).into_response())
}

/// `DELETE /theme-songs/:id/links/:link_id` ハンドラ
pub async fn delete_theme_song_link_handler(
    State(state): State<AppState>,
    Path((theme_song_id, link_id)): Path<(String, String)>,
) -> Result<axum::response::Response, ApiError> {
    let theme_song_id = parse_item_id(&theme_song_id)?;
    let link_id = parse_item_id(&link_id)?;

    let deleted =
        theme_song_repository::delete_theme_song_link(&state.db, theme_song_id, link_id).await?;
    if !deleted {
        return Err(theme_song_not_found());
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}

/// `GET /items/:id/theme-songs` ハンドラ。theme_type順 → display_order順で一覧取得する
pub async fn list_item_theme_songs_handler(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let theme_songs = theme_song_repository::list_item_theme_songs(&state.db, item_id).await?;

    Ok(Json(ApiOk::new(theme_songs)).into_response())
}

/// `POST /items/:id/theme-songs` ハンドラ。既存の曲をアイテムへ紐づける
pub async fn create_item_theme_song_handler(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let request: CreateItemThemeSongRequest = deserialize_request(body)?;

    let item_theme_song = theme_song_repository::create_item_theme_song(
        &state.db,
        item_id,
        request.theme_song_id,
        request.theme_type,
        request.display_order.unwrap_or(0),
    )
    .await?;

    Ok((StatusCode::CREATED, Json(ApiOk::new(item_theme_song))).into_response())
}

/// `DELETE /items/:id/theme-songs/:item_theme_song_id` ハンドラ。紐付けのみを解除する
pub async fn delete_item_theme_song_handler(
    State(state): State<AppState>,
    Path((item_id, item_theme_song_id)): Path<(String, String)>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let item_theme_song_id = parse_item_id(&item_theme_song_id)?;

    let deleted =
        theme_song_repository::delete_item_theme_song(&state.db, item_id, item_theme_song_id)
            .await?;
    if !deleted {
        return Err(ApiError::new(
            ApiErrorCode::ItemNotFound,
            "指定されたテーマソングの紐付けが見つかりません",
        ));
    }

    Ok(StatusCode::NO_CONTENT.into_response())
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
            .expect("theme-songsルーティング統合テストにはDATABASE_URL環境変数が必要です");
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
    async fn post_theme_song_with_blank_title_returns_400() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/theme-songs")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::json!({ "title": "  " }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[ignore]
    async fn post_theme_song_with_invalid_link_type_returns_400() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/theme-songs")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "title": "曲名",
                            "links": [{ "link_type": "line_music", "url": "https://example.com" }]
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[ignore]
    async fn get_theme_song_with_nonexistent_id_returns_404() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);
        let id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/theme-songs/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[ignore]
    async fn get_theme_song_with_invalid_uuid_returns_400() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/theme-songs/not-a-uuid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[ignore]
    async fn post_item_theme_song_with_invalid_theme_type_returns_400() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);
        let item_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/items/{item_id}/theme-songs"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "theme_song_id": Uuid::new_v4(),
                            "theme_type": "opening"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[ignore]
    async fn get_item_theme_songs_with_nonexistent_item_returns_404() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);
        let item_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/items/{item_id}/theme-songs"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
