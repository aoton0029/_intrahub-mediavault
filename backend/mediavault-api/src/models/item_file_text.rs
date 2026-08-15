//! 抽出済み本文のDBモデル・公開API DTO・チャンク補助関数。

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::models::item_file::FileType;
use crate::models::response::{ApiError, ApiErrorCode};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ItemFileText {
    pub id: Uuid,
    pub item_file_id: Uuid,
    pub content: String,
    pub boundaries: JsonValue,
    pub extraction_version: String,
    pub extractor: JsonValue,
    pub extracted_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// 本文中のページ・章境界。範囲は文字オフセットのhalf-open区間。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBoundary {
    pub start: i64,
    pub end: i64,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ItemTextResponse {
    pub item_id: Uuid,
    pub file_id: Uuid,
    pub extracted_at: NaiveDateTime,
    pub extraction_version: String,
    pub extractor: JsonValue,
    pub chunk: TextChunk,
}

#[derive(Debug, Clone, Serialize)]
pub struct TextChunk {
    pub index: i64,
    pub size: i64,
    pub total_chunks: i64,
    pub label: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ItemTextQuery {
    pub file_id: Option<Uuid>,
    pub chunk_index: Option<i64>,
    pub chunk_size: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct ValidatedItemTextQuery {
    pub file_id: Option<Uuid>,
    pub chunk_index: i64,
    pub chunk_size: i64,
}

pub const DEFAULT_CHUNK_SIZE: i64 = 4_000;
pub const MAX_CHUNK_SIZE: i64 = 20_000;

pub fn parse_item_text_query(query: ItemTextQuery) -> Result<ValidatedItemTextQuery, ApiError> {
    let chunk_index = query.chunk_index.unwrap_or(0);
    if chunk_index < 0 {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "chunk_indexは0以上である必要があります",
        ));
    }

    let chunk_size = query.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE);
    if !(1..=MAX_CHUNK_SIZE).contains(&chunk_size) {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "chunk_sizeは1以上20000以下である必要があります",
        ));
    }

    Ok(ValidatedItemTextQuery {
        file_id: query.file_id,
        chunk_index,
        chunk_size,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct AmbiguousFileCandidate {
    pub file_id: Uuid,
    pub label: Option<String>,
    pub file_type: FileType,
}

/// チャンク範囲と交差する境界から表示用ラベルを合成する。
pub fn compose_chunk_label(
    boundaries: &[TextBoundary],
    chunk_start: i64,
    chunk_end: i64,
) -> Option<String> {
    let mut overlapping = boundaries
        .iter()
        .filter(|boundary| boundary.start < chunk_end && boundary.end > chunk_start);
    let first = overlapping.next()?;
    let last = overlapping.next_back();

    match last {
        None => Some(first.label.clone()),
        Some(last) => {
            let last_label = numeric_suffix(&last.label).unwrap_or(&last.label);
            Some(format!("{}-{}", first.label, last_label))
        }
    }
}

fn numeric_suffix(label: &str) -> Option<&str> {
    let suffix_len = label
        .as_bytes()
        .iter()
        .rev()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    (suffix_len > 0).then(|| &label[label.len() - suffix_len..])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn boundary(start: i64, end: i64, label: &str) -> TextBoundary {
        TextBoundary {
            start,
            end,
            label: label.to_string(),
        }
    }

    #[test]
    fn item_text_query_applies_defaults() {
        let parsed = parse_item_text_query(ItemTextQuery {
            file_id: None,
            chunk_index: None,
            chunk_size: None,
        })
        .unwrap();

        assert_eq!(parsed.chunk_index, 0);
        assert_eq!(parsed.chunk_size, DEFAULT_CHUNK_SIZE);
    }

    #[test]
    fn item_text_query_rejects_invalid_ranges() {
        for (chunk_index, chunk_size) in [(-1, 1), (0, 0), (0, -1), (0, 20_001)] {
            let error = parse_item_text_query(ItemTextQuery {
                file_id: None,
                chunk_index: Some(chunk_index),
                chunk_size: Some(chunk_size),
            })
            .unwrap_err();
            assert_eq!(error.error.code, "VALIDATION_ERROR");
        }
    }

    #[test]
    fn compose_label_for_single_boundary() {
        let boundaries = vec![boundary(11_800, 16_200, "p.9")];
        assert_eq!(
            compose_chunk_label(&boundaries, 12_000, 16_000),
            Some("p.9".to_string())
        );
    }

    #[test]
    fn compose_label_for_multiple_numeric_boundaries() {
        let boundaries = vec![
            boundary(0, 1_200, "p.1"),
            boundary(1_200, 2_900, "p.2"),
            boundary(2_900, 4_500, "p.3"),
        ];
        assert_eq!(
            compose_chunk_label(&boundaries, 0, 4_000),
            Some("p.1-3".to_string())
        );
    }

    #[test]
    fn compose_label_returns_none_without_overlap() {
        assert_eq!(compose_chunk_label(&[], 0, 100), None);
        assert_eq!(
            compose_chunk_label(&[boundary(0, 100, "p.1")], 100, 200),
            None
        );
    }

    #[test]
    fn compose_label_preserves_non_numeric_last_label() {
        let boundaries = vec![boundary(0, 100, "第1章"), boundary(100, 200, "第2章")];
        assert_eq!(
            compose_chunk_label(&boundaries, 0, 200),
            Some("第1章-第2章".to_string())
        );
    }
}
