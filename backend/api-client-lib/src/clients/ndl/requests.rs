/// NDL OpenSearch 検索リクエスト。
#[derive(Debug, Clone, Default)]
pub struct NdlSearchRequest {
    pub title: Option<String>,
    pub isbn: Option<String>,
    pub creator: Option<String>,
    pub publisher: Option<String>,
    pub any: Option<String>,
    /// 取得件数（デフォルト 20、最大 100）
    pub cnt: Option<u32>,
    /// データプロバイダー ID（例: "jpro-book", "ndl-dl"）
    pub dpid: Option<String>,
}

/// NDL クライアントへのリクエスト列挙型。
#[derive(Debug, Clone)]
pub enum NdlRequest {
    Search(NdlSearchRequest),
}
