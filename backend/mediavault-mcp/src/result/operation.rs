//! 共通結果型: `OperationResult`（部分失敗の表現）
//!
//! TASK-0009: 共通結果型と rmcp サーバー骨格

use serde::Serialize;
use uuid::Uuid;

use crate::result::outcome::{Outcome, ToolError};

/// 複数段階の書き込みにおける、各操作の結果。
///
/// 🔵 Intent: REQ-114 / REQ-061 / REQ-113・EDGE-003 / EDGE-004 より。
///    「成功」「既に適用済み」「未解決」「失敗」「未処理」を区別する。
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum OperationResult {
    Applied {
        target_id: Uuid,
        target_name: String,
        created_new: bool,
    },
    AlreadyApplied {
        target_id: Uuid,
        target_name: String,
    },
    NotResolved {
        requested_name: String,
        available_names: Vec<String>,
    },
    Failed {
        requested_name: String,
        error: ToolError,
    },
    Skipped {
        requested_name: String,
    },
}

/// `Vec<OperationResult>` 群から全体の `Outcome` を決める。
///
/// 🟡 Intent: 複数ツールで同じ集約が必要になることから導かれる実装上の判断。
///    空リストは「実行対象がなかった」ため Success とする。
pub fn aggregate_outcome(results: &[OperationResult]) -> Outcome {
    if results.is_empty() {
        return Outcome::Success;
    }

    let succeeded = results
        .iter()
        .filter(|r| {
            matches!(
                r,
                OperationResult::Applied { .. } | OperationResult::AlreadyApplied { .. }
            )
        })
        .count();
    let failed = results.len() - succeeded;

    if failed == 0 {
        Outcome::Success
    } else if succeeded == 0 {
        Outcome::Error
    } else {
        Outcome::Partial
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn applied(name: &str) -> OperationResult {
        OperationResult::Applied {
            target_id: Uuid::new_v4(),
            target_name: name.to_string(),
            created_new: false,
        }
    }

    fn failed(name: &str) -> OperationResult {
        OperationResult::Failed {
            requested_name: name.to_string(),
            error: ToolError {
                code: "MCP_API_UNREACHABLE".to_string(),
                message: "接続に失敗しました".to_string(),
                retriable: true,
            },
        }
    }

    fn not_resolved(name: &str) -> OperationResult {
        OperationResult::NotResolved {
            requested_name: name.to_string(),
            available_names: vec![],
        }
    }

    /// テストケース2: OperationResult のシリアライズ
    #[test]
    fn applied_serializes_with_result_tag() {
        let target_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let result = OperationResult::Applied {
            target_id,
            target_name: "積読".to_string(),
            created_new: false,
        };

        let value = serde_json::to_value(&result).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "result": "applied",
                "target_id": "00000000-0000-0000-0000-000000000001",
                "target_name": "積読",
                "created_new": false,
            })
        );
    }

    /// テストケース7: Outcome 集約ロジック
    #[test]
    fn aggregates_all_success_as_success() {
        let results = vec![applied("a"), applied("b")];
        assert_eq!(aggregate_outcome(&results), Outcome::Success);
    }

    #[test]
    fn aggregates_partial_failure_as_partial() {
        let results = vec![applied("a"), failed("b")];
        assert_eq!(aggregate_outcome(&results), Outcome::Partial);
    }

    #[test]
    fn aggregates_all_failure_as_error() {
        let results = vec![failed("a"), not_resolved("b")];
        assert_eq!(aggregate_outcome(&results), Outcome::Error);
    }

    #[test]
    fn aggregates_empty_list_as_success() {
        let results: Vec<OperationResult> = vec![];
        assert_eq!(aggregate_outcome(&results), Outcome::Success);
    }
}
