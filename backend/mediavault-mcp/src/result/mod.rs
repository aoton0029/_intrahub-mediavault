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
}
