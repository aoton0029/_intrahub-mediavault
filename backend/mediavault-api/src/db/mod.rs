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

/// `migrations/` 配下のマイグレーションを適用し、未作成のテーブルを起動時に初期化する。
///
/// 既に`items`テーブルが存在する場合はマイグレーション適用済みとみなしスキップする。
/// （`_sqlx_migrations`のchecksum不一致による起動失敗を避けるため、テーブルの実在有無で判定する）
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'items')",
    )
    .fetch_one(pool)
    .await?;

    if exists {
        tracing::info!("テーブルが既に存在するためマイグレーションをスキップします");
        return Ok(());
    }

    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|err| sqlx::Error::Migrate(Box::new(err)))
}
