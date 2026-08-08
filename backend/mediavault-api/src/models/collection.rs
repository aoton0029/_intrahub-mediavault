//! コレクション統計モデル・DTO
//!
//! TASK-0004: GET /api/v1/collection/overview の新設

use serde::{Deserialize, Serialize};

use crate::models::item::ItemWithRefs;
use crate::models::response::{ApiError, ApiErrorCode};

/// `GET /collection/overview` クエリパラメータDTO
#[derive(Debug, Clone, Deserialize)]
pub struct CollectionOverviewQuery {
    pub recent_limit: Option<u32>,
}

/// key/count の集計エントリ（`by_media_type` / `by_status` 共通）
#[derive(Debug, Clone, Serialize)]
pub struct CountEntry {
    pub key: String,
    pub count: i64,
}

/// `GET /collection/overview` レスポンスの`data`
#[derive(Debug, Clone, Serialize)]
pub struct CollectionOverview {
    pub total_items: i64,
    pub favorite_count: i64,
    pub by_media_type: Vec<CountEntry>,
    pub by_status: Vec<CountEntry>,
    pub recently_added: Vec<ItemWithRefs>,
    pub recently_updated: Vec<ItemWithRefs>,
}

/// `list_recent_items` のソート対象カラム切り替え
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecentItemsOrder {
    CreatedAt,
    UpdatedAt,
}

/// 既定値（10件）
const DEFAULT_RECENT_LIMIT: u32 = 10;
/// 上限（50件）
const MAX_RECENT_LIMIT: u32 = 50;

/// 【機能概要】: `recent_limit`クエリパラメータを検証する
/// 【実装方針】: 未指定は既定10、1..=50の範囲外は400 VALIDATION_ERRORとする。
/// `normalize_limit`（GET /items のlimit）とは異なり、範囲外値をクランプせず拒否する
/// （タスク完了条件「範囲外はVALIDATION_ERROR」より）
pub fn validate_recent_limit(recent_limit: Option<u32>) -> Result<u32, ApiError> {
    match recent_limit {
        None => Ok(DEFAULT_RECENT_LIMIT),
        Some(limit) if !(1..=MAX_RECENT_LIMIT).contains(&limit) => Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "recent_limitは1から50の範囲で指定してください",
        )),
        Some(limit) => Ok(limit),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_recent_limit_defaults_to_10_when_unspecified() {
        assert_eq!(validate_recent_limit(None).unwrap(), 10);
    }

    #[test]
    fn validate_recent_limit_accepts_boundary_values() {
        assert_eq!(validate_recent_limit(Some(1)).unwrap(), 1);
        assert_eq!(validate_recent_limit(Some(50)).unwrap(), 50);
    }

    #[test]
    fn validate_recent_limit_rejects_zero() {
        let err = validate_recent_limit(Some(0)).unwrap_err();
        assert_eq!(err.error.code, "VALIDATION_ERROR");
    }

    #[test]
    fn validate_recent_limit_rejects_over_max() {
        let err = validate_recent_limit(Some(51)).unwrap_err();
        assert_eq!(err.error.code, "VALIDATION_ERROR");
    }
}
