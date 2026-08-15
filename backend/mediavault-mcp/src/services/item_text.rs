//! Service層: 抽出済み本文のチャンク取得と行動可能なエラーへの変換。

use serde::Deserialize;

use crate::api::client::ApiClient;
use crate::api::error::ApiClientError;
use crate::result::outcome::{Outcome, ToolError, classify_api_error};
use crate::tools::get_item_text::{
    ApiItemText, FileCandidate, GetItemTextParams, GetItemTextResult,
};

pub async fn get_item_text(api: &ApiClient, params: GetItemTextParams) -> GetItemTextResult {
    let item_id = params.item_id;
    let mut query = Vec::new();
    if let Some(file_id) = params.file_id {
        query.push(("file_id", file_id.to_string()));
    }
    if let Some(chunk_index) = params.chunk_index {
        query.push(("chunk_index", chunk_index.to_string()));
    }
    if let Some(chunk_size) = params.chunk_size {
        query.push(("chunk_size", chunk_size.to_string()));
    }

    let path = format!("/api/v1/items/{item_id}/text");
    match api.get::<ApiItemText>(&path, &query).await {
        Ok(response) => GetItemTextResult::success(response.data),
        Err(ApiClientError::AmbiguousFile {
            message,
            candidates,
        }) => {
            let parsed = serde_json::from_value::<Vec<ApiFileCandidate>>(candidates)
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect();
            let mut result = GetItemTextResult::error(
                item_id,
                Outcome::Ambiguous,
                ToolError {
                    code: "AMBIGUOUS_FILE".to_string(),
                    message: format!(
                        "{message} candidates から file_id を選んで再試行してください"
                    ),
                    retriable: false,
                },
            );
            result.candidates = parsed;
            result
        }
        Err(err) => {
            let (outcome, mut error) = classify_api_error(&err);
            if error.code == "TEXT_NOT_EXTRACTED" {
                error.message = format!(
                    "{}。request_extraction で抽出を依頼し、完了後に再試行してください",
                    error.message
                );
            } else if error.code == "FILE_NOT_FOUND" {
                error.message = format!("{}。別の情報源を検討してください", error.message);
            }
            GetItemTextResult::error(item_id, outcome, error)
        }
    }
}

#[derive(Deserialize)]
struct ApiFileCandidate {
    file_id: uuid::Uuid,
    label: Option<String>,
    file_type: String,
}

impl From<ApiFileCandidate> for FileCandidate {
    fn from(value: ApiFileCandidate) -> Self {
        Self {
            file_id: value.file_id,
            label: value.label,
            file_type: value.file_type,
        }
    }
}
