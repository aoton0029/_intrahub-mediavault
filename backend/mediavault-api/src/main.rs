//! mediavault-api エントリポイント
//!
//! TASK-0007: Axumルーター骨格・DB接続プール設定・main.rs実装
//! TASK-0032: `tests/`統合テストからクレート内部を参照可能にするため、モジュール宣言・
//! `AppState`定義は`lib.rs`へ移動した。本ファイルは`mediavault_api::*`を利用する薄い
//! エントリポイントとする（振る舞いの変更なし）。

use mediavault_api::{AppState, db, routes};

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

    // REQ-007/REQ-009・architecture.md「API設計: /api/v1 配下」に対応するため、
    // 公開APIのみ /api/v1 配下にnestする。/internal/* は内部プロセス直結用のため
    // バージョンプレフィックスを付与しない（REQ-402の境界を維持）。
    let app = axum::Router::new()
        .nest("/api/v1", routes::build_router(state.clone()))
        .merge(routes::internal::build_internal_router(state));

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|err| panic!("ポート{port}でのリスニングに失敗しました: {err}"));

    tracing::info!("mediavault-api がポート{port}でリスニングを開始しました");

    axum::serve(listener, app)
        .await
        .expect("サーバーの起動に失敗しました");
}
