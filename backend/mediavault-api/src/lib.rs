//! mediavault-api ライブラリルート
//!
//! TASK-0032: `tests/`配下の統合テスト（`tests/*.rs`）がクレート内部
//! （`AppState`・`routes::build_router`・`handlers`・`models`等）へアクセスするためには
//! ライブラリターゲットが必要となる。本クレートはこれまで`main.rs`のみのバイナリクレートで
//! あったため、`main.rs`の既存`mod`宣言群をそのまま本ファイルへ移し、`main.rs`側は
//! `mediavault_api::*`を`use`する薄いエントリポイントに変更する。
//! 既存モジュールの実装・公開性は一切変更しない（振る舞いの変更なし、コンパイル経路の追加のみ）。

pub mod db;
pub mod handlers;
pub mod import;
pub mod middleware;
pub mod models;
pub mod repositories;
pub mod routes;
pub mod services;

use sqlx::PgPool;

/// Axumハンドラ間で共有するアプリケーション状態
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub internal_api_key: String,
}
