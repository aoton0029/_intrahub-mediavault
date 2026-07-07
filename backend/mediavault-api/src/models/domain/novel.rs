//! 小説・書籍のドメインモデル（共通コア + 書誌固有項目）
//!
//! docs/api-samples/openlibrary/isbn.json・search.json・ndl/search_title.xml を根拠とする。

use api_client_lib::clients::ndl::models::NdlItemModel;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::core::{MediaCore, json_str, json_str_array, json_u32};
use crate::models::api_credential::ApiProvider;
use crate::models::item::MediaType;

/// Open Library カバー画像 URL（cover id から生成）
fn openlibrary_cover_url(cover_id: u64) -> String {
    format!("https://covers.openlibrary.org/b/id/{cover_id}-M.jpg")
}

/// 小説・書籍詳細モデル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelDetails {
    #[serde(flatten)]
    pub core: MediaCore,
    #[serde(default)]
    pub authors: Vec<String>,
    pub publisher: Option<String>,
    pub isbn: Option<String>,
    pub page_count: Option<u32>,
    /// 判型（Open Library `physical_format`: "Paperback" 等）
    pub physical_format: Option<String>,
}

impl NovelDetails {
    /// Open Library `GET /isbn/{isbn}.json`（Edition オブジェクト）から構築する。
    pub fn from_openlibrary_edition(data: &Value) -> Self {
        let isbn = data
            .get("isbn_13")
            .and_then(|a| a.get(0))
            .or_else(|| data.get("isbn_10").and_then(|a| a.get(0)))
            .and_then(Value::as_str)
            .map(String::from);
        let core = MediaCore {
            media_type: MediaType::Novel,
            provider: Some(ApiProvider::OpenLibrary),
            external_id: json_str(data, "key").unwrap_or_default(),
            title: json_str(data, "title").unwrap_or_default(),
            original_title: None,
            alternative_titles: Vec::new(),
            description: json_str(data, "description"),
            release_date: json_str(data, "publish_date"),
            image_url: data
                .get("covers")
                .and_then(|c| c.get(0))
                .and_then(Value::as_u64)
                .map(openlibrary_cover_url),
            genres: json_str_array(data, "subjects"),
            rating: None, // Open Library に評価値はない
            url: json_str(data, "key").map(|k| format!("https://openlibrary.org{k}")),
        };
        NovelDetails {
            core,
            authors: Vec::new(), // Edition レスポンスには著者名がない（キー参照のみ）
            publisher: data
                .get("publishers")
                .and_then(|a| a.get(0))
                .and_then(Value::as_str)
                .map(String::from),
            isbn,
            page_count: json_u32(data, "number_of_pages"),
            physical_format: json_str(data, "physical_format"),
        }
    }

    /// Open Library `GET /search.json` の `docs` の1要素から構築する。
    pub fn from_openlibrary_search_doc(doc: &Value) -> Self {
        let core = MediaCore {
            media_type: MediaType::Novel,
            provider: Some(ApiProvider::OpenLibrary),
            external_id: json_str(doc, "key").unwrap_or_default(),
            title: json_str(doc, "title").unwrap_or_default(),
            original_title: None,
            alternative_titles: Vec::new(),
            description: None,
            release_date: doc
                .get("first_publish_year")
                .and_then(Value::as_u64)
                .map(|y| y.to_string()),
            image_url: doc
                .get("cover_i")
                .and_then(Value::as_u64)
                .map(openlibrary_cover_url),
            genres: Vec::new(),
            rating: None,
            url: json_str(doc, "key").map(|k| format!("https://openlibrary.org{k}")),
        };
        NovelDetails {
            core,
            authors: json_str_array(doc, "author_name"),
            publisher: None,
            isbn: doc
                .get("isbn")
                .and_then(|a| a.get(0))
                .and_then(Value::as_str)
                .map(String::from),
            page_count: None,
            physical_format: None,
        }
    }

    /// NDL OpenSearch の書誌アイテム（api-client-lib がパース済み）から構築する。
    ///
    /// NDL は novel / academic_book / paper の共通プロバイダのため、
    /// 検索時の media_type を呼び出し側から受け取る。
    pub fn from_ndl_item(item: &NdlItemModel, media_type: MediaType) -> Self {
        let isbn = item.isbn13.clone().or_else(|| item.isbn.clone());
        let core = MediaCore {
            media_type,
            provider: Some(ApiProvider::Ndl),
            external_id: isbn.clone().unwrap_or_default(),
            title: item.title.clone().unwrap_or_default(),
            original_title: None,
            alternative_titles: Vec::new(),
            description: item.description.clone(),
            release_date: item.pub_date.clone(),
            image_url: item.thumbnail_url.clone(),
            genres: Vec::new(),
            rating: None, // NDL に評価値はない
            url: None,
        };
        NovelDetails {
            core,
            authors: item.creator.clone().unwrap_or_default(),
            publisher: item.publisher.clone(),
            isbn,
            page_count: None,
            physical_format: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_util::load_fixture;
    use super::*;

    #[test]
    fn from_openlibrary_edition_maps_isbn_response() {
        let Some(json) = load_fixture("openlibrary/isbn.json") else {
            eprintln!("fixture missing, skipped");
            return;
        };
        let details = NovelDetails::from_openlibrary_edition(&json);

        assert_eq!(details.core.media_type, MediaType::Novel);
        assert_eq!(details.core.provider, Some(ApiProvider::OpenLibrary));
        assert_eq!(details.core.external_id, "/books/OL33912545M");
        assert_eq!(details.core.title, "The Lord of the Rings");
        assert!(
            details
                .core
                .image_url
                .as_deref()
                .unwrap()
                .starts_with("https://covers.openlibrary.org/")
        );
        assert_eq!(details.publisher.as_deref(), Some("HarperCollins"));
        assert_eq!(details.page_count, Some(1031));
        assert!(details.isbn.is_some());
    }

    #[test]
    fn from_openlibrary_search_doc_maps_first_doc() {
        let Some(json) = load_fixture("openlibrary/search.json") else {
            eprintln!("fixture missing, skipped");
            return;
        };
        let details = NovelDetails::from_openlibrary_search_doc(&json["docs"][0]);
        assert!(!details.core.title.is_empty());
        assert!(!details.core.external_id.is_empty());
        assert!(!details.authors.is_empty());
    }

    #[test]
    fn from_ndl_item_maps_bibliographic_fields() {
        let item = NdlItemModel {
            title: Some("吾輩は猫である".to_string()),
            creator: Some(vec!["夏目漱石".to_string()]),
            publisher: Some("新潮社".to_string()),
            isbn: None,
            isbn13: Some("9784101010014".to_string()),
            pub_date: Some("2003".to_string()),
            description: None,
            thumbnail_url: Some(
                "https://ndlsearch.ndl.go.jp/thumbnail/9784101010014.jpg".to_string(),
            ),
        };
        let details = NovelDetails::from_ndl_item(&item, MediaType::Novel);

        assert_eq!(details.core.media_type, MediaType::Novel);
        assert_eq!(details.core.provider, Some(ApiProvider::Ndl));
        assert_eq!(details.core.external_id, "9784101010014");
        assert_eq!(details.core.title, "吾輩は猫である");
        assert_eq!(details.authors, vec!["夏目漱石".to_string()]);
        assert_eq!(details.publisher.as_deref(), Some("新潮社"));
        assert_eq!(details.isbn.as_deref(), Some("9784101010014"));
    }
}
