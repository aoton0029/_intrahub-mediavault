/// Steam ゲームエントリ（所持ゲームリスト内）。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SteamGameEntry {
    pub appid: u32,
    pub name: Option<String>,
    /// プレイ時間（分）
    pub playtime_forever: u32,
    pub img_icon_url: Option<String>,
}

/// Steam 所持ゲームモデル。
#[derive(Debug, Clone)]
pub struct SteamOwnedGamesModel {
    pub game_count: u32,
    pub games: Vec<SteamGameEntry>,
}

/// Steam アプリ詳細モデル。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SteamAppDetailsModel {
    #[serde(rename = "type")]
    pub app_type: Option<String>,
    pub name: Option<String>,
    pub steam_appid: Option<u32>,
    pub is_free: Option<bool>,
    pub short_description: Option<String>,
    pub header_image: Option<String>,
}

/// Steam ストア検索結果モデル。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SteamStoreSearchResultModel {
    pub id: Option<u64>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub result_type: Option<String>,
    pub tiny_image: Option<String>,
}

/// Steam クライアントが返すモデル列挙型。
#[derive(Debug)]
pub enum SteamModel {
    /// `get_owned_games` の結果
    OwnedGames(SteamOwnedGamesModel),
    /// `get_app_details` の結果
    AppDetails(SteamAppDetailsModel),
    /// `store_search` の結果
    StoreResults(Vec<SteamStoreSearchResultModel>),
}

// ── 内部デシリアライズ用ラッパー ──

#[derive(serde::Deserialize)]
pub(super) struct GetOwnedGamesWrapper {
    pub response: GetOwnedGamesInner,
}

#[derive(serde::Deserialize)]
pub(super) struct GetOwnedGamesInner {
    pub game_count: u32,
    #[serde(default)]
    pub games: Vec<SteamGameEntry>,
}

#[derive(serde::Deserialize)]
pub(super) struct StoreSearchWrapper {
    #[allow(dead_code)]
    pub total: Option<u64>,
    // 【修正】: Steam Store Search APIの実レスポンスはキー名が`items`であり`results`ではないため、
    // `#[serde(default)]`によりデシリアライズが常に空配列にフォールバックしていた（TASK-0031で未検証のまま追加）。
    // `rename`で実際のAPIキー名に合わせる 🔵 実APIレスポンス構造の実測に基づく修正
    #[serde(default, rename = "items")]
    pub results: Vec<SteamStoreSearchResultModel>,
}

/// `/api/appdetails` は `{"<appid>": {"success": bool, "data": {...}}}` 形式
#[derive(serde::Deserialize)]
pub(super) struct AppDetailsEntry {
    pub success: bool,
    pub data: Option<SteamAppDetailsModel>,
}
