/// Open Library 検索結果モデル（/search.json の docs 配列要素）。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OlSearchResultModel {
    pub key: Option<String>,
    pub title: Option<String>,
    pub author_name: Option<Vec<String>>,
    pub first_publish_year: Option<u32>,
    pub isbn: Option<Vec<String>>,
    pub cover_i: Option<u64>,
}

/// Open Library Edition モデル（/isbn/{isbn}.json）。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OlEditionModel {
    pub key: Option<String>,
    pub title: Option<String>,
    pub isbn_13: Option<Vec<String>>,
    pub publishers: Option<Vec<String>>,
    pub publish_date: Option<String>,
    pub number_of_pages: Option<u32>,
    #[serde(default)]
    pub works: Vec<OlWorkRef>,
}

/// Works への参照（Edition に含まれる）。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OlWorkRef {
    pub key: Option<String>,
}

/// Open Library Works モデル（/works/{olid}.json）。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OlWorkModel {
    pub key: Option<String>,
    pub title: Option<String>,
    pub description: Option<serde_json::Value>,
    pub first_publish_date: Option<String>,
    pub authors: Option<Vec<OlAuthorRef>>,
}

/// Authors への参照（Works に含まれる）。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OlAuthorRef {
    pub author: Option<OlKeyRef>,
}

/// 汎用キー参照。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OlKeyRef {
    pub key: Option<String>,
}

/// Open Library クライアントが返すモデル列挙型。
#[derive(Debug)]
pub enum OlModel {
    /// `search` の結果
    SearchResults(Vec<OlSearchResultModel>),
    /// `get_by_isbn` の結果
    Edition(OlEditionModel),
    /// `get_works` の結果
    Works(OlWorkModel),
}

// ── 内部デシリアライズ用ラッパー ──

#[derive(serde::Deserialize)]
pub(super) struct OlSearchResponse {
    pub docs: Vec<OlSearchResultModel>,
}
