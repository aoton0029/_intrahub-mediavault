//! 内部API用APIキー検証ミドルウェア
//!
//! TASK-0006: 内部API用APIキー検証ミドルウェア実装

use axum::extract::Request;
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::Response;

use crate::models::response::{ApiError, ApiErrorCode};

/// `Authorization` ヘッダーを `INTERNAL_API_KEY` と比較し、内部APIへのアクセスを検証する。
///
/// ヘッダー値は `Bearer <key>` 形式、または生キー文字列のいずれかを許容する。
pub async fn api_key_auth(req: Request, next: Next) -> Result<Response, ApiError> {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let expected = std::env::var("INTERNAL_API_KEY").unwrap_or_default();

    let provided = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.strip_prefix("Bearer ").unwrap_or(value));

    match provided {
        Some(key) if !expected.is_empty() && key == expected => Ok(next.run(req).await),
        _ => {
            tracing::warn!(%method, %uri, "内部APIへの認証されていないアクセスを拒否しました");
            Err(ApiError::new(
                ApiErrorCode::Unauthorized,
                "認証に失敗しました",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request as HttpRequest, StatusCode};
    use axum::routing::get;
    use tower::ServiceExt;

    const TEST_KEY: &str = "secret-key";

    fn app() -> Router {
        unsafe {
            std::env::set_var("INTERNAL_API_KEY", TEST_KEY);
        }
        Router::new()
            .route("/ping", get(|| async { "pong" }))
            .layer(axum::middleware::from_fn(api_key_auth))
    }

    /// TC1 (TC-018-01): 正しいAPIキーで通過する
    #[tokio::test]
    async fn valid_key_passes_through() {
        let response = app()
            .oneshot(
                HttpRequest::builder()
                    .uri("/ping")
                    .header(AUTHORIZATION, TEST_KEY)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    /// TC1拡張: Bearer形式でも通過する
    #[tokio::test]
    async fn valid_bearer_key_passes_through() {
        let response = app()
            .oneshot(
                HttpRequest::builder()
                    .uri("/ping")
                    .header(AUTHORIZATION, format!("Bearer {TEST_KEY}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    /// TC2 (TC-018-E01): Authorizationヘッダーなしで401
    #[tokio::test]
    async fn missing_header_returns_401() {
        let response = app()
            .oneshot(
                HttpRequest::builder()
                    .uri("/ping")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// TC3 (TC-018-E02): 誤ったAPIキーで401
    #[tokio::test]
    async fn wrong_key_returns_401() {
        let response = app()
            .oneshot(
                HttpRequest::builder()
                    .uri("/ping")
                    .header(AUTHORIZATION, "wrong-key")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
