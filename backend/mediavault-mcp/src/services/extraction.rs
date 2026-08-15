//! 公開APIを使う抽出操作。内部APIキーを必要とするパスは使用しない。

use chrono::NaiveDateTime;
use serde::Deserialize;

use crate::api::client::ApiClient;
use crate::api::error::ApiClientError;
use crate::result::outcome::{Outcome, classify_api_error};
use crate::tools::extraction::{
    ExtractionErrorDetail, ExtractionNextAction, ExtractionParams, ExtractionResult,
    ExtractionState,
};

#[derive(Debug, Deserialize)]
struct ApiExtraction {
    #[allow(dead_code)]
    id: uuid::Uuid,
    #[allow(dead_code)]
    item_file_id: uuid::Uuid,
    state: ExtractionState,
    attempts: i32,
    max_attempts: i32,
    progress_current: i32,
    progress_total: Option<i32>,
    error: Option<ExtractionErrorDetail>,
    #[allow(dead_code)]
    created_at: NaiveDateTime,
    #[allow(dead_code)]
    updated_at: NaiveDateTime,
}

fn path(params: ExtractionParams) -> String {
    format!(
        "/api/v1/items/{}/files/{}/extraction",
        params.item_id, params.file_id
    )
}

fn success(
    params: ExtractionParams,
    extraction: ApiExtraction,
    requested: bool,
) -> ExtractionResult {
    let next_action = match extraction.state {
        ExtractionState::Queued | ExtractionState::Running => ExtractionNextAction::Wait,
        ExtractionState::Succeeded => ExtractionNextAction::ReadText,
        ExtractionState::Cancelling | ExtractionState::Cancelled => {
            ExtractionNextAction::AlreadyCancelled
        }
        ExtractionState::Failed
            if extraction
                .error
                .as_ref()
                .is_some_and(|error| error.retryable)
                && extraction.attempts < extraction.max_attempts =>
        {
            ExtractionNextAction::Wait
        }
        ExtractionState::Failed => ExtractionNextAction::GiveUp,
    };
    let message = if requested {
        "抽出を受け付けました。処理は非同期です。get_extraction_status で succeeded を確認してから get_item_text を呼んでください。"
    } else {
        match next_action {
            ExtractionNextAction::Wait => {
                "抽出は進行中です。待ってから再度状態を確認してください。"
            }
            ExtractionNextAction::ReadText => {
                "抽出は完了しました。get_item_text で本文を読めます。"
            }
            ExtractionNextAction::GiveUp => {
                "抽出は再試行できない状態で失敗しました。同じ依頼を繰り返さないでください。"
            }
            ExtractionNextAction::AlreadyCancelled => {
                "抽出はキャンセル済み、またはキャンセル処理中です。"
            }
            _ => "抽出状態を取得しました。",
        }
    };
    ExtractionResult {
        outcome: Outcome::Success,
        item_id: params.item_id,
        file_id: params.file_id,
        state: Some(extraction.state),
        progress_current: Some(extraction.progress_current),
        progress_total: extraction.progress_total,
        attempts: Some(extraction.attempts),
        max_attempts: Some(extraction.max_attempts),
        extraction_error: extraction.error,
        next_action,
        message: message.to_string(),
        error: None,
    }
}

fn failure(params: ExtractionParams, err: ApiClientError) -> ExtractionResult {
    let (outcome, mut error) = classify_api_error(&err);
    let next_action = match error.code.as_str() {
        "FILE_NOT_FOUND" => ExtractionNextAction::UseAnotherFile,
        "UNSUPPORTED_FILE_TYPE" => ExtractionNextAction::UseAnotherFile,
        "EXTRACTION_NOT_FOUND" => ExtractionNextAction::RequestExtraction,
        "MCP_API_UNREACHABLE" => ExtractionNextAction::WaitForApiRecovery,
        _ => ExtractionNextAction::None,
    };
    error.message = match error.code.as_str() {
        "FILE_NOT_FOUND" => "指定したファイルがありません。別のファイルを選んでください。".into(),
        "UNSUPPORTED_FILE_TYPE" => {
            "このファイル形式は抽出できません。同じ依頼を繰り返さず、別の材料を使ってください。"
                .into()
        }
        "EXTRACTION_NOT_FOUND" => {
            "抽出はまだ依頼されていません。request_extraction を呼んでください。".into()
        }
        _ => error.message,
    };
    ExtractionResult {
        outcome,
        item_id: params.item_id,
        file_id: params.file_id,
        state: None,
        progress_current: None,
        progress_total: None,
        attempts: None,
        max_attempts: None,
        extraction_error: None,
        next_action,
        message: error.message.clone(),
        error: Some(error),
    }
}

pub async fn request(api: &ApiClient, params: ExtractionParams) -> ExtractionResult {
    match api.post_empty::<ApiExtraction>(&path(params)).await {
        Ok(extraction) => success(params, extraction, true),
        Err(err) => failure(params, err),
    }
}

pub async fn status(api: &ApiClient, params: ExtractionParams) -> ExtractionResult {
    match api.get::<ApiExtraction>(&path(params), &[]).await {
        Ok(response) => success(params, response.data, false),
        Err(err) => failure(params, err),
    }
}

pub async fn cancel(api: &ApiClient, params: ExtractionParams) -> ExtractionResult {
    let cancel_path = format!("{}/cancel", path(params));
    match api.post_empty::<ApiExtraction>(&cancel_path).await {
        Ok(extraction) => success(params, extraction, false),
        Err(ApiClientError::Api {
            code, status: 409, ..
        }) if code == "EXTRACTION_ALREADY_FINISHED" => ExtractionResult {
            outcome: Outcome::Success,
            item_id: params.item_id,
            file_id: params.file_id,
            state: None,
            progress_current: None,
            progress_total: None,
            attempts: None,
            max_attempts: None,
            extraction_error: None,
            next_action: ExtractionNextAction::None,
            message: "抽出はすでに終了しているため、キャンセルは不要です。".into(),
            error: None,
        },
        Err(err) => failure(params, err),
    }
}
