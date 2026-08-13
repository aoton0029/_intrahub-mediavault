//! 共通結果型: `Section<T>`（コンテキスト取得の3状態）
//!
//! TASK-0009: 共通結果型と rmcp サーバー骨格

pub mod operation;
pub mod outcome;
pub mod summary;

use serde::Serialize;

use crate::api::error::ApiClientError;
pub use outcome::ToolError;
use outcome::classify_api_error;

/// コンテキストの各セクションが取りうる3状態。
///
/// 🔵 Intent: REQ-021・EDGE-105・設計決定 D-05 より。「未登録」と「取得失敗」を区別する。
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Section<T> {
    Loaded { items: Vec<T> },
    Empty,
    Failed { error: ToolError },
}

impl<T> Section<T> {
    /// `Result<Vec<T>, ApiClientError>` から `Section<T>` を作る。
    ///
    /// 🔵 Intent: EDGE-105「未登録を明示して返す」より、空の `Vec` は `Empty` に落とす。
    pub fn from_result(result: Result<Vec<T>, ApiClientError>) -> Self {
        match result {
            Ok(items) if items.is_empty() => Section::Empty,
            Ok(items) => Section::Loaded { items },
            Err(err) => {
                let (_, error) = classify_api_error(&err);
                Section::Failed { error }
            }
        }
    }

    /// 取得に失敗したセクションかどうか。
    ///
    /// 🔵 Intent: REQ-114 より、1つでも `Failed` があれば全体を `Partial` にする判定に使う。
    pub fn is_failed(&self) -> bool {
        matches!(self, Section::Failed { .. })
    }
}

/// 件数だけを返すセクション。本文を含めるとレスポンスサイズが
/// 対象依存で予測不能になるものに使う。
///
/// 🟡 Intent: 設計決定 D-12（api-tool-mapping.md §3）・NFR-002 より。`citations` は
///    `quote_text` の長さ・件数とも上限がなく、`Section<T>` と同じく本文を積むと
///    `get_item_context` 1回あたりのトークン量が Item 依存で予測不能になる。
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum CountSection {
    Loaded { count: u32 },
    Empty,
    Failed { error: ToolError },
}

impl CountSection {
    /// 件数取得の結果から `CountSection` を作る。0件は `Empty` に落とす。
    ///
    /// 🔵 Intent: `Section::from_result` と同じ「未登録と取得失敗を区別する」規約に揃える。
    pub fn from_result<T>(result: Result<Vec<T>, ApiClientError>) -> Self {
        match result {
            Ok(items) if items.is_empty() => CountSection::Empty,
            Ok(items) => CountSection::Loaded {
                count: items.len() as u32,
            },
            Err(err) => {
                let (_, error) = classify_api_error(&err);
                CountSection::Failed { error }
            }
        }
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, CountSection::Failed { .. })
    }
}

/// シリーズ（親作品）を表すセクション。配列ではなく単一の値を持つ。
///
/// 🔵 Intent: 設計決定 D-07・intrahub-mastra REQ-016a より。利用側は Knowledge Note の
///    配置先をこの値から決定し、LLM による推測を禁じている。したがって
///    **解決できないことを `Empty` として正確に返す**のが本型の責務であり、
///    `group_name` や続編関係からの推測で埋め合わせてはならない。
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum SeriesSection {
    Loaded { item_id: uuid::Uuid, title: String },
    Empty,
    Failed { error: ToolError },
}

impl SeriesSection {
    pub fn is_failed(&self) -> bool {
        matches!(self, SeriesSection::Failed { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テストケース3: Section の3状態
    #[test]
    fn loaded_serializes_with_state_tag() {
        let section = Section::Loaded { items: vec![1, 2] };
        let value = serde_json::to_value(&section).unwrap();
        assert_eq!(
            value,
            serde_json::json!({"state": "loaded", "items": [1, 2]})
        );
    }

    #[test]
    fn empty_serializes_with_state_tag() {
        let section: Section<i32> = Section::Empty;
        let value = serde_json::to_value(&section).unwrap();
        assert_eq!(value, serde_json::json!({"state": "empty"}));
    }

    #[test]
    fn failed_serializes_with_state_tag() {
        let section: Section<i32> = Section::Failed {
            error: ToolError {
                code: "MCP_API_UNREACHABLE".to_string(),
                message: "接続に失敗しました".to_string(),
                retriable: true,
            },
        };
        let value = serde_json::to_value(&section).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "state": "failed",
                "error": {
                    "code": "MCP_API_UNREACHABLE",
                    "message": "接続に失敗しました",
                    "retriable": true,
                }
            })
        );
    }

    /// テストケース4: 空 Vec が Empty になる
    #[test]
    fn empty_vec_becomes_empty_not_loaded() {
        let result: Result<Vec<i32>, ApiClientError> = Ok(vec![]);
        let section = Section::from_result(result);
        assert!(matches!(section, Section::Empty));
    }

    #[test]
    fn non_empty_vec_becomes_loaded() {
        let result: Result<Vec<i32>, ApiClientError> = Ok(vec![1, 2, 3]);
        let section = Section::from_result(result);
        assert!(matches!(section, Section::Loaded { items } if items == vec![1, 2, 3]));
    }

    #[test]
    fn error_becomes_failed() {
        let result: Result<Vec<i32>, ApiClientError> =
            Err(ApiClientError::Connection("timeout".to_string()));
        let section = Section::from_result(result);
        assert!(matches!(section, Section::Failed { .. }));
    }

    // --- D-12: CountSection ---

    /// 件数セクションは `items` を持たず `count` のみを返す（D-12・NFR-002）。
    #[test]
    fn count_section_serializes_count_without_items() {
        let section = CountSection::from_result(Ok(vec![1, 2, 3]));
        let value = serde_json::to_value(&section).unwrap();
        assert_eq!(value, serde_json::json!({"state": "loaded", "count": 3}));
        assert!(
            value.get("items").is_none(),
            "本文を含めてはならない（レスポンスサイズが Item 依存で予測不能になる）"
        );
    }

    #[test]
    fn count_section_empty_vec_becomes_empty() {
        let section = CountSection::from_result::<i32>(Ok(vec![]));
        let value = serde_json::to_value(&section).unwrap();
        assert_eq!(value, serde_json::json!({"state": "empty"}));
    }

    #[test]
    fn count_section_error_becomes_failed() {
        let result: Result<Vec<i32>, ApiClientError> =
            Err(ApiClientError::Connection("timeout".to_string()));
        let section = CountSection::from_result(result);
        assert!(section.is_failed());
    }

    // --- D-07: SeriesSection ---

    /// シリーズは配列ではなく単一の値を持つ（D-07）。
    #[test]
    fn series_section_loaded_serializes_item_id_and_title() {
        let item_id = uuid::Uuid::new_v4();
        let section = SeriesSection::Loaded {
            item_id,
            title: "作品Aシリーズ".to_string(),
        };
        let value = serde_json::to_value(&section).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "state": "loaded",
                "item_id": item_id,
                "title": "作品Aシリーズ",
            })
        );
    }

    /// 解決できないことを `empty` として正確に返す。利用側（intrahub-mastra REQ-016a）は
    /// これを受けて未分類階層へ配置する。推測で埋め合わせない。
    #[test]
    fn series_section_empty_serializes_without_title() {
        let section = SeriesSection::Empty;
        let value = serde_json::to_value(&section).unwrap();
        assert_eq!(value, serde_json::json!({"state": "empty"}));
        assert!(value.get("title").is_none());
    }
}
