/// OpenBD 一括書誌取得リクエスト（GET /get?isbn=...）。
#[derive(Debug, Clone)]
pub struct OpenBdGetRequest {
    /// ISBN一覧（カンマ区切りでまとめてクエリする）。
    pub isbns: Vec<String>,
}

/// OpenBD クライアントへのリクエスト列挙型。
#[derive(Debug, Clone)]
pub enum OpenBdRequest {
    Get(OpenBdGetRequest),
}
