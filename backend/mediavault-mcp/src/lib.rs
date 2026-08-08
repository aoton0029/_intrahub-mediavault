//! mediavault-mcp ライブラリクレート
//!
//! 🟡 Intent: `tests/` の統合テストからモジュールを利用できるようにするため、
//!    `main.rs` のロジックをライブラリクレートへ切り出す（TASK-0008 での追加）。

pub mod api;
pub mod auth;
pub mod config;
pub mod result;
pub mod server;
pub mod services;
pub mod tools;
