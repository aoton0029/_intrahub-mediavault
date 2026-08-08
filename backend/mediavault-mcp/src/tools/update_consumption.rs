//! Tool層: `update_consumption`
//!
//! TASK-0019: update_consumption ツールの実装

use uuid::Uuid;

use crate::api::models::ItemStatus;
use crate::result::outcome::{Outcome, ToolError};

/// `update_consumption` ツールの引数。
///
/// 🔵 Intent: mcp-tools.md 6.・interfaces.rs `UpdateConsumptionParams` より。
///    対象は `item_id`（UUID）のみで、タイトル指定は受け付けない（REQ-142 / 設計決定 D-02）。
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct UpdateConsumptionParams {
    pub item_id: Uuid,
    pub status: Option<ItemStatus>,
    /// `"YYYY-MM-DD"` のみ受け付ける。自然言語の日付はデシリアライズ時点で失敗する（REQ-052）🔵
    pub consumed_date: Option<chrono::NaiveDate>,
    pub rating: Option<f32>,
    pub is_favorite: Option<bool>,
}

/// 更新前後の値の差分。変化がなかったフィールドは含めない 🟡
///
/// 🔵 Intent: interfaces.rs `FieldChange`・REQ-051 より。
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct FieldChange {
    pub field: String,
    pub before: Option<serde_json::Value>,
    pub after: Option<serde_json::Value>,
}

/// 🔵 Intent: interfaces.rs `UpdateConsumptionResult`・REQ-051 より。
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct UpdateConsumptionResult {
    pub outcome: Outcome,
    pub item_id: Uuid,
    pub title: Option<String>,
    pub changes: Vec<FieldChange>,
    pub error: Option<ToolError>,
}

impl UpdateConsumptionResult {
    /// 更新を行わずエラーを返す場合の共通コンストラクタ（`not_found` を含む）。
    pub fn early_return(item_id: Uuid, outcome: Outcome, error: Option<ToolError>) -> Self {
        UpdateConsumptionResult {
            outcome,
            item_id,
            title: None,
            changes: Vec::new(),
            error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テストケース1: 引数スキーマに title がない
    #[test]
    fn params_schema_has_only_item_id_as_target_identifier() {
        let schema = schemars::schema_for!(UpdateConsumptionParams);
        let value = serde_json::to_value(&schema).unwrap();
        let properties = value["properties"].as_object().unwrap();

        assert!(properties.contains_key("item_id"));
        assert!(!properties.contains_key("title"));
        assert!(!properties.contains_key("query"));

        let item_id_schema = &properties["item_id"];
        let item_id_str = serde_json::to_string(item_id_schema).unwrap();
        assert!(item_id_str.contains("uuid"));
    }

    /// テストケース2: 自然言語の日付を拒否
    #[test]
    fn rejects_non_iso_date_strings() {
        for date in ["昨日", "2026/08/06"] {
            let json = serde_json::json!({
                "item_id": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001",
                "consumed_date": date
            });
            let result: Result<UpdateConsumptionParams, _> = serde_json::from_value(json);
            assert!(result.is_err(), "{date} は拒否されるべき");
        }
    }

    /// テストケース3: 不正な status
    #[test]
    fn rejects_invalid_status_value() {
        let json = serde_json::json!({
            "item_id": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001",
            "status": "on_hold"
        });
        let result: Result<UpdateConsumptionParams, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    /// テストケース4: changes の組み立て
    #[test]
    fn result_json_has_five_fields() {
        let result = UpdateConsumptionResult {
            outcome: Outcome::Success,
            item_id: Uuid::nil(),
            title: Some("作品A".to_string()),
            changes: vec![FieldChange {
                field: "status".to_string(),
                before: Some(serde_json::json!("in_progress")),
                after: Some(serde_json::json!("completed")),
            }],
            error: None,
        };
        let value = serde_json::to_value(&result).unwrap();
        assert!(value.get("outcome").is_some());
        assert!(value.get("item_id").is_some());
        assert!(value.get("title").is_some());
        assert!(value.get("changes").is_some());
        assert!(value.get("error").is_some());
        assert_eq!(value["changes"].as_array().unwrap().len(), 1);
    }
}
