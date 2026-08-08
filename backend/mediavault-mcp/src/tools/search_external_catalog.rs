//! Tool層: `search_external_catalog`
//!
//! TASK-0015: search_external_catalog ツールの実装

use crate::api::models::MediaType;
use crate::result::outcome::{Outcome, ToolError};

/// `summary` を切り詰める上限文字数(char単位)。
///
/// 🟡 Intent: PRD §11 のレスポンスサイズ方針より。候補20件でのトークン消費を抑える。
pub const SUMMARY_MAX_CHARS: usize = 160;

/// `search_external_catalog` ツールの引数。
///
/// 🔵 Intent: `GET /items/search` の `ItemSearchQuery`（media_type, q ともに必須）より。
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchExternalCatalogParams {
    /// プロバイダの自動選択に使う 🔵
    pub media_type: MediaType,
    /// 検索語 🔵
    pub q: String,
}

/// 外部カタログの検索候補。**MediaVault の `item_id` を持たない**。
///
/// 🔵 Intent: REQ-032・dataflow.md「所蔵品との取り違えを型レベルで防ぐ」より。
///    UUIDフィールドを定義しないことで、所蔵品(`ItemSummary`)との混同を型で防ぐ。
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct ExternalCandidate {
    /// `import_external_item` の `external_id` へそのまま渡す 🔵 REQ-031 / NFR-003
    pub external_id: String,
    /// annict / rakuten / tmdb / steam / ndl 🔵
    pub provider: Option<String>,
    pub title: String,
    pub release_year: Option<i32>,
    pub media_type: MediaType,
    pub cover_image_url: Option<String>,
    /// 候補の区別に役立つ短い補足 🟡 REQ-031
    pub summary: Option<String>,
}

/// 🔵 Intent: interfaces.rs `SearchExternalCatalogResult` より。
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct SearchExternalCatalogResult {
    pub outcome: Outcome,
    /// 検索対象が外部カタログであることを明示する固定値。
    /// `search_library` の `source` と対になる 🔵 REQ-032 / NFR-202
    pub source: &'static str,
    pub provider: Option<String>,
    pub candidates: Vec<ExternalCandidate>,
    pub error: Option<ToolError>,
}

/// `search_external_catalog` の固定 `source` 値。
///
/// 🔵 Intent: REQ-032・`search_library` の `"mediavault_library"` と対になる。
pub const SOURCE: &str = "external_catalog";

impl SearchExternalCatalogResult {
    /// 検索を実行せずエラーを返す場合の共通コンストラクタ。
    pub fn early_return(outcome: Outcome, error: Option<ToolError>) -> Self {
        SearchExternalCatalogResult {
            outcome,
            source: SOURCE,
            provider: None,
            candidates: Vec::new(),
            error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テストケース1: ExternalCandidate に item_id がない
    #[test]
    fn external_candidate_json_has_no_item_id_field() {
        let candidate = ExternalCandidate {
            external_id: "12345".to_string(),
            provider: Some("annict".to_string()),
            title: "作品A".to_string(),
            release_year: Some(2023),
            media_type: MediaType::Anime,
            cover_image_url: None,
            summary: None,
        };

        let value = serde_json::to_value(&candidate).unwrap();
        assert!(value.get("item_id").is_none());
    }
}
