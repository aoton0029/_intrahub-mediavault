//! Service層: `collection_overview`
//!
//! TASK-0016: collection_overview ツールの実装

use crate::api::client::ApiClient;
use crate::api::models::{
    CollectionOverview as ApiCollectionOverview, CountEntry as ApiCountEntry,
};
use crate::result::outcome::{McpErrorCode, Outcome, ToolError, classify_api_error};
use crate::result::summary::ItemSummary;
use crate::tools::collection_overview::{
    CollectionOverviewParams, CollectionOverviewResult, CountEntry,
};

/// `recent_limit` の下限・上限・既定値。🔵 REQ-143「1..=50、既定10」
const RECENT_LIMIT_MIN: u32 = 1;
const RECENT_LIMIT_MAX: u32 = 50;
const RECENT_LIMIT_DEFAULT: u32 = 10;

fn invalid_argument(message: impl Into<String>) -> ToolError {
    ToolError {
        code: McpErrorCode::InvalidArgument.as_str().to_string(),
        message: message.into(),
        retriable: false,
    }
}

/// `recent_limit` を検証する。**丸めない**（`search_library` の `limit` と同じ規約）。
fn validate_recent_limit(recent_limit: Option<u32>) -> Result<u32, ToolError> {
    match recent_limit {
        None => Ok(RECENT_LIMIT_DEFAULT),
        Some(value) if (RECENT_LIMIT_MIN..=RECENT_LIMIT_MAX).contains(&value) => Ok(value),
        Some(value) => Err(invalid_argument(format!(
            "recent_limit は {RECENT_LIMIT_MIN}..={RECENT_LIMIT_MAX} の範囲で指定してください（指定値: {value}）"
        ))),
    }
}

/// 件数降順に並べ替え、0件のエントリを除外する。
///
/// 🟡 Intent: タスクファイル実装項目3「件数の多い順」「0件の種別は除外」より（`by_media_type` 用）。
fn sorted_nonzero_entries(entries: Vec<ApiCountEntry>) -> Vec<CountEntry> {
    let mut result: Vec<CountEntry> = entries
        .into_iter()
        .filter(|entry| entry.count > 0)
        .map(|entry| CountEntry {
            key: entry.key,
            count: entry.count.max(0) as u64,
        })
        .collect();
    result.sort_by_key(|entry| std::cmp::Reverse(entry.count));
    result
}

/// api の並び順をそのまま保持する。
///
/// 🔵 Intent: 統合テスト2「by_status に3種すべて含まれる」より、0件でも除外しない。
fn preserve_order_entries(entries: Vec<ApiCountEntry>) -> Vec<CountEntry> {
    entries
        .into_iter()
        .map(|entry| CountEntry {
            key: entry.key,
            count: entry.count.max(0) as u64,
        })
        .collect()
}

/// `collection_overview` の本体。
///
/// 🔵 Intent: 実装項目2「単一呼び出しで完結する」より。
pub async fn get_overview(
    api: &ApiClient,
    params: CollectionOverviewParams,
) -> CollectionOverviewResult {
    let recent_limit = match validate_recent_limit(params.recent_limit) {
        Ok(limit) => limit,
        Err(error) => return CollectionOverviewResult::early_return(Outcome::Error, Some(error)),
    };

    let query = vec![("recent_limit", recent_limit.to_string())];

    match api
        .get::<ApiCollectionOverview>("/api/v1/collection/overview", &query)
        .await
    {
        Ok(response) => {
            let data = response.data;
            CollectionOverviewResult {
                outcome: Outcome::Success,
                total_items: data.total_items.max(0) as u64,
                by_media_type: sorted_nonzero_entries(data.by_media_type),
                by_status: preserve_order_entries(data.by_status),
                favorite_count: data.favorite_count.max(0) as u64,
                recently_added: data.recently_added.iter().map(ItemSummary::from).collect(),
                recently_updated: data
                    .recently_updated
                    .iter()
                    .map(ItemSummary::from)
                    .collect(),
                error: None,
            }
        }
        Err(err) => {
            // 🔵 Intent: EDGE-002。api エラーを空の成功結果へフォールバックしない。
            let (outcome, error) = classify_api_error(&err);
            CollectionOverviewResult::early_return(outcome, Some(error))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テストケース1: recent_limit のバリデーション
    #[test]
    fn validate_recent_limit_uses_default_when_unspecified() {
        assert_eq!(validate_recent_limit(None).unwrap(), 10);
    }

    #[test]
    fn validate_recent_limit_accepts_boundary_values() {
        assert_eq!(validate_recent_limit(Some(1)).unwrap(), 1);
        assert_eq!(validate_recent_limit(Some(50)).unwrap(), 50);
    }

    #[test]
    fn validate_recent_limit_rejects_51_without_rounding() {
        let err = validate_recent_limit(Some(51)).unwrap_err();
        assert_eq!(err.code, "MCP_INVALID_ARGUMENT");
    }

    #[test]
    fn validate_recent_limit_rejects_zero() {
        let err = validate_recent_limit(Some(0)).unwrap_err();
        assert_eq!(err.code, "MCP_INVALID_ARGUMENT");
    }

    /// テストケース2: CountEntry への変換（降順・0件除外）
    #[test]
    fn sorted_nonzero_entries_orders_by_count_desc_and_drops_zero() {
        let entries = vec![
            ApiCountEntry {
                key: "anime".to_string(),
                count: 42,
            },
            ApiCountEntry {
                key: "manga".to_string(),
                count: 87,
            },
            ApiCountEntry {
                key: "movie".to_string(),
                count: 0,
            },
        ];

        let result = sorted_nonzero_entries(entries);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].key, "manga");
        assert_eq!(result[0].count, 87);
        assert_eq!(result[1].key, "anime");
        assert_eq!(result[1].count, 42);
    }

    #[test]
    fn preserve_order_entries_keeps_zero_count_entries() {
        let entries = vec![
            ApiCountEntry {
                key: "not_started".to_string(),
                count: 90,
            },
            ApiCountEntry {
                key: "in_progress".to_string(),
                count: 0,
            },
            ApiCountEntry {
                key: "completed".to_string(),
                count: 70,
            },
        ];

        let result = preserve_order_entries(entries);

        assert_eq!(result.len(), 3);
        assert_eq!(result[1].key, "in_progress");
        assert_eq!(result[1].count, 0);
    }
}
