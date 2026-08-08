//! Tool層: `health`
//!
//! TASK-0010: health ツールの実装

use crate::result::outcome::Outcome;
use crate::services::health::ApiHealth;

/// `health` ツールの引数（なし）🔵 REQ-100
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct HealthParams {}

/// 🔵 interfaces.rs `HealthResult` より。
/// 確認範囲は「MediaVault-api 到達性」と「MCP 自身のバージョン」に限定する。
/// 内部APIキーの有効性・外部APIキーの設定状況は含めない。
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct HealthResult {
    pub outcome: Outcome,
    pub mcp_version: String,
    pub api: ApiHealth,
}
