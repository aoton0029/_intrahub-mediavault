//! citations（作品・論文からの引用）のモデル・リクエストDTO・バリデーション

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::response::{ApiError, ApiErrorCode};

/// 引用の付加情報の種類（映像作品の秒数、書籍・論文のページ番号、電子書籍の位置No.等）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "locator_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum LocatorType {
    Page,
    Timestamp,
    Location,
    Chapter,
    None,
}

/// citations本体（各エンドポイントのレスポンスで返す表現）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Citation {
    pub id: Uuid,
    pub item_id: Uuid,
    pub quote_text: String,
    pub note: Option<String>,
    pub locator_type: LocatorType,
    pub page_number: Option<i32>,
    pub timestamp_seconds: Option<i32>,
    pub location_number: Option<i32>,
    pub chapter: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// `POST /items/:id/citations` リクエストDTO
#[derive(Debug, Clone, Deserialize)]
pub struct CreateCitationRequest {
    pub quote_text: String,
    #[serde(default)]
    pub note: Option<String>,
    pub locator_type: LocatorType,
    #[serde(default)]
    pub page_number: Option<i32>,
    #[serde(default)]
    pub timestamp_seconds: Option<i32>,
    #[serde(default)]
    pub location_number: Option<i32>,
    #[serde(default)]
    pub chapter: Option<String>,
}

/// `PATCH /citations/:id` リクエストDTO（部分更新）
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateCitationRequest {
    pub quote_text: Option<String>,
    pub note: Option<String>,
    pub locator_type: Option<LocatorType>,
    pub page_number: Option<i32>,
    pub timestamp_seconds: Option<i32>,
    pub location_number: Option<i32>,
    pub chapter: Option<String>,
}

/// CreateCitationRequestのバリデーション（quote_text空文字拒否）を行う
pub fn parse_create_citation_request(
    request: CreateCitationRequest,
) -> Result<CreateCitationRequest, ApiError> {
    if request.quote_text.trim().is_empty() {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "quote_textは必須です",
        ));
    }
    Ok(request)
}

/// UpdateCitationRequestのバリデーション（quote_text指定時の空文字拒否）を行う
pub fn parse_update_citation_request(
    request: UpdateCitationRequest,
) -> Result<UpdateCitationRequest, ApiError> {
    if let Some(quote_text) = &request.quote_text
        && quote_text.trim().is_empty()
    {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "quote_textは空にできません",
        ));
    }
    Ok(request)
}

/// UpdateCitationRequestの全フィールドが`None`かどうかを判定する。
pub fn has_any_update_field(request: &UpdateCitationRequest) -> bool {
    request.quote_text.is_some()
        || request.note.is_some()
        || request.locator_type.is_some()
        || request.page_number.is_some()
        || request.timestamp_seconds.is_some()
        || request.location_number.is_some()
        || request.chapter.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_citation_request_deserializes_valid_fields() {
        let value = serde_json::json!({
            "quote_text": "人は見たいものしか見ようとしない。",
            "locator_type": "page",
            "page_number": 128
        });

        let request: CreateCitationRequest = serde_json::from_value(value).unwrap();

        assert_eq!(request.quote_text, "人は見たいものしか見ようとしない。");
        assert_eq!(request.locator_type, LocatorType::Page);
        assert_eq!(request.page_number, Some(128));
    }

    #[test]
    fn parse_create_citation_request_rejects_empty_quote_text() {
        let request = CreateCitationRequest {
            quote_text: "".to_string(),
            note: None,
            locator_type: LocatorType::None,
            page_number: None,
            timestamp_seconds: None,
            location_number: None,
            chapter: None,
        };

        let result = parse_create_citation_request(request);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().error.code,
            ApiErrorCode::ValidationError.as_code_str()
        );
    }

    #[test]
    fn parse_create_citation_request_accepts_valid_fields() {
        let request = CreateCitationRequest {
            quote_text: "引用文".to_string(),
            note: None,
            locator_type: LocatorType::None,
            page_number: None,
            timestamp_seconds: None,
            location_number: None,
            chapter: None,
        };

        let result = parse_create_citation_request(request);

        assert!(result.is_ok());
    }

    #[test]
    fn parse_update_citation_request_rejects_empty_quote_text() {
        let request = UpdateCitationRequest {
            quote_text: Some("".to_string()),
            note: None,
            locator_type: None,
            page_number: None,
            timestamp_seconds: None,
            location_number: None,
            chapter: None,
        };

        let result = parse_update_citation_request(request);

        assert!(result.is_err());
    }

    #[test]
    fn parse_update_citation_request_allows_omitted_quote_text() {
        let request = UpdateCitationRequest {
            quote_text: None,
            note: None,
            locator_type: None,
            page_number: Some(42),
            timestamp_seconds: None,
            location_number: None,
            chapter: None,
        };

        let result = parse_update_citation_request(request);

        assert!(result.is_ok());
    }

    #[test]
    fn has_any_update_field_returns_false_when_all_fields_none() {
        let request = UpdateCitationRequest {
            quote_text: None,
            note: None,
            locator_type: None,
            page_number: None,
            timestamp_seconds: None,
            location_number: None,
            chapter: None,
        };

        assert!(!has_any_update_field(&request));
    }

    #[test]
    fn has_any_update_field_returns_true_when_one_field_set() {
        let request = UpdateCitationRequest {
            quote_text: None,
            note: None,
            locator_type: None,
            page_number: Some(1),
            timestamp_seconds: None,
            location_number: None,
            chapter: None,
        };

        assert!(has_any_update_field(&request));
    }
}
