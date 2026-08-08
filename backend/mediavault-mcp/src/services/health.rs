//! Service層: `health` ツール
//!
//! TASK-0010: health ツールの実装

use std::time::Instant;

use crate::api::client::ApiClient;
use crate::result::outcome::{Outcome, ToolError, classify_api_error};

/// `GET /api/v1/health` への到達性と応答時間。
///
/// 🔵 Intent: interfaces.rs `ApiHealth` より。
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct ApiHealth {
    pub reachable: bool,
    pub latency_ms: Option<u64>,
    pub error: Option<ToolError>,
}

/// `GET /api/v1/health` を呼び、到達性と応答時間を測る。
///
/// 🔵 Intent: 接続失敗（`ApiClientError::Connection`）をツール全体のエラーへ
///    昇格させない（TC-100-E01・EDGE-001）。呼び出し側は常に `Outcome::Success` を返す設計。
///    確認範囲は到達性のみに限定し、内部APIキーの有効性・外部APIキーの設定状況は確認しない。
pub async fn check_api_health(api: &ApiClient) -> ApiHealth {
    let started = Instant::now();
    match api.get::<serde_json::Value>("/api/v1/health", &[]).await {
        Ok(_) => ApiHealth {
            reachable: true,
            latency_ms: Some(started.elapsed().as_millis() as u64),
            error: None,
        },
        Err(err) => {
            let (_, error) = classify_api_error(&err);
            ApiHealth {
                reachable: false,
                latency_ms: None,
                error: Some(error),
            }
        }
    }
}

/// health ツール全体の `Outcome`。api が停止していても常に `Success`（EDGE-001）。
pub fn tool_outcome() -> Outcome {
    Outcome::Success
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テストケース1: バージョン文字列は空でなく Cargo.toml の version と一致する
    #[test]
    fn cargo_pkg_version_is_not_empty() {
        assert_eq!(env!("CARGO_PKG_VERSION"), "0.1.0");
        assert!(!env!("CARGO_PKG_VERSION").is_empty());
    }

    #[test]
    fn tool_outcome_is_always_success() {
        assert_eq!(tool_outcome(), Outcome::Success);
    }
}
