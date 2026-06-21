/// IGDB /games エンドポイント向け Apicalypse クエリリクエスト。
#[derive(Debug, Clone)]
pub struct GamesQueryRequest {
    /// Apicalypse クエリ文字列
    /// 例: `search "Halo"; fields id,name; limit 20;`
    pub query: String,
}

/// IGDB /search エンドポイント向けリクエスト。
#[derive(Debug, Clone)]
pub struct IgdbSearchRequest {
    /// Apicalypse クエリ文字列
    pub query: String,
}

/// IGDB 汎用エンドポイントリクエスト（/companies, /platforms, /covers, /screenshots など）。
#[derive(Debug, Clone)]
pub struct IgdbEndpointRequest {
    /// エンドポイント名（例: "companies", "platforms"）
    pub endpoint: String,
    /// Apicalypse クエリ文字列
    pub query: String,
}

/// IGDB クライアントへのリクエスト列挙型。
#[derive(Debug, Clone)]
pub enum IgdbRequest {
    QueryGames(GamesQueryRequest),
    Search(IgdbSearchRequest),
    QueryEndpoint(IgdbEndpointRequest),
}
