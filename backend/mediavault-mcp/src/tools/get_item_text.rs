//! Tool層: `get_item_text` の入出力スキーマ。

use chrono::NaiveDateTime;
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::result::outcome::{Outcome, ToolError};

#[derive(Debug, Clone, Default, serde::Deserialize, schemars::JsonSchema)]
pub struct GetItemTextParams {
    /// `search_library` で解決した item_id
    pub item_id: Uuid,
    /// 複数ファイルがある場合に `AMBIGUOUS_FILE.candidates` から選んだ file_id
    pub file_id: Option<Uuid>,
    /// 0起点のチャンク連番。ページ番号ではない
    pub chunk_index: Option<i64>,
    /// 1..=20000。省略時は API の既定値を使用する
    pub chunk_size: Option<i64>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct TextChunk {
    pub index: i64,
    pub size: i64,
    pub total_chunks: i64,
    pub label: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct ApiItemText {
    pub item_id: Uuid,
    pub file_id: Uuid,
    pub extracted_at: NaiveDateTime,
    pub extraction_version: String,
    pub extractor: JsonValue,
    pub chunk: TextChunk,
}

#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct FileCandidate {
    pub file_id: Uuid,
    pub label: Option<String>,
    pub file_type: String,
}

#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct GetItemTextResult {
    pub outcome: Outcome,
    pub item_id: Uuid,
    pub file_id: Option<Uuid>,
    pub extracted_at: Option<NaiveDateTime>,
    pub extraction_version: Option<String>,
    pub extractor: Option<JsonValue>,
    pub chunk: Option<TextChunk>,
    pub candidates: Vec<FileCandidate>,
    pub error: Option<ToolError>,
}

impl GetItemTextResult {
    pub(crate) fn success(value: ApiItemText) -> Self {
        Self {
            outcome: Outcome::Success,
            item_id: value.item_id,
            file_id: Some(value.file_id),
            extracted_at: Some(value.extracted_at),
            extraction_version: Some(value.extraction_version),
            extractor: Some(value.extractor),
            chunk: Some(value.chunk),
            candidates: Vec::new(),
            error: None,
        }
    }

    pub(crate) fn error(item_id: Uuid, outcome: Outcome, error: ToolError) -> Self {
        Self {
            outcome,
            item_id,
            file_id: None,
            extracted_at: None,
            extraction_version: None,
            extractor: None,
            chunk: None,
            candidates: Vec::new(),
            error: Some(error),
        }
    }
}
