//! Service層: `search_external_catalog`
//!
//! TASK-0015: search_external_catalog ツールの実装

use crate::api::client::ApiClient;
use crate::api::models::SearchResultItem;
use crate::result::outcome::classify_api_error;
use crate::tools::search_external_catalog::{
    ExternalCandidate, SUMMARY_MAX_CHARS, SearchExternalCatalogParams, SearchExternalCatalogResult,
};

/// `summary` を上限文字数へ切り詰める。切り詰めた場合は末尾に `…` を付ける（実装項目3）。
///
/// 🟡 Intent: `GET /items/search` は現状あらすじを返さないため呼び出し箇所は無いが、
///    タスクファイルのテスト要件（切り詰め仕様）を満たすため独立した関数として用意する。
#[allow(
    dead_code,
    reason = "api が summary/description を返すようになった際に使用する"
)]
fn truncate_summary(text: &str) -> String {
    let char_count = text.chars().count();
    if char_count <= SUMMARY_MAX_CHARS {
        return text.to_string();
    }
    let truncated: String = text.chars().take(SUMMARY_MAX_CHARS).collect();
    format!("{truncated}…")
}

/// `SearchResultItem`(api) を `ExternalCandidate`(MCP) へ変換する。
///
/// 🔵 Intent: `items.md` の `SearchResultItem` は `year` を整数で返すため、
///    `ItemSummary` のような日付文字列からの抽出は不要（そのまま渡す）。
fn to_candidate(item: SearchResultItem) -> ExternalCandidate {
    ExternalCandidate {
        external_id: item.id,
        provider: item.provider,
        title: item.title,
        release_year: item.year,
        media_type: item.media_type,
        cover_image_url: item.thumbnail_url,
        // 🟡 Intent: api が現状あらすじを返さないため常に None。
        summary: None,
    }
}

/// `search_external_catalog` の本体。
///
/// 🔵 Intent: mcp-tools.md 3. のとおり `media_type` / `q` をそのまま `GET /items/search`
///    へ渡す。プロバイダの選択は api 側の責務(REQ-030)。
pub async fn search(
    api: &ApiClient,
    params: SearchExternalCatalogParams,
) -> SearchExternalCatalogResult {
    let query = vec![
        ("media_type", as_query_string(params.media_type)),
        ("q", params.q),
    ];

    match api
        .get_external::<Vec<SearchResultItem>>("/api/v1/items/search", &query)
        .await
    {
        Ok(response) => {
            // 🟡 Intent: api は候補ごとに provider を返すが、media_type から自動選択される
            //    ため実質単一プロバイダになる。トップレベル `provider` は先頭候補から採る。
            let provider = response.data.first().and_then(|item| item.provider.clone());
            let candidates = response.data.into_iter().map(to_candidate).collect();

            SearchExternalCatalogResult {
                outcome: crate::result::outcome::Outcome::Success,
                source: crate::tools::search_external_catalog::SOURCE,
                provider,
                candidates,
                error: None,
            }
        }
        Err(err) => {
            let (outcome, error) = classify_api_error(&err);
            SearchExternalCatalogResult::early_return(outcome, Some(error))
        }
    }
}

/// `#[serde(rename_all = "snake_case")]` な enum をクエリ文字列へ変換する。
fn as_query_string<T: serde::Serialize>(value: T) -> String {
    match serde_json::to_value(value).expect("enum は常にシリアライズ可能") {
        serde_json::Value::String(s) => s,
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::models::MediaType;

    /// テストケース2: summary の切り詰め
    #[test]
    fn truncate_summary_cuts_long_text_and_marks_it() {
        let long_text = "あ".repeat(1000);
        let truncated = truncate_summary(&long_text);

        assert!(truncated.chars().count() <= SUMMARY_MAX_CHARS + 1);
        assert!(truncated.ends_with('…'));
    }

    #[test]
    fn truncate_summary_keeps_short_text_unchanged() {
        let text = "短いあらすじ";
        assert_eq!(truncate_summary(text), text);
    }

    /// テストケース3: release_year の抽出(api は既に整数で返すためそのまま渡る)
    #[test]
    fn to_candidate_passes_through_year() {
        let item = SearchResultItem {
            id: "12345".to_string(),
            media_type: MediaType::Anime,
            provider: Some("annict".to_string()),
            title: "作品A".to_string(),
            thumbnail_url: None,
            year: Some(2023),
        };
        assert_eq!(to_candidate(item).release_year, Some(2023));
    }

    #[test]
    fn to_candidate_keeps_none_year_as_none() {
        let item = SearchResultItem {
            id: "12345".to_string(),
            media_type: MediaType::Anime,
            provider: None,
            title: "作品A".to_string(),
            thumbnail_url: None,
            year: None,
        };
        assert_eq!(to_candidate(item).release_year, None);
    }

    #[test]
    fn as_query_string_converts_snake_case_enum() {
        assert_eq!(as_query_string(MediaType::Anime), "anime");
    }
}
