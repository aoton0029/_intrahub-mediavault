//! Tool層: `create_item`
//!
//! TASK-0018: create_item ツールの実装

use crate::api::models::MediaType;
use crate::result::operation::OperationResult;
use crate::result::outcome::{Outcome, ToolError};
use crate::result::summary::ItemSummary;

/// `create_item` ツールの引数。
///
/// 🔵 Intent: interfaces.rs `CreateItemParams`・mcp-tools.md 5. のパラメータ表より。
///    `title` の空白のみチェックはクライアント側で先回りせず、api の 400 を透過する
///    （タスクファイル「注意事項」）。
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateItemParams {
    pub media_type: MediaType,
    pub title: String,
    pub original_title: Option<String>,
    pub description: Option<String>,
    pub release_date: Option<chrono::NaiveDate>,
    pub homepage_url: Option<String>,
    pub rating: Option<f32>,
    pub is_favorite: Option<bool>,
    /// ファイルパス（実データ領域の絶対パス）。MCP は実在確認を行わない 🔵 REQ-041
    #[serde(default)]
    pub file_paths: Vec<String>,
    /// 参照URL。http/https のみ許可 🔵 REQ-041 / REQ-116 と同一規約
    #[serde(default)]
    pub urls: Vec<String>,
    /// タグ名 🔵 REQ-041
    #[serde(default)]
    pub tags: Vec<String>,
    /// マイリスト名 🔵 REQ-041
    #[serde(default)]
    pub mylists: Vec<String>,
    /// タグ・マイリストが存在しない場合に作成するか。既定 false
    /// 🔵 REQ-111（organize_item と同一の規約を適用）
    #[serde(default)]
    pub create_if_missing: bool,
}

/// 🔵 Intent: interfaces.rs `CreateItemResult`・REQ-114 / EDGE-004 より。
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct CreateItemResult {
    pub outcome: Outcome,
    /// Item 作成に成功していれば、後続が失敗しても必ず値が入る 🔵 EDGE-004
    pub item: Option<ItemSummary>,
    pub tags: Vec<OperationResult>,
    pub mylists: Vec<OperationResult>,
    pub files: Vec<OperationResult>,
    pub links: Vec<OperationResult>,
    pub error: Option<ToolError>,
}

impl CreateItemResult {
    /// Item 作成自体が失敗した場合の共通コンストラクタ。関連付けは一切実行していない。
    pub fn creation_failed(outcome: Outcome, error: ToolError) -> Self {
        CreateItemResult {
            outcome,
            item: None,
            tags: Vec::new(),
            mylists: Vec::new(),
            files: Vec::new(),
            links: Vec::new(),
            error: Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テストケース: 結果の構造（outcome/item/tags/mylists/files/links/error の7フィールド）
    #[test]
    fn result_json_has_seven_fields() {
        let result = CreateItemResult {
            outcome: Outcome::Success,
            item: None,
            tags: Vec::new(),
            mylists: Vec::new(),
            files: Vec::new(),
            links: Vec::new(),
            error: None,
        };
        let value = serde_json::to_value(&result).unwrap();
        for field in [
            "outcome", "item", "tags", "mylists", "files", "links", "error",
        ] {
            assert!(value.get(field).is_some(), "missing field: {field}");
        }
    }

    /// creation_failed は item: null・関連付け配列が全て空
    #[test]
    fn creation_failed_has_null_item_and_empty_associations() {
        let error = ToolError {
            code: "VALIDATION_ERROR".to_string(),
            message: "不正なリクエストです".to_string(),
            retriable: false,
        };
        let result = CreateItemResult::creation_failed(Outcome::Error, error);

        assert!(result.item.is_none());
        assert!(result.tags.is_empty());
        assert!(result.mylists.is_empty());
        assert!(result.files.is_empty());
        assert!(result.links.is_empty());
    }
}
