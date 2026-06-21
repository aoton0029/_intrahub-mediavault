/// Steam ユーザー所持ゲーム取得リクエスト。
#[derive(Debug, Clone)]
pub struct GetOwnedGamesRequest {
    /// SteamID64（例: 76561197960435530）
    pub steam_id: u64,
    /// true にするとゲーム名・画像情報が含まれる
    pub include_appinfo: bool,
    /// true にすると無料ゲームも含まれる
    pub include_played_free_games: bool,
}

/// Steam アプリ詳細取得リクエスト（store.steampowered.com/api/appdetails）。
#[derive(Debug, Clone)]
pub struct SteamAppDetailsRequest {
    pub app_id: u32,
}

/// Steam ストア検索リクエスト（store.steampowered.com/api/storesearch）。
#[derive(Debug, Clone)]
pub struct SteamStoreSearchRequest {
    pub term: String,
    pub page: Option<u32>,
}

/// Steam クライアントへのリクエスト列挙型。
#[derive(Debug, Clone)]
pub enum SteamRequest {
    GetOwnedGames(GetOwnedGamesRequest),
    GetAppDetails(SteamAppDetailsRequest),
    StoreSearch(SteamStoreSearchRequest),
}
