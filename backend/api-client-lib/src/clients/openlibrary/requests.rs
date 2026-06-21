/// Open Library 全文検索リクエスト（GET /search.json）。
#[derive(Debug, Clone)]
pub struct OlSearchRequest {
    pub q: String,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

/// Open Library ISBN 取得リクエスト（GET /isbn/{isbn}.json）。
#[derive(Debug, Clone)]
pub struct OlIsbnRequest {
    pub isbn: String,
}

/// Open Library Works 取得リクエスト（GET /works/{olid}.json）。
#[derive(Debug, Clone)]
pub struct OlWorksRequest {
    /// Works ID（例: "OL45804W"）
    pub olid: String,
}

/// Open Library クライアントへのリクエスト列挙型。
#[derive(Debug, Clone)]
pub enum OlRequest {
    Search(OlSearchRequest),
    GetByIsbn(OlIsbnRequest),
    GetWorks(OlWorksRequest),
}
