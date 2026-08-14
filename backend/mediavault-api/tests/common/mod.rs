//! TASK-0032: 主要フロー統合テスト共通ヘルパー
//!
//! 既存の`test_app_state()`パターン（backend/mediavault-api/src/routes/mod.rs:186-196、
//! backend/mediavault-api/src/routes/internal.rs:63-73）を踏襲し、`tests/`配下の
//! 統合テストファイル間で共有する。
//! 🔵 信頼性レベル: note.md 3章・要件定義書3章「制約条件」に直接対応

use mediavault_api::AppState;
use mediavault_api::routes::build_router;
use mediavault_api::routes::internal::build_internal_router;
use sqlx::PgPool;

/// 【テスト用ヘルパー】: `DATABASE_URL`環境変数から実DBプールへ接続し`AppState`を構築する。
/// 既存`test_app_state()`パターン（routes/mod.rs, routes/internal.rs）と同一の取得方式。
/// 🔵 信頼性レベル: note.md 3章「test_app_state()ヘルパー」に直接対応
pub async fn test_app_state() -> AppState {
    let database_url = std::env::var("DATABASE_URL")
        .expect("TASK-0032統合テストにはDATABASE_URL環境変数が必要です");
    let db = PgPool::connect(&database_url)
        .await
        .expect("テスト用DBへの接続に失敗しました");
    AppState {
        db,
        internal_api_key: String::new(),
    }
}

/// 【テスト用ヘルパー】: 公開ルーター（`build_router`）と内部ルーター（`build_internal_router`）を
/// `main.rs`同様にマージして`/api/v1`へnestしたフルAxum Routerを構築する。`tower::ServiceExt::oneshot`での
/// E2E検証に利用する。
/// 🔵 信頼性レベル: backend/mediavault-api/src/main.rsのルーター構成に直接対応
pub fn build_full_router(state: AppState) -> axum::Router {
    axum::Router::new().nest(
        "/api/v1",
        build_router(state.clone()).merge(build_internal_router(state)),
    )
}

/// 【テスト用ヘルパー】: `INTERNAL_API_KEY`環境変数を既知値に設定する。
/// `api_key_auth`ミドルウェアは`AppState`ではなく`std::env::var("INTERNAL_API_KEY")`を
/// 照合元とするため、内部API認証を伴うテストではこの関数で環境変数を設定する
/// （既存routes/internal.rs `set_internal_api_key`パターンを踏襲。Rust 2024 editionでは
/// `std::env::set_var`がunsafe化されているため`unsafe`ブロックで包む）。
/// 🔵 信頼性レベル: note.md 3章「内部APIキーはstd::env::var("INTERNAL_API_KEY")を直接読む」に直接対応
#[allow(dead_code)]
pub fn internal_api_key(key: &str) {
    unsafe {
        std::env::set_var("INTERNAL_API_KEY", key);
    }
}

/// テスト全体で使う既定の内部APIキー値
#[allow(dead_code)]
pub const TEST_INTERNAL_API_KEY: &str = "task-0032-integration-test-key";
