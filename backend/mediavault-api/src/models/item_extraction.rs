//! 文字抽出ジョブのDBモデル・API DTO・バリデーション。

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::models::item_file::FileType;
use crate::models::item_file_text::TextBoundary;
use crate::models::response::{ApiError, ApiErrorCode};

/// 抽出ジョブの状態。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "extraction_state", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ExtractionState {
    Queued,
    Running,
    Cancelling,
    Succeeded,
    Failed,
    Cancelled,
}

impl ExtractionState {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
    }

    pub fn is_active(self) -> bool {
        !self.is_terminal()
    }
}

/// workerへ渡すファイル参照のルート種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileRefRoot {
    Storage,
    Library,
}

/// `item_file_extractions` のDB行。
///
/// lease情報の外部流出を防ぐため、この型自体はSerializeしない。
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ItemFileExtraction {
    pub id: Uuid,
    pub item_file_id: Uuid,
    pub state: ExtractionState,
    pub attempts: i32,
    pub max_attempts: i32,
    pub progress_current: i32,
    pub progress_total: Option<i32>,
    pub claimed_by: Option<String>,
    pub lease_token: Option<Uuid>,
    pub lease_expires_at: Option<NaiveDateTime>,
    pub error: Option<JsonValue>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractorMetadata {
    pub method: ExtractionMethod,
    pub embedded_text_pages: u32,
    pub ocr_pages: u32,
    pub ocr: Option<OcrMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionMethod {
    EmbeddedText,
    Ocr,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrMetadata {
    pub engine: String,
    pub device: OcrDevice,
    pub model: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OcrDevice {
    Cpu,
    Gpu,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionError {
    pub kind: ExtractionErrorKind,
    pub message: String,
    pub retryable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionErrorKind {
    UnsupportedFormat,
    CorruptFile,
    FileNotFound,
    SizeLimitExceeded,
    OcrFailed,
    ApiUnreachable,
    LeaseExpired,
    Internal,
}

/// 公開API用の抽出状態。leaseやworkerの識別情報は含めない。
#[derive(Debug, Clone, Serialize)]
pub struct ExtractionResponse {
    pub id: Uuid,
    pub item_file_id: Uuid,
    pub state: ExtractionState,
    pub attempts: i32,
    pub max_attempts: i32,
    pub progress_current: i32,
    pub progress_total: Option<i32>,
    pub error: Option<ExtractionError>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<ItemFileExtraction> for ExtractionResponse {
    fn from(row: ItemFileExtraction) -> Self {
        Self {
            id: row.id,
            item_file_id: row.item_file_id,
            state: row.state,
            attempts: row.attempts,
            max_attempts: row.max_attempts,
            progress_current: row.progress_current,
            progress_total: row.progress_total,
            // 壊れた観測用JSONがあっても、主要な状態取得は失敗させない。
            error: row
                .error
                .and_then(|value| serde_json::from_value(value).ok()),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClaimRequest {
    pub worker_id: String,
    pub lease_seconds: i64,
}

/// 🔵 Intent: DB更新前にworker識別子とlease期間を検証し、不正なinterval生成を防ぐ。
pub fn validate_claim_request(request: &ClaimRequest) -> Result<(), ApiError> {
    if request.worker_id.trim().is_empty() {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "worker_idは必須です",
        ));
    }
    if request.lease_seconds <= 0 {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "lease_secondsは正数である必要があります",
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct ClaimResponse {
    pub extraction_id: Uuid,
    pub item_file_id: Uuid,
    pub item_id: Uuid,
    pub file_type: FileType,
    pub size_bytes: i64,
    pub attempts: i32,
    pub lease_token: Uuid,
    pub lease_expires_at: NaiveDateTime,
    pub file_ref: FileRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRef {
    pub root: FileRefRoot,
    pub relative_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HeartbeatRequest {
    pub lease_token: Uuid,
    pub progress_current: Option<i32>,
    pub progress_total: Option<i32>,
    pub lease_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeartbeatResponse {
    pub state: ExtractionState,
    pub cancel_requested: bool,
    pub lease_expires_at: NaiveDateTime,
}

/// 🔵 Intent: cancelling状態をworker向けのキャンセル通知へ一意に変換する。
pub fn cancel_requested(state: ExtractionState) -> bool {
    state == ExtractionState::Cancelling
}

/// 🔵 Intent: DB更新前に任意指定のlease期間を検証し、不正なinterval生成を防ぐ。
pub fn validate_heartbeat_request(request: &HeartbeatRequest) -> Result<(), ApiError> {
    if request.lease_seconds.is_some_and(|seconds| seconds <= 0) {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "lease_secondsは正数である必要があります",
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompleteRequest {
    pub lease_token: Uuid,
    pub content: String,
    pub boundaries: Vec<TextBoundary>,
    pub extraction_version: String,
    pub extracted_at: NaiveDateTime,
    pub extractor: ExtractorMetadata,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FailRequest {
    pub lease_token: Uuid,
    pub error: ExtractionError,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CancelledRequest {
    pub lease_token: Uuid,
}

/// 抽出本文の保存上限（Unicode文字数）。
pub const MAX_CONTENT_CHARS: usize = 5_000_000;
pub const MAX_ERROR_MESSAGE_CHARS: usize = 4_000;

/// 🟡 Intent: worker由来の巨大な診断文によるDB圧迫を、保存処理より前に拒否する。
pub fn validate_fail_request(request: &FailRequest) -> Result<(), ApiError> {
    if request.error.message.chars().count() > MAX_ERROR_MESSAGE_CHARS {
        return Err(ApiError::new(
            ApiErrorCode::UnprocessableEntity,
            "エラーメッセージが保存サイズ上限を超えています",
        ));
    }
    Ok(())
}

pub fn validate_complete_request(request: &CompleteRequest) -> Result<(), ApiError> {
    let content_chars = request.content.chars().count();
    if content_chars > MAX_CONTENT_CHARS {
        return Err(ApiError::new(
            ApiErrorCode::UnprocessableEntity,
            "抽出本文が保存サイズ上限を超えています",
        ));
    }

    let content_len = content_chars as i64;
    if request.boundaries.iter().any(|boundary| {
        boundary.start < 0 || boundary.start > boundary.end || boundary.end > content_len
    }) {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "boundariesの範囲が本文と整合しません",
        ));
    }

    Ok(())
}

pub fn is_extractable(file_type: FileType) -> bool {
    matches!(file_type, FileType::Pdf | FileType::Image)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn complete_request(content: String, boundaries: Vec<TextBoundary>) -> CompleteRequest {
        CompleteRequest {
            lease_token: Uuid::new_v4(),
            content,
            boundaries,
            extraction_version: "test-v1".to_string(),
            extracted_at: NaiveDateTime::default(),
            extractor: ExtractorMetadata {
                method: ExtractionMethod::EmbeddedText,
                embedded_text_pages: 1,
                ocr_pages: 0,
                ocr: None,
            },
        }
    }

    #[test]
    fn terminal_and_active_states_are_complements() {
        for state in [
            ExtractionState::Queued,
            ExtractionState::Running,
            ExtractionState::Cancelling,
        ] {
            assert!(!state.is_terminal());
            assert!(state.is_active());
        }
        for state in [
            ExtractionState::Succeeded,
            ExtractionState::Failed,
            ExtractionState::Cancelled,
        ] {
            assert!(state.is_terminal());
            assert!(!state.is_active());
        }
    }

    #[test]
    fn extraction_response_does_not_serialize_lease_fields() {
        let now = NaiveDateTime::default();
        let row = ItemFileExtraction {
            id: Uuid::new_v4(),
            item_file_id: Uuid::new_v4(),
            state: ExtractionState::Running,
            attempts: 1,
            max_attempts: 3,
            progress_current: 2,
            progress_total: Some(10),
            claimed_by: Some("worker-1".to_string()),
            lease_token: Some(Uuid::new_v4()),
            lease_expires_at: Some(now),
            error: None,
            created_at: now,
            updated_at: now,
        };

        let value = serde_json::to_value(ExtractionResponse::from(row)).unwrap();

        assert!(value.get("lease_token").is_none());
        assert!(value.get("lease_expires_at").is_none());
        assert!(value.get("claimed_by").is_none());
    }

    #[test]
    fn complete_request_uses_character_count_for_boundaries() {
        let request = complete_request(
            "日本語".to_string(),
            vec![TextBoundary {
                start: 0,
                end: 3,
                label: "p.1".to_string(),
            }],
        );

        assert!(validate_complete_request(&request).is_ok());
    }

    #[test]
    fn complete_request_rejects_invalid_boundaries() {
        for boundary in [
            TextBoundary {
                start: -1,
                end: 1,
                label: "negative".to_string(),
            },
            TextBoundary {
                start: 2,
                end: 1,
                label: "reversed".to_string(),
            },
            TextBoundary {
                start: 0,
                end: 4,
                label: "too-long".to_string(),
            },
        ] {
            let error =
                validate_complete_request(&complete_request("abc".to_string(), vec![boundary]))
                    .unwrap_err();
            assert_eq!(error.error.code, "VALIDATION_ERROR");
        }
    }

    #[test]
    fn complete_request_rejects_content_over_limit() {
        let request = complete_request("a".repeat(MAX_CONTENT_CHARS + 1), Vec::new());
        let error = validate_complete_request(&request).unwrap_err();
        assert_eq!(error.error.code, "UNPROCESSABLE_ENTITY");
    }

    #[test]
    fn only_pdf_and_image_are_extractable() {
        assert!(is_extractable(FileType::Pdf));
        assert!(is_extractable(FileType::Image));
        for file_type in [
            FileType::Video,
            FileType::Audio,
            FileType::Archive,
            FileType::Other,
        ] {
            assert!(!is_extractable(file_type));
        }
    }

    #[test]
    fn extraction_error_codes_have_expected_wire_codes_and_statuses() {
        use axum::http::StatusCode;

        let cases = [
            (
                ApiErrorCode::ExtractionNotFound,
                "EXTRACTION_NOT_FOUND",
                StatusCode::NOT_FOUND,
            ),
            (
                ApiErrorCode::ExtractionAlreadyFinished,
                "EXTRACTION_ALREADY_FINISHED",
                StatusCode::CONFLICT,
            ),
            (
                ApiErrorCode::UnsupportedFileType,
                "UNSUPPORTED_FILE_TYPE",
                StatusCode::UNPROCESSABLE_ENTITY,
            ),
            (
                ApiErrorCode::InvalidLeaseToken,
                "INVALID_LEASE_TOKEN",
                StatusCode::CONFLICT,
            ),
            (
                ApiErrorCode::TextNotExtracted,
                "TEXT_NOT_EXTRACTED",
                StatusCode::UNPROCESSABLE_ENTITY,
            ),
            (
                ApiErrorCode::AmbiguousFile,
                "AMBIGUOUS_FILE",
                StatusCode::CONFLICT,
            ),
        ];

        for (code, wire_code, status) in cases {
            assert_eq!(code.as_code_str(), wire_code);
            assert_eq!(code.status_code(), status);
        }
    }

    #[test]
    fn claim_request_requires_worker_and_positive_lease() {
        for request in [
            ClaimRequest {
                worker_id: String::new(),
                lease_seconds: 300,
            },
            ClaimRequest {
                worker_id: "worker-1".to_string(),
                lease_seconds: 0,
            },
        ] {
            let error = validate_claim_request(&request).unwrap_err();
            assert_eq!(error.error.code, "VALIDATION_ERROR");
        }
    }

    #[test]
    fn cancelling_state_requests_worker_cancellation() {
        assert!(cancel_requested(ExtractionState::Cancelling));
        assert!(!cancel_requested(ExtractionState::Running));
    }

    #[test]
    fn heartbeat_request_rejects_non_positive_lease() {
        let request = HeartbeatRequest {
            lease_token: Uuid::new_v4(),
            progress_current: None,
            progress_total: None,
            lease_seconds: Some(0),
        };

        let error = validate_heartbeat_request(&request).unwrap_err();
        assert_eq!(error.error.code, "VALIDATION_ERROR");
    }
}
