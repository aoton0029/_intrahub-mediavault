//! mediavault-api エントリポイント
//!
//! TASK-0007: Axumルーター骨格・DB接続プール設定・main.rs実装

mod db;
mod handlers;
mod middleware;
mod models;
mod repositories;
mod routes;
mod services;

use sqlx::PgPool;

/// Axumハンドラ間で共有するアプリケーション状態
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub internal_api_key: String,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let internal_api_key = std::env::var("INTERNAL_API_KEY").unwrap_or_default();
    let port: u16 = std::env::var("APP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let db = match db::create_pool(&database_url).await {
        Ok(pool) => pool,
        Err(err) => {
            tracing::error!("DB接続に失敗しました: {err}");
            std::process::exit(1);
        }
    };

    let state = AppState {
        db,
        internal_api_key,
    };

    let app = routes::build_router(state);

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|err| panic!("ポート{port}でのリスニングに失敗しました: {err}"));

    tracing::info!("mediavault-api がポート{port}でリスニングを開始しました");

    axum::serve(listener, app)
        .await
        .expect("サーバーの起動に失敗しました");
}
