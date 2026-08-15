//! 共通結果型: `Outcome` / `ToolError` / `McpErrorCode`
//!
//! TASK-0009: 共通結果型と rmcp サーバー骨格

use serde::Serialize;

use crate::api::error::ApiClientError;

/// すべてのMCPツール結果が持つ結果種別。
///
/// 🔵 Intent: 設計決定 D-01・REQ-146 / REQ-114 より。5値を snake_case でシリアライズする。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Success,
    Partial,
    Error,
    Ambiguous,
    NotFound,
}

/// ツール結果に埋め込むエラー情報。
///
/// 🔵 Intent: REQ-146・既存 `mediavault-api` の ApiErrorBody より。
///    MediaVault-api 由来の code / message をそのまま保持する。
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct ToolError {
    pub code: String,
    pub message: String,
    pub retriable: bool,
}

/// MCP 自身が生成するエラーコード。API 由来のコードと区別するため `MCP_` を付ける。
///
/// 🟡 Intent: dataflow.md エラーハンドリングフローより。`MCP_DECODE_FAILED` は
///    interfaces.rs に未記載だがデシリアライズ失敗の分類に必要なため追加する。
#[derive(Debug, Clone, Copy)]
pub enum McpErrorCode {
    ApiUnreachable,
    InternalAuthFailed,
    InvalidArgument,
    NameNotResolved,
    DecodeFailed,
}

impl McpErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ApiUnreachable => "MCP_API_UNREACHABLE",
            Self::InternalAuthFailed => "MCP_INTERNAL_AUTH_FAILED",
            Self::InvalidArgument => "MCP_INVALID_ARGUMENT",
            Self::NameNotResolved => "MCP_NAME_NOT_RESOLVED",
            Self::DecodeFailed => "MCP_DECODE_FAILED",
        }
    }
}

/// `ApiClientError` から `(Outcome, ToolError)` への変換。
///
/// 🔵 Intent: dataflow.md の変換表どおり。409 の冪等化判断はここに含めない
///    （各ツールの Service 層が個別に握る）。
pub fn classify_api_error(err: &ApiClientError) -> (Outcome, ToolError) {
    match err {
        ApiClientError::Connection(_) => (
            Outcome::Error,
            ToolError {
                code: McpErrorCode::ApiUnreachable.as_str().to_string(),
                message: err.to_string(),
                retriable: true,
            },
        ),
        ApiClientError::Auth => (
            Outcome::Error,
            ToolError {
                code: McpErrorCode::InternalAuthFailed.as_str().to_string(),
                message: err.to_string(),
                retriable: false,
            },
        ),
        ApiClientError::Api {
            code,
            message,
            status,
        } => {
            let outcome = if *status == 404 {
                Outcome::NotFound
            } else {
                Outcome::Error
            };
            let retriable = *status >= 500;
            (
                outcome,
                ToolError {
                    code: code.clone(),
                    message: message.clone(),
                    retriable,
                },
            )
        }
        ApiClientError::AmbiguousFile { message, .. } => (
            Outcome::Ambiguous,
            ToolError {
                code: "AMBIGUOUS_FILE".to_string(),
                message: message.clone(),
                retriable: false,
            },
        ),
        ApiClientError::Decode(_) => (
            Outcome::Error,
            ToolError {
                code: McpErrorCode::DecodeFailed.as_str().to_string(),
                message: err.to_string(),
                retriable: false,
            },
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テストケース1: Outcome のシリアライズ
    #[test]
    fn outcome_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_value(Outcome::NotFound).unwrap(),
            serde_json::json!("not_found")
        );
        assert_eq!(
            serde_json::to_value(Outcome::Success).unwrap(),
            serde_json::json!("success")
        );
        assert_eq!(
            serde_json::to_value(Outcome::Partial).unwrap(),
            serde_json::json!("partial")
        );
        assert_eq!(
            serde_json::to_value(Outcome::Error).unwrap(),
            serde_json::json!("error")
        );
        assert_eq!(
            serde_json::to_value(Outcome::Ambiguous).unwrap(),
            serde_json::json!("ambiguous")
        );
    }

    /// テストケース5: エラー変換マッピング（実装項目2の表の全行）
    #[test]
    fn classify_connection_error() {
        let err = ApiClientError::Connection("timeout".to_string());
        let (outcome, tool_error) = classify_api_error(&err);
        assert_eq!(outcome, Outcome::Error);
        assert_eq!(tool_error.code, "MCP_API_UNREACHABLE");
        assert!(tool_error.retriable);
    }

    #[test]
    fn classify_auth_error() {
        let err = ApiClientError::Auth;
        let (outcome, tool_error) = classify_api_error(&err);
        assert_eq!(outcome, Outcome::Error);
        assert_eq!(tool_error.code, "MCP_INTERNAL_AUTH_FAILED");
        assert!(!tool_error.retriable);
    }

    #[test]
    fn classify_api_404_as_not_found() {
        let err = ApiClientError::Api {
            code: "ITEM_NOT_FOUND".to_string(),
            message: "見つかりません".to_string(),
            status: 404,
        };
        let (outcome, tool_error) = classify_api_error(&err);
        assert_eq!(outcome, Outcome::NotFound);
        assert_eq!(tool_error.code, "ITEM_NOT_FOUND");
        assert!(!tool_error.retriable);
    }

    #[test]
    fn classify_api_400_and_422_as_error_not_retriable() {
        for status in [400, 422] {
            let err = ApiClientError::Api {
                code: "VALIDATION_ERROR".to_string(),
                message: "不正なリクエストです".to_string(),
                status,
            };
            let (outcome, tool_error) = classify_api_error(&err);
            assert_eq!(outcome, Outcome::Error);
            assert!(!tool_error.retriable);
        }
    }

    #[test]
    fn classify_api_5xx_as_error_retriable() {
        for status in [500, 502, 503] {
            let err = ApiClientError::Api {
                code: "INTERNAL_ERROR".to_string(),
                message: "サーバーエラー".to_string(),
                status,
            };
            let (outcome, tool_error) = classify_api_error(&err);
            assert_eq!(outcome, Outcome::Error);
            assert!(tool_error.retriable);
        }
    }

    #[test]
    fn classify_decode_error() {
        let err = ApiClientError::Decode("invalid json".to_string());
        let (outcome, tool_error) = classify_api_error(&err);
        assert_eq!(outcome, Outcome::Error);
        assert_eq!(tool_error.code, "MCP_DECODE_FAILED");
        assert!(!tool_error.retriable);
    }

    /// テストケース6: code / message の透過
    #[test]
    fn preserves_original_code_and_message() {
        let err = ApiClientError::Api {
            code: "TAG_NOT_FOUND".to_string(),
            message: "タグが見つかりません".to_string(),
            status: 404,
        };
        let (_, tool_error) = classify_api_error(&err);
        assert_eq!(tool_error.code, "TAG_NOT_FOUND");
        assert_eq!(tool_error.message, "タグが見つかりません");
    }
}
