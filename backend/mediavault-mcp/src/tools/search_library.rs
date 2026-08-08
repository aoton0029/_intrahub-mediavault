//! Tool層: `search_library`
//!
//! TASK-0013: search_library ツールの実装

use crate::api::models::{ItemStatus, MediaType, SortOrder};
use crate::result::outcome::{Outcome, ToolError};
use crate::result::summary::ItemSummary;

/// `search_library` ツールの引数。
///
/// 🔵 Intent: interfaces.rs `SearchLibraryParams` より。`year` / `sort` を含む（設計決定 D-06）。
#[derive(Debug, Clone, Default, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchLibraryParams {
    /// タイトル部分一致。本題・原題・別名を横断して検索する 🔵 REQ-011
    pub title: Option<String>,
    /// 🔵 REQ-010
    pub media_type: Option<MediaType>,
    /// 🔵 REQ-010
    pub status: Option<ItemStatus>,
    /// タグ名。MCP が `GET /tags` で ID へ解決する 🔵 REQ-010
    pub tag: Option<String>,
    /// カテゴリ名。同上 🔵 REQ-010
    pub category: Option<String>,
    /// 🔵 REQ-010
    pub is_favorite: Option<bool>,
    /// 公開年 🔵 ヒアリング2026-08-07 Q3
    pub year: Option<i32>,
    /// 並び順 🔵 ヒアリング2026-08-07 Q3
    pub sort: Option<SortOrder>,
    /// 1..=50。既定 20 🔵 REQ-143。丸めない
    pub limit: Option<u32>,
    /// 前回結果の `next_cursor` をそのまま渡す 🔵 REQ-130
    pub cursor: Option<String>,
}

/// 🔵 Intent: interfaces.rs `SearchLibraryResult` より。
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct SearchLibraryResult {
    pub outcome: Outcome,
    /// 検索対象が MediaVault 内であることを明示する固定値。
    /// 外部カタログ結果との取り違えを防ぐ 🔵 REQ-014 / REQ-032 / NFR-202
    pub source: &'static str,
    pub total_count: u64,
    pub items: Vec<ItemSummary>,
    /// 続きがある場合の不透明なカーソル 🟡
    pub next_cursor: Option<String>,
    /// 実際に適用された検索条件 🔵 REQ-013
    pub applied_filters: serde_json::Value,
    pub error: Option<ToolError>,
}

/// `search_library` の固定 `source` 値。
///
/// 🔵 Intent: REQ-014 / REQ-032・`search_external_catalog` の `"external_catalog"` と対になる。
pub const SOURCE: &str = "mediavault_library";

impl SearchLibraryResult {
    /// 検索を実行せずエラー・not_found を返す場合の共通コンストラクタ。
    ///
    /// 🟡 Intent: バリデーションエラー・名前解決失敗の各分岐で重複するフィールド初期化をまとめる。
    pub fn early_return(outcome: Outcome, error: Option<ToolError>) -> Self {
        SearchLibraryResult {
            outcome,
            source: SOURCE,
            total_count: 0,
            items: Vec::new(),
            next_cursor: None,
            applied_filters: serde_json::json!({}),
            error,
        }
    }
}
