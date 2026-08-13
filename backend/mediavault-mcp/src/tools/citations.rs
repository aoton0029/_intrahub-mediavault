//! Tool層: `list_citations` / `add_citation`
//!
//! 第2段階。設計決定 D-11（api-tool-mapping.md §3）・REQ-903 / REQ-904 / REQ-905 より。
//!
//! 引用の**更新・削除ツールは提供しない**（REQ-905）。`quote_text` は利用者が書いた本文であり、
//! AI による上書きは実質的に破壊的だからである。

use uuid::Uuid;

use crate::api::models::LocatorType;
use crate::result::outcome::{Outcome, ToolError};

/// `list_citations` ツールの引数。
///
/// 🟡 Intent: REQ-903・設計決定 D-11 より。api 側にページネーションが無いため
///    `limit` / `cursor` は MCP 側で切り出す。
#[derive(Debug, Clone, Default, serde::Deserialize, schemars::JsonSchema)]
pub struct ListCitationsParams {
    /// `search_library` で解決した item_id
    pub item_id: Uuid,
    /// 1..=50。既定 20。丸めない（`search_library` と同じ規約）
    pub limit: Option<u32>,
    /// 前回結果の `next_cursor` をそのまま渡す
    pub cursor: Option<String>,
}

/// 引用1件の表現。
///
/// 🔵 Intent: 位置情報は api の値を**そのまま透過**する。「p.128」のような表示文字列へ
///    整形しない（REQ-146）。整形すると利用者側で位置種別ごとの処理ができなくなる。
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct CitationView {
    pub citation_id: Uuid,
    pub quote_text: String,
    pub note: Option<String>,
    pub locator_type: LocatorType,
    pub page_number: Option<i32>,
    pub timestamp_seconds: Option<i32>,
    pub location_number: Option<i32>,
    pub chapter: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

/// 🟡 Intent: REQ-903 より。api が全件を返すため `total_count` は常に確定する
///    （`search_library` が `include_total` を必要とするのとは異なる）。
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct ListCitationsResult {
    pub outcome: Outcome,
    pub item_id: Uuid,
    pub total_count: u32,
    pub citations: Vec<CitationView>,
    pub next_cursor: Option<String>,
    pub error: Option<ToolError>,
}

impl ListCitationsResult {
    pub fn early_return(item_id: Uuid, outcome: Outcome, error: Option<ToolError>) -> Self {
        ListCitationsResult {
            outcome,
            item_id,
            total_count: 0,
            citations: Vec::new(),
            next_cursor: None,
            error,
        }
    }
}

/// `add_citation` ツールの引数。
///
/// 🟡 Intent: REQ-904・設計決定 D-11 より。`locator_type` と位置フィールドの整合は
///    **MCP 側で検証する**（api は必須バリデーションしないため）。
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct AddCitationParams {
    /// `search_library` で解決した item_id（設計決定 D-02: 書き込み系は UUID のみ）
    pub item_id: Uuid,
    /// 引用本文
    pub quote_text: String,
    /// 位置情報の種別。指定した種別に対応する位置フィールドが必須になる
    pub locator_type: LocatorType,
    /// 引用に対する所感・文脈
    pub note: Option<String>,
    /// `locator_type: "page"` のとき必須
    pub page_number: Option<i32>,
    /// `locator_type: "timestamp"` のとき必須
    pub timestamp_seconds: Option<i32>,
    /// `locator_type: "location"` のとき必須
    pub location_number: Option<i32>,
    /// `locator_type: "chapter"` のとき必須
    pub chapter: Option<String>,
}

/// 🟡 Intent: REQ-904 より。
///
/// ⚠️ **冪等ではない**。api に重複検出がなく、同一 `quote_text` の二重登録を防げない。
///    設計決定 D-03「冪等性は事前取得 + 差分適用で担保」の明示的な例外であり、
///    PRD §6 原則6 の適用外（引用には一意キーが無く、差分判定の基準を作れないため）。
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct AddCitationResult {
    pub outcome: Outcome,
    pub citation: Option<CitationView>,
    pub error: Option<ToolError>,
}

impl AddCitationResult {
    pub fn early_return(outcome: Outcome, error: Option<ToolError>) -> Self {
        AddCitationResult {
            outcome,
            citation: None,
            error,
        }
    }
}
