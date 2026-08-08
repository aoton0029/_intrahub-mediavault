//! Tool層: `relate_items`
//!
//! TASK-0021: relate_items ツールの実装

use uuid::Uuid;

use crate::api::models::RelationType;
use crate::result::outcome::{Outcome, ToolError};
use crate::result::summary::ItemSummary;

/// `relate_items` ツールの引数。
///
/// 🔵 Intent: interfaces.rs `RelateItemsParams`・mcp-tools.md 8. のパラメータ表より。
///    向きの意味はスキーマの外に置かない（タスクファイル「注意事項」）ため、
///    各フィールドのdocコメントに実装項目2の表を埋め込む。
///
/// | `relation_type` | `item_id`（起点） | `related_item_id`（終点） |
/// |---|---|---|
/// | `adaptation` | 原作 | 映像化・翻案作品 |
/// | `sequel` | 前作 | 続編 |
/// | `prequel` | 後の作品 | 前日譚 |
/// | `spinoff` | 本編 | スピンオフ |
/// | `dlc` | 本編 | DLC・追加コンテンツ |
/// | `reference` | 引用元 | 引用先 |
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct RelateItemsParams {
    /// 関係の起点。`relation_type` ごとの意味は上表を参照 🔵 REQ-072
    pub item_id: Uuid,
    /// 関係の終点。`relation_type` ごとの意味は上表を参照 🔵 REQ-072
    pub related_item_id: Uuid,
    /// 固定6種のみ受け付ける 🔵 REQ-071
    pub relation_type: RelationType,
}

/// 🔵 Intent: interfaces.rs `RelateItemsResult` より。
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct RelateItemsResult {
    pub outcome: Outcome,
    pub relation_id: Option<Uuid>,
    pub relation_type: RelationType,
    /// 「A（原作）→ B（映像化）」のように向きを自然文で示す 🔵 REQ-072
    pub description: String,
    pub item: Option<ItemSummary>,
    pub related_item: Option<ItemSummary>,
    /// 409 `DUPLICATE_RELATION` により既存関係を返した場合 true 🔵 REQ-113
    pub already_related: bool,
    pub error: Option<ToolError>,
}

impl RelateItemsResult {
    /// 事前検証・事前取得の段階で終了する場合の共通コンストラクタ。api への書き込みは一切実行していない。
    pub fn early_return(relation_type: RelationType, outcome: Outcome, error: ToolError) -> Self {
        RelateItemsResult {
            outcome,
            relation_id: None,
            relation_type,
            description: String::new(),
            item: None,
            related_item: None,
            already_related: false,
            error: Some(error),
        }
    }
}

/// `relation_type` ごとの起点・終点ラベル。
///
/// 🔵 Intent: 実装項目2の表・実装項目7「種別ごとの起点・終点のラベルを使って組み立てる」より。
pub fn relation_labels(relation_type: RelationType) -> (&'static str, &'static str) {
    match relation_type {
        RelationType::Adaptation => ("原作", "映像化"),
        RelationType::Sequel => ("前作", "続編"),
        RelationType::Prequel => ("後の作品", "前日譚"),
        RelationType::Spinoff => ("本編", "スピンオフ"),
        RelationType::Dlc => ("本編", "DLC"),
        RelationType::Reference => ("引用元", "引用先"),
    }
}

/// 向きを自然文で示す `description` を組み立てる。
///
/// 🔵 Intent: 実装項目7「「小説A」（原作）→「映画A」（映像化）」の形式より。
pub fn build_description(
    relation_type: RelationType,
    item_title: &str,
    related_title: &str,
) -> String {
    let (origin_label, dest_label) = relation_labels(relation_type);
    format!("「{item_title}」（{origin_label}）→「{related_title}」（{dest_label}）")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テストケース1: 6種別すべてデシリアライズでき、範囲外は失敗する（TC-071-01・TC-071-E01）
    #[test]
    fn six_relation_types_deserialize_and_unknown_fails() {
        for raw in [
            "adaptation",
            "sequel",
            "prequel",
            "spinoff",
            "dlc",
            "reference",
        ] {
            let value = serde_json::json!({
                "item_id": "00000000-0000-0000-0000-000000000001",
                "related_item_id": "00000000-0000-0000-0000-000000000002",
                "relation_type": raw,
            });
            let result: Result<RelateItemsParams, _> = serde_json::from_value(value);
            assert!(result.is_ok(), "raw={raw} should deserialize");
        }

        let value = serde_json::json!({
            "item_id": "00000000-0000-0000-0000-000000000001",
            "related_item_id": "00000000-0000-0000-0000-000000000002",
            "relation_type": "inspired_by",
        });
        let result: Result<RelateItemsParams, _> = serde_json::from_value(value);
        assert!(result.is_err());
    }

    /// テストケース3: 6種別それぞれで起点・終点のラベルが正しく入る
    #[test]
    fn description_embeds_direction_labels_for_all_types() {
        let cases = [
            (
                RelationType::Adaptation,
                "「小説A」（原作）→「映画A」（映像化）",
            ),
            (RelationType::Sequel, "「小説A」（前作）→「映画A」（続編）"),
            (
                RelationType::Prequel,
                "「小説A」（後の作品）→「映画A」（前日譚）",
            ),
            (
                RelationType::Spinoff,
                "「小説A」（本編）→「映画A」（スピンオフ）",
            ),
            (RelationType::Dlc, "「小説A」（本編）→「映画A」（DLC）"),
            (
                RelationType::Reference,
                "「小説A」（引用元）→「映画A」（引用先）",
            ),
        ];

        for (relation_type, expected) in cases {
            assert_eq!(build_description(relation_type, "小説A", "映画A"), expected);
        }
    }

    /// early_return は relation_id: None・already_related: false・item/related_item: None
    #[test]
    fn early_return_has_empty_relation_fields() {
        let error = ToolError {
            code: "MCP_INVALID_ARGUMENT".to_string(),
            message: "自己参照はできません".to_string(),
            retriable: false,
        };
        let result =
            RelateItemsResult::early_return(RelationType::Reference, Outcome::Error, error);

        assert!(result.relation_id.is_none());
        assert!(!result.already_related);
        assert!(result.item.is_none());
        assert!(result.related_item.is_none());
        assert!(result.error.is_some());
    }
}
