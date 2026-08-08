//! Bearer認証ミドルウェア
//!
//! TASK-0007: Bearer認証ミドルウェアと /healthz

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::Response;
use subtle::ConstantTimeEq;

use crate::config::Config;

/// 🔵 Intent: タスクファイルで指定された定数時間比較によるトークン検証。
///    `Bearer ` プレフィックス必須（既存 api とは異なりMCPクライアントの標準に合わせる）。
///    401時はパスのみをログに残し、トークン値は一切出力しない（REQ-145）。
pub async fn bearer_auth(
    State(config): State<Arc<Config>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let provided = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match provided {
        Some(token) if constant_time_eq(token, config.auth_token.expose()) => {
            Ok(next.run(req).await)
        }
        _ => {
            tracing::warn!(path = %req.uri().path(), "unauthorized request");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

/// 🔵 Intent: `subtle::ConstantTimeEq` によりタイミング攻撃を防ぐ（NFR-102）。
///    長さが異なる場合は即座に false を返すため長さの漏洩は許容する（設計文書の注記通り）。
pub fn constant_time_eq(a: &str, b: &str) -> bool {
    a.as_bytes().ct_eq(b.as_bytes()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use axum::Router;
    use axum::body::Body;
    use axum::http::Request as HttpRequest;
    use axum::middleware;
    use axum::routing::get;
    use tower::ServiceExt;

    const TOKEN: &str = "correct-horse-battery-staple-token-value";

    fn test_config() -> Arc<Config> {
        let mut env = HashMap::new();
        env.insert("MCP_AUTH_TOKEN".to_string(), TOKEN.to_string());
        env.insert(
            "MEDIAVAULT_API_BASE_URL".to_string(),
            "http://mediavault-api:8080".to_string(),
        );
        Arc::new(Config::from_map_for_test(&env))
    }

    fn app_with_counter() -> (Router, Arc<AtomicUsize>) {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_for_handler = counter.clone();
        let config = test_config();

        let router = Router::new()
            .route(
                "/mcp",
                get(move || {
                    let counter = counter_for_handler.clone();
                    async move {
                        counter.fetch_add(1, Ordering::SeqCst);
                        "ok"
                    }
                }),
            )
            .layer(middleware::from_fn_with_state(config, bearer_auth));

        (router, counter)
    }

    /// テストケース1: ヘッダーなしで401、内側のハンドラは呼ばれない
    #[tokio::test]
    async fn missing_header_returns_401_and_handler_not_called() {
        let (app, counter) = app_with_counter();

        let response = app
            .oneshot(
                HttpRequest::builder()
                    .uri("/mcp")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }

    /// テストケース2: 誤ったトークンで401
    #[tokio::test]
    async fn wrong_token_returns_401() {
        let (app, counter) = app_with_counter();

        let response = app
            .oneshot(
                HttpRequest::builder()
                    .uri("/mcp")
                    .header(AUTHORIZATION, "Bearer wrong-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }

    /// テストケース3: 正しいトークンで200、ハンドラが1回呼ばれる
    #[tokio::test]
    async fn correct_token_passes_through() {
        let (app, counter) = app_with_counter();

        let response = app
            .oneshot(
                HttpRequest::builder()
                    .uri("/mcp")
                    .header(AUTHORIZATION, format!("Bearer {TOKEN}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    /// テストケース4: Bearer プレフィックスなしは401
    #[tokio::test]
    async fn missing_bearer_prefix_returns_401() {
        let (app, counter) = app_with_counter();

        let response = app
            .oneshot(
                HttpRequest::builder()
                    .uri("/mcp")
                    .header(AUTHORIZATION, TOKEN)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }

    /// テストケース5: 部分一致・前方一致の拒否
    #[test]
    fn partial_prefix_matches_are_rejected() {
        assert!(!constant_time_eq("abc", "abcdef"));
        assert!(!constant_time_eq("abcdefg", "abcdef"));
        assert!(constant_time_eq("abcdef", "abcdef"));
    }

    /// テストケース7: 定数時間比較関数が正しく動作する
    #[test]
    fn constant_time_eq_matches_and_mismatches() {
        assert!(constant_time_eq(TOKEN, TOKEN));
        assert!(!constant_time_eq(TOKEN, "different-token"));
        assert!(!constant_time_eq("", TOKEN));
    }
}
