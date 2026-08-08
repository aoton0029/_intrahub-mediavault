//! item_relations（関連付け・DLC）のモデル・リクエストDTO・バリデーション
//!
//! TASK-0017: item_relations CRUD実装（models/mylist.rsと対称な構造）

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::response::{ApiError, ApiErrorCode};

/// 関連付け種別。`item_id`（起点）→ `related_item_id`（終点）の向きが種別ごとに意味を持つ。
///
/// | 値 | 起点 | 終点 |
/// |---|---|---|
/// | `adaptation` | 原作 | 映像化・翻案作品 |
/// | `sequel` | 前作 | 続編 |
/// | `prequel` | 後の作品 | 前日譚 |
/// | `spinoff` | 本編 | スピンオフ |
/// | `dlc` | 本編 | DLC・追加コンテンツ |
/// | `reference` | 引用元 | 引用先 |
///
/// 🔵 信頼性レベル: init_schema.up.sqlのrelation_type ENUM定義に直接対応
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "relation_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum RelationType {
    Adaptation,
    Sequel,
    Prequel,
    Spinoff,
    Dlc,
    Reference,
}

/// item_relations本体（`POST /item-relations`のレスポンスで返す表現）
/// 🔵 信頼性レベル: database-schema.sqlのitem_relationsテーブル定義に直接対応
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemRelation {
    pub id: Uuid,
    pub item_id: Uuid,
    pub related_item_id: Uuid,
    pub relation_type: RelationType,
    pub created_at: NaiveDateTime,
}

/// `POST /item-relations` リクエストDTO
/// 🔵 信頼性レベル: タスク仕様「item_id, related_item_id, relation_typeを受け取る」に直接対応
#[derive(Debug, Clone, Deserialize)]
pub struct CreateItemRelationRequest {
    pub item_id: Uuid,
    pub related_item_id: Uuid,
    pub relation_type: RelationType,
}

/// 【機能概要】: item_id == related_item_idの自己参照を事前に拒否する
/// 【実装方針】: DB側のCHECK制約chk_item_relations_not_selfに対するアプリケーション層の第一防衛線
/// 🔵 信頼性レベル: タスク仕様TC-013-02・chk_item_relations_not_self制約に直接対応
pub fn validate_not_self_reference(request: &CreateItemRelationRequest) -> Result<(), ApiError> {
    if request.item_id == request.related_item_id {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "item_idとrelated_item_idは異なるアイテムを指定してください",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テストケース1: CreateItemRelationRequestの正常デシリアライズ
    #[test]
    fn create_item_relation_request_deserializes_valid_relation_type() {
        let item_id = Uuid::new_v4();
        let related_item_id = Uuid::new_v4();
        let value = serde_json::json!({
            "item_id": item_id,
            "related_item_id": related_item_id,
            "relation_type": "dlc"
        });

        let request: CreateItemRelationRequest = serde_json::from_value(value).unwrap();

        assert_eq!(request.item_id, item_id);
        assert_eq!(request.related_item_id, related_item_id);
        assert_eq!(request.relation_type, RelationType::Dlc);
    }

    /// TASK-0001 テストケース1: 6種別すべてがデシリアライズできる
    #[test]
    fn create_item_relation_request_deserializes_all_six_relation_types() {
        let cases = [
            ("adaptation", RelationType::Adaptation),
            ("sequel", RelationType::Sequel),
            ("prequel", RelationType::Prequel),
            ("spinoff", RelationType::Spinoff),
            ("dlc", RelationType::Dlc),
            ("reference", RelationType::Reference),
        ];

        for (raw, expected) in cases {
            let value = serde_json::json!({
                "item_id": Uuid::new_v4(),
                "related_item_id": Uuid::new_v4(),
                "relation_type": raw
            });

            let request: CreateItemRelationRequest = serde_json::from_value(value)
                .unwrap_or_else(|e| panic!("{raw} のデシリアライズに失敗: {e}"));

            assert_eq!(request.relation_type, expected, "raw={raw}");
        }
    }

    /// TASK-0001: 6種別すべてがENUM文字列と一致する形でシリアライズされる
    #[test]
    fn relation_type_serializes_to_lowercase_enum_labels() {
        let cases = [
            (RelationType::Adaptation, "adaptation"),
            (RelationType::Sequel, "sequel"),
            (RelationType::Prequel, "prequel"),
            (RelationType::Spinoff, "spinoff"),
            (RelationType::Dlc, "dlc"),
            (RelationType::Reference, "reference"),
        ];

        for (variant, expected) in cases {
            assert_eq!(serde_json::to_value(variant).unwrap(), expected);
        }
    }

    /// テストケース3: relation_typeが不正値の場合はデシリアライズエラーになる
    #[test]
    fn create_item_relation_request_with_invalid_relation_type_fails_deserialization() {
        for raw in ["invalid", "inspired_by", "Adaptation", "SEQUEL"] {
            let value = serde_json::json!({
                "item_id": Uuid::new_v4(),
                "related_item_id": Uuid::new_v4(),
                "relation_type": raw
            });

            let result: Result<CreateItemRelationRequest, _> = serde_json::from_value(value);

            assert!(result.is_err(), "{raw} は拒否されるべき");
        }
    }

    /// テストケース2: item_id == related_item_idでVALIDATION_ERRORになる
    #[test]
    fn validate_not_self_reference_rejects_same_ids() {
        let id = Uuid::new_v4();
        let request = CreateItemRelationRequest {
            item_id: id,
            related_item_id: id,
            relation_type: RelationType::Reference,
        };

        let result = validate_not_self_reference(&request);

        assert!(result.is_err());
    }

    /// item_idとrelated_item_idが異なる場合は検証を通過する
    #[test]
    fn validate_not_self_reference_accepts_different_ids() {
        let request = CreateItemRelationRequest {
            item_id: Uuid::new_v4(),
            related_item_id: Uuid::new_v4(),
            relation_type: RelationType::Reference,
        };

        let result = validate_not_self_reference(&request);

        assert!(result.is_ok());
    }
}
