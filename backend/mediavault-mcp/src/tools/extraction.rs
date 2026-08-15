//! 抽出の依頼・状態確認・キャンセルに共通するMCP型。

use serde::Serialize;
use uuid::Uuid;

use crate::result::outcome::{Outcome, ToolError};

#[derive(Debug, Clone, Copy, serde::Deserialize, schemars::JsonSchema)]
pub struct ExtractionParams {
    /// `search_library` / `get_item_context` で特定した作品ID。
    pub item_id: Uuid,
    /// `get_item_context` の files から選んだ抽出対象ファイルID。
    pub file_id: Uuid,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, Serialize, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum ExtractionState {
    Queued,
    Running,
    Cancelling,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, serde::Deserialize, Serialize, schemars::JsonSchema)]
pub struct ExtractionErrorDetail {
    pub kind: String,
    pub message: String,
    pub retryable: bool,
}

/// エージェントが次に取るべき行動。状態名だけを解釈し直さなくてよいよう明示する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionNextAction {
    Wait,
    ReadText,
    GiveUp,
    AlreadyCancelled,
    RequestExtraction,
    UseAnotherFile,
    WaitForApiRecovery,
    None,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct ExtractionResult {
    pub outcome: Outcome,
    pub item_id: Uuid,
    pub file_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<ExtractionState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_current: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_total: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attempts: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_attempts: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extraction_error: Option<ExtractionErrorDetail>,
    pub next_action: ExtractionNextAction,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ToolError>,
}
