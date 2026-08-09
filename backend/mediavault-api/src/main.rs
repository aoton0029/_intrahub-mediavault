//! mediavault-api エントリポイント
//!
//! TASK-0007: Axumルーター骨格・DB接続プール設定・main.rs実装
//! TASK-0032: `tests/`統合テストからクレート内部を参照可能にするため、モジュール宣言・
//! `AppState`定義は`lib.rs`へ移動した。本ファイルは`mediavault_api::*`を利用する薄い
//! エントリポイントとする（振る舞いの変更なし）。

use axum::http::HeaderValue;
use mediavault_api::{AppState, db, logging, routes};
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    logging::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let internal_api_key = std::env::var("INTERNAL_API_KEY").unwrap_or_default();
    let port: u16 = std::env::var("APP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let cors_allowed_origin =
        std::env::var("CORS_ALLOWED_ORIGIN").unwrap_or_else(|_| "http://localhost".to_string());

    tracing::info!(port, "mediavault-apiを起動します");
    tracing::info!("データベースへ接続します");

    let db = match db::create_pool(&database_url).await {
        Ok(pool) => pool,
        Err(err) => {
            tracing::error!("DB接続に失敗しました: {err}");
            std::process::exit(1);
        }
    };
    tracing::info!("データベースへ接続しました");

    tracing::info!("データベースマイグレーションを確認します");
    if let Err(err) = db::run_migrations(&db).await {
        tracing::error!("マイグレーションの適用に失敗しました: {err}");
        std::process::exit(1);
    }
    tracing::info!("データベースの準備が完了しました");

    db::seed_api_credentials_from_env(&db).await;

    let state = AppState {
        db,
        internal_api_key,
    };

    // REQ-007/REQ-009・architecture.md「API設計: /api/v1 配下」に対応するため、
    // 公開APIのみ /api/v1 配下にnestする。/internal/* は内部プロセス直結用のため
    // バージョンプレフィックスを付与しない（REQ-402の境界を維持）。
    let cors = CorsLayer::new()
        .allow_origin(
            cors_allowed_origin
                .parse::<HeaderValue>()
                .unwrap_or_else(|err| panic!("CORS_ALLOWED_ORIGINが不正です: {err}")),
        )
        .allow_methods(Any)
        .allow_headers(Any);

    let app = axum::Router::new()
        .nest("/api/v1", routes::build_router(state.clone()))
        .merge(routes::internal::build_internal_router(state))
        .layer(cors);
    let app = logging::with_http_tracing(app);

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|err| panic!("ポート{port}でのリスニングに失敗しました: {err}"));

    tracing::info!(%addr, "mediavault-apiがリクエスト受付を開始しました");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("サーバーの起動に失敗しました");

    tracing::info!("mediavault-apiを停止しました");
}

async fn shutdown_signal() {
    if let Err(err) = tokio::signal::ctrl_c().await {
        tracing::error!(error = %err, "停止シグナルの待機に失敗しました");
        return;
    }

    tracing::info!("停止シグナルを受信しました");
}
