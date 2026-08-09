//! アプリケーションログとHTTPアクセスログの設定。

use axum::Router;
use tower_http::trace::{DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use tracing_subscriber::EnvFilter;

/// 🟡 Intent: `RUST_LOG`を尊重しつつ、未指定時も運用に必要なINFOログを標準出力へ送る。
pub fn init() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,api_client_lib=debug"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stdout)
        .init();

    install_panic_hook();
}

// 🟡 Intent: 通常のResult経路を通らないpanicも、発生位置付きでコンソールへ残す。
fn install_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        if let Some(location) = panic_info.location() {
            tracing::error!(
                file = location.file(),
                line = location.line(),
                column = location.column(),
                "panicが発生しました"
            );
        } else {
            tracing::error!("発生位置不明のpanicが発生しました");
        }
    }));
}

/// 🟡 Intent: 全ルートに共通の構造化アクセスログを適用し、個別ハンドラでの重複を避ける。
pub fn with_http_tracing(router: Router) -> Router {
    router.layer(
        TraceLayer::new_for_http()
            .make_span_with(|request: &axum::http::Request<_>| {
                tracing::info_span!(
                    "http_request",
                    request_id = %uuid::Uuid::new_v4(),
                    method = %request.method(),
                    uri = %request.uri(),
                    version = ?request.version(),
                )
            })
            .on_request(DefaultOnRequest::new().level(Level::INFO))
            .on_response(
                DefaultOnResponse::new()
                    .level(Level::INFO)
                    .latency_unit(tower_http::LatencyUnit::Millis),
            )
            .on_failure(
                DefaultOnFailure::new()
                    .level(Level::ERROR)
                    .latency_unit(tower_http::LatencyUnit::Millis),
            ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::routing::get;
    use tower::ServiceExt;

    #[tokio::test]
    async fn http_tracing_preserves_router_response() {
        let app = with_http_tracing(Router::new().route("/health", get(|| async { "ok" })));

        let response = app
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
