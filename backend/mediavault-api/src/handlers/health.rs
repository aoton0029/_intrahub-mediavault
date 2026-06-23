//! ヘルスチェックハンドラ
//!
//! TASK-0007: Axumルーター骨格・DB接続プール設定・main.rs実装

use axum::extract::State;
use axum::response::IntoResponse;

use crate::models::response::{ApiError, ApiErrorCode, ApiOk};
use crate::AppState;

/// `GET /health` ハンドラ。DB接続確認を行い、成功時は200で `{"status": "ok"}` を返す。
pub async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => Ok(ApiOk::new(serde_json::json!({"status": "ok"}))),
        Err(_) => Err(ApiError::new(
            ApiErrorCode::InternalError,
            "DB接続に失敗しました",
        )),
    }
}
