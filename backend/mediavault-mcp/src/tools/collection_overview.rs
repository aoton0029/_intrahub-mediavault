//! Tool層: `collection_overview`
//!
//! TASK-0016: collection_overview ツールの実装

use crate::result::outcome::{Outcome, ToolError};
use crate::result::summary::ItemSummary;

/// `collection_overview` ツールの引数。
///
/// 🔵 Intent: interfaces.rs・mcp-tools.md 10. より。`recent_limit` は 1..=50、既定10 🟡
#[derive(Debug, Clone, Default, serde::Deserialize, schemars::JsonSchema)]
pub struct CollectionOverviewParams {
    pub recent_limit: Option<u32>,
}

/// `by_media_type` / `by_status` 共通の key/count エントリ。
///
/// 🔵 Intent: interfaces.rs `CountEntry` より。
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct CountEntry {
    pub key: String,
    pub count: u64,
}

/// 🔵 Intent: interfaces.rs `CollectionOverviewResult` より。
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct CollectionOverviewResult {
    pub outcome: Outcome,
    pub total_items: u64,
    /// media_type 別件数 🔵 REQ-090
    pub by_media_type: Vec<CountEntry>,
    /// status 別件数 🔵 REQ-090
    pub by_status: Vec<CountEntry>,
    /// お気に入り件数 🔵 REQ-090
    pub favorite_count: u64,
    pub recently_added: Vec<ItemSummary>,
    pub recently_updated: Vec<ItemSummary>,
    pub error: Option<ToolError>,
}

impl CollectionOverviewResult {
    /// api を呼び出さずにエラーを返す場合の共通コンストラクタ（EDGE-002: 空の成功にしない）。
    ///
    /// 🔵 Intent: `SearchLibraryResult::early_return` と同じパターン。
    pub fn early_return(outcome: Outcome, error: Option<ToolError>) -> Self {
        CollectionOverviewResult {
            outcome,
            total_items: 0,
            by_media_type: Vec::new(),
            by_status: Vec::new(),
            favorite_count: 0,
            recently_added: Vec::new(),
            recently_updated: Vec::new(),
            error,
        }
    }
}
