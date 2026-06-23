//! DB接続プール設定
//!
//! TASK-0007: Axumルーター骨格・DB接続プール設定・main.rs実装

use sqlx::postgres::{PgPool, PgPoolOptions};

/// `DATABASE_URL` からPostgreSQL接続プールを作成する。
///
/// 接続失敗時はエラーを返す（呼び出し側でログ出力後プロセスを終了させる想定）。
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}
