//! DB接続プール設定
//!
//! TASK-0007: Axumルーター骨格・DB接続プール設定・main.rs実装

use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::models::api_credential::ApiProvider;
use crate::repositories::api_credential_repository;

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

/// 環境変数に設定されている外部APIキーをapi_credentialsテーブルへ保存する。
///
/// 起動のたびに実行し、環境変数の値でDBの値を上書きする（環境変数側を正とする運用のため）。
/// 該当する環境変数が未設定・空文字の場合はそのproviderをスキップし、DBの既存値はそのまま残す。
pub async fn seed_api_credentials_from_env(pool: &PgPool) {
    seed_provider_key(pool, ApiProvider::Tmdb, std::env::var("TMDB_API_KEY").ok()).await;
    seed_provider_key(
        pool,
        ApiProvider::Annict,
        std::env::var("ANNICT_ACCESS_TOKEN").ok(),
    )
    .await;

    // Rakutenは`applicationId:accessKey`形式で1つのapi_keyにエンコードして保存する
    // （services/external_search.rsのパース仕様と一致させる）
    let rakuten_key = match (
        non_empty_env("RAKUTEN_APPLICATION_ID"),
        non_empty_env("RAKUTEN_ACCESS_KEY"),
    ) {
        (Some(application_id), Some(access_key)) => Some(format!("{application_id}:{access_key}")),
        _ => None,
    };
    seed_provider_key(pool, ApiProvider::Rakuten, rakuten_key).await;
}

fn non_empty_env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.is_empty())
}

async fn seed_provider_key(pool: &PgPool, provider: ApiProvider, api_key: Option<String>) {
    let Some(api_key) = api_key.filter(|v| !v.is_empty()) else {
        return;
    };

    if let Err(err) =
        api_credential_repository::upsert_api_credential(pool, provider, api_key).await
    {
        tracing::error!(
            "環境変数からのAPIキー保存に失敗しました（provider={provider:?}）: {err:?}"
        );
    }
}
