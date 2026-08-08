use std::sync::Arc;
use std::time::Duration;

use axum::{Json, Router, middleware, routing::get};
use mediavault_mcp::api::client::ApiClient;
use mediavault_mcp::auth;
use mediavault_mcp::config::Config;
use mediavault_mcp::server::MediaVaultServer;
use rmcp::transport::streamable_http_server::StreamableHttpService;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use serde_json::{Value, json};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let config = Config::from_env().unwrap_or_else(|err| {
        tracing::error!("設定の読み込みに失敗しました: {err}");
        std::process::exit(1);
    });
    let bind_addr = config.bind_addr.clone();
    let app = build_app(Arc::new(config));

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|err| panic!("failed to bind {bind_addr}: {err}"));
    tracing::info!("mediavault-mcp listening on {}", bind_addr);
    axum::serve(listener, app).await.expect("server error");
}

/// 🔵 Intent: `/healthz` は認証ミドルウェアの外側に置く。全体に `layer` を適用してから
///    パスで除外する方式は除外漏れのリスクがあるため採らない（タスクファイルの方針通り）。
fn build_app(config: Arc<Config>) -> Router {
    let api_base_url =
        url::Url::parse(&config.api_base_url).expect("api_base_url should be validated by Config");
    let api = Arc::new(
        ApiClient::new(
            api_base_url,
            config.internal_api_key.clone(),
            Duration::from_secs(config.connect_timeout_secs),
            Duration::from_secs(config.request_timeout_secs),
        )
        .expect("ApiClient should build with validated config"),
    );

    let mcp_service = StreamableHttpService::new(
        move || Ok(MediaVaultServer::new(api.clone())),
        Arc::new(LocalSessionManager::default()),
        Default::default(),
    );

    let protected =
        Router::new()
            .nest_service("/mcp", mcp_service)
            .layer(middleware::from_fn_with_state(
                config.clone(),
                auth::bearer_auth,
            ));

    Router::new()
        .route("/healthz", get(healthz))
        .merge(protected)
}

/// 🔵 Intent: MCPプロセス自身の生存のみを返す。MediaVault-api へはアクセスしない
///    （NFR-104: api障害時にコンテナが再起動ループへ陥るのを防ぐ）。
async fn healthz() -> Json<Value> {
    Json(json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") }))
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    use axum::body::Body;
    use axum::http::Request as HttpRequest;
    use axum::http::header::AUTHORIZATION;
    use tower::ServiceExt;

    fn test_config() -> Arc<Config> {
        let mut env = HashMap::new();
        env.insert("MCP_AUTH_TOKEN".to_string(), "a".repeat(40));
        env.insert(
            "MEDIAVAULT_API_BASE_URL".to_string(),
            "http://mediavault-api:8080".to_string(),
        );
        Arc::new(Config::from_map_for_test(&env))
    }

    /// テストケース6: /healthz は認証なしで200
    #[tokio::test]
    async fn healthz_requires_no_auth() {
        let app = build_app(test_config());

        let response = app
            .oneshot(
                HttpRequest::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    /// 統合テスト3: 無認証で /mcp は401
    #[tokio::test]
    async fn mcp_requires_auth() {
        let app = build_app(test_config());

        let response = app
            .oneshot(
                HttpRequest::builder()
                    .method("POST")
                    .uri("/mcp")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }

    /// 正しいトークンなら認証を通過し、rmcp サービスへ到達する（401にならない）
    #[tokio::test]
    async fn mcp_passes_with_correct_token() {
        let app = build_app(test_config());

        let response = app
            .oneshot(
                HttpRequest::builder()
                    .method("POST")
                    .uri("/mcp")
                    .header(AUTHORIZATION, format!("Bearer {}", "a".repeat(40)))
                    .header(axum::http::header::CONTENT_TYPE, "application/json")
                    .header(
                        axum::http::header::ACCEPT,
                        "application/json, text/event-stream",
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_ne!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
    }
}
