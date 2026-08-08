//! Tool層: `organize_item`
//!
//! TASK-0020: organize_item ツールの実装

use uuid::Uuid;

use crate::result::operation::OperationResult;
use crate::result::outcome::{Outcome, ToolError};

/// `organize_item` ツールの引数。
///
/// 🔵 Intent: interfaces.rs `OrganizeItemParams`・mcp-tools.md 7. のパラメータ表より。
///    対象は `item_id`（UUID）のみ。タイトル指定は受け付けない（REQ-142）。
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct OrganizeItemParams {
    pub item_id: Uuid,
    /// タグ名 🔵 REQ-060
    #[serde(default)]
    pub tags: Vec<String>,
    /// カテゴリ名 🔵 REQ-060
    #[serde(default)]
    pub categories: Vec<String>,
    /// マイリスト名 🔵 REQ-060
    #[serde(default)]
    pub mylists: Vec<String>,
    /// タグ・カテゴリ・マイリストが存在しない場合に作成するか。既定 false
    /// 🔵 REQ-111・PRD §15.1「既定は作成しない」
    #[serde(default)]
    pub create_if_missing: bool,
}

/// 🔵 Intent: interfaces.rs `OrganizeItemResult` より。
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct OrganizeItemResult {
    pub outcome: Outcome,
    pub item_id: Uuid,
    /// 事前取得した Item のタイトル。取得自体に失敗した場合は `None` 🔵
    pub title: Option<String>,
    pub tags: Vec<OperationResult>,
    pub categories: Vec<OperationResult>,
    pub mylists: Vec<OperationResult>,
    pub error: Option<ToolError>,
}

impl OrganizeItemResult {
    /// 事前取得（`GET /items/{id}` 等）自体が失敗した場合の共通コンストラクタ。
    /// 付与は一切実行していない。
    pub fn early_return(item_id: Uuid, outcome: Outcome, error: ToolError) -> Self {
        OrganizeItemResult {
            outcome,
            item_id,
            title: None,
            tags: Vec::new(),
            categories: Vec::new(),
            mylists: Vec::new(),
            error: Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テストケース1: create_if_missing の既定値
    #[test]
    fn create_if_missing_defaults_to_false() {
        let json = serde_json::json!({
            "item_id": "00000000-0000-0000-0000-000000000001",
        });
        let params: OrganizeItemParams = serde_json::from_value(json).unwrap();

        assert!(!params.create_if_missing);
        assert!(params.tags.is_empty());
        assert!(params.categories.is_empty());
        assert!(params.mylists.is_empty());
    }

    /// 結果の構造（outcome/item_id/title/tags/categories/mylists/error の7フィールド）
    #[test]
    fn result_json_has_seven_fields() {
        let result = OrganizeItemResult {
            outcome: Outcome::Success,
            item_id: Uuid::nil(),
            title: None,
            tags: Vec::new(),
            categories: Vec::new(),
            mylists: Vec::new(),
            error: None,
        };
        let value = serde_json::to_value(&result).unwrap();
        for field in [
            "outcome",
            "item_id",
            "title",
            "tags",
            "categories",
            "mylists",
            "error",
        ] {
            assert!(value.get(field).is_some(), "missing field: {field}");
        }
    }

    /// early_return は title: null・関連付け配列が全て空
    #[test]
    fn early_return_has_null_title_and_empty_results() {
        let error = ToolError {
            code: "ITEM_NOT_FOUND".to_string(),
            message: "見つかりません".to_string(),
            retriable: false,
        };
        let result = OrganizeItemResult::early_return(Uuid::nil(), Outcome::NotFound, error);

        assert!(result.title.is_none());
        assert!(result.tags.is_empty());
        assert!(result.categories.is_empty());
        assert!(result.mylists.is_empty());
        assert!(result.error.is_some());
    }
}
