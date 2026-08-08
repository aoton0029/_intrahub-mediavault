//! Service層: `update_consumption`
//!
//! TASK-0019: update_consumption ツールの実装

use chrono::NaiveDate;
use serde::Serialize;

use crate::api::client::ApiClient;
use crate::api::models::{ItemDetail, ItemStatus};
use crate::result::outcome::{Outcome, classify_api_error};
use crate::tools::update_consumption::{
    FieldChange, UpdateConsumptionParams, UpdateConsumptionResult,
};

/// `PATCH /items/{id}` のリクエストボディ。
///
/// 🔵 Intent: `models/item.rs` の `UpdateItemRequest` より。指定されなかった項目は
///    `null` としてでなく、フィールド自体を送らない（統合テスト2）。
#[derive(Debug, Clone, Serialize)]
struct UpdateItemRequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<ItemStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    consumed_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rating: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_favorite: Option<bool>,
}

/// `update_consumption` の本体。
///
/// 🔵 Intent: dataflow.md 機能5 のとおり、GET → PATCH → 差分組み立ての順で行う。
///    GET が 404 なら更新せず `not_found` で終了する（REQ-110）。
///    **書き込みは自動リトライしない**（architecture.md 可用性より）。
pub async fn update(api: &ApiClient, params: UpdateConsumptionParams) -> UpdateConsumptionResult {
    let path = format!("/api/v1/items/{}", params.item_id);

    let before = match api.get::<ItemDetail>(&path, &[]).await {
        Ok(response) => response.data,
        Err(err) => {
            let (outcome, error) = classify_api_error(&err);
            return UpdateConsumptionResult::early_return(params.item_id, outcome, Some(error));
        }
    };

    let body = UpdateItemRequestBody {
        status: params.status,
        consumed_date: params.consumed_date,
        rating: params.rating,
        is_favorite: params.is_favorite,
    };

    match api.patch::<_, ItemDetail>(&path, &body).await {
        Ok(after) => {
            let changes = build_changes(&before, &params, &after);
            UpdateConsumptionResult {
                outcome: Outcome::Success,
                item_id: params.item_id,
                title: Some(after.title.clone()),
                changes,
                error: None,
            }
        }
        Err(err) => {
            let (outcome, error) = classify_api_error(&err);
            UpdateConsumptionResult::early_return(params.item_id, outcome, Some(error))
        }
    }
}

/// 指定された項目のうち、実際に値が変わったものだけを `changes` に含める。
///
/// 🟡 Intent: interfaces.rs の設計上の判断。変化がなかったフィールドは含めない
///    （例: `is_favorite` が既に `true` で `is_favorite: true` を指定した場合）。
fn build_changes(
    before: &ItemDetail,
    params: &UpdateConsumptionParams,
    after: &ItemDetail,
) -> Vec<FieldChange> {
    let mut changes = Vec::new();

    if params.status.is_some() && before.status != after.status {
        changes.push(FieldChange {
            field: "status".to_string(),
            before: serde_json::to_value(before.status).ok(),
            after: serde_json::to_value(after.status).ok(),
        });
    }

    if params.consumed_date.is_some() && before.consumed_date != after.consumed_date {
        changes.push(FieldChange {
            field: "consumed_date".to_string(),
            before: before.consumed_date.map(|d| serde_json::json!(d)),
            after: after.consumed_date.map(|d| serde_json::json!(d)),
        });
    }

    if params.rating.is_some() && before.rating != after.rating {
        changes.push(FieldChange {
            field: "rating".to_string(),
            before: before.rating.map(|r| serde_json::json!(r)),
            after: after.rating.map(|r| serde_json::json!(r)),
        });
    }

    if params.is_favorite.is_some() && before.is_favorite != after.is_favorite {
        changes.push(FieldChange {
            field: "is_favorite".to_string(),
            before: Some(serde_json::json!(before.is_favorite)),
            after: Some(serde_json::json!(after.is_favorite)),
        });
    }

    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テストケース4: changes の組み立て（指定されたが変化しなかったフィールドは含めない）
    #[test]
    fn build_changes_excludes_unchanged_fields() {
        let before = sample_detail(ItemStatus::InProgress, Some(7.0));
        let after = sample_detail(ItemStatus::Completed, Some(7.0));
        let params = UpdateConsumptionParams {
            item_id: before.id,
            status: Some(ItemStatus::Completed),
            consumed_date: None,
            rating: Some(7.0),
            is_favorite: None,
        };

        let changes = build_changes(&before, &params, &after);

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "status");
    }

    fn sample_detail(status: ItemStatus, rating: Option<f32>) -> ItemDetail {
        ItemDetail {
            id: uuid::Uuid::nil(),
            media_type: crate::api::models::MediaType::Anime,
            title: "作品A".to_string(),
            original_title: None,
            description: None,
            release_date: None,
            homepage_url: None,
            status,
            consumed_date: None,
            rating,
            is_favorite: false,
            external_id: None,
            created_at: chrono::NaiveDate::from_ymd_opt(2026, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            updated_at: chrono::NaiveDate::from_ymd_opt(2026, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            detail: None,
            tags: Vec::new(),
            categories: Vec::new(),
            streaming_links: Vec::new(),
        }
    }
}
