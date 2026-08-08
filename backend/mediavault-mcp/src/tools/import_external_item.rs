//! Tool層: `import_external_item`
//!
//! TASK-0017: import_external_item ツールの実装

use crate::api::models::MediaType;
use crate::result::outcome::{Outcome, ToolError};
use crate::result::summary::ItemSummary;

/// `import_external_item` ツールの引数。
///
/// 🔵 Intent: `POST /items/import` の `ImportByIdRequest` より。
///    `search_external_catalog` の候補の値を変換せずそのまま渡せる（NFR-003）。
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct ImportExternalItemParams {
    pub media_type: MediaType,
    /// `search_external_catalog` の候補から選んだ値をそのまま渡す 🔵
    pub external_id: String,
    pub provider: Option<String>,
}

/// 🔵 Intent: interfaces.rs `ImportExternalItemResult`・REQ-051 より。
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct ImportExternalItemResult {
    pub outcome: Outcome,
    pub item: Option<ItemSummary>,
    /// 409 `ITEM_ALREADY_IMPORTED` により既存Itemを返した場合 true 🔵 REQ-112
    pub already_existed: bool,
    pub error: Option<ToolError>,
}

impl ImportExternalItemResult {
    /// 登録を行わずエラーを返す場合の共通コンストラクタ。
    pub fn early_return(outcome: Outcome, error: Option<ToolError>) -> Self {
        ImportExternalItemResult {
            outcome,
            item: None,
            already_existed: false,
            error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テストケース1: 結果の構造（outcome/item/already_existed/error の4フィールド）
    #[test]
    fn result_json_has_four_fields() {
        let result = ImportExternalItemResult {
            outcome: Outcome::Success,
            item: None,
            already_existed: false,
            error: None,
        };
        let value = serde_json::to_value(&result).unwrap();
        assert!(value.get("outcome").is_some());
        assert!(value.get("item").is_some());
        assert!(value.get("already_existed").is_some());
        assert!(value.get("error").is_some());
    }
}
