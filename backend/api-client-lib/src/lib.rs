//! multi-api-client — 統一インターフェースを提供する複数 API クライアントライブラリ
//!
//! # 概要
//! アニメ・ゲーム・書籍・映画など複数ジャンルをカバーする 7 つの外部 API に対して
//! 統一されたインターフェースを提供する Rust ライブラリ。
//!
//! # 対応 API
//! - [`clients::annict`]  — Annict (REST)
//! - [`clients::igdb`]    — IGDB (Apicalypse / Twitch OAuth2)
//! - [`clients::jikan`]   — Jikan (REST)
//! - [`clients::ndl`]     — NDL OpenSearch (REST + XML)
//! - [`clients::openlibrary`] — Open Library (REST)
//! - [`clients::steam`]   — Steam Web API (REST)
//! - [`clients::tmdb`]    — TMDb (REST)

pub mod auth;
pub mod clients;
pub mod error;
pub mod rate_limit;
pub mod response;
pub mod traits;

pub use auth::AuthStrategy;
pub use error::ApiError;
pub use response::{ApiResponse, RawData, RequestResult};
pub use traits::ApiClient;
