/// IGDB ゲームモデル（フラット構造体）。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GameModel {
    pub id: u64,
    pub name: Option<String>,
    pub summary: Option<String>,
    pub rating: Option<f64>,
    pub total_rating: Option<f64>,
    /// Unix タイムスタンプ（秒）
    pub first_release_date: Option<i64>,
    /// ジャンル ID のリスト
    pub genres: Option<Vec<serde_json::Value>>,
    /// カバー画像 image_id
    pub cover: Option<serde_json::Value>,
}

/// IGDB クライアントが返すモデル列挙型。
#[derive(Debug)]
pub enum IgdbModel {
    /// `query_games` の結果
    Games(Vec<GameModel>),
    /// `search` の結果（型が不定のため汎用 Value）
    SearchResults(Vec<serde_json::Value>),
    /// 汎用エンドポイントの結果
    EndpointResults(Vec<serde_json::Value>),
}

// Twitch トークンレスポンス（内部用）
#[derive(serde::Deserialize)]
pub(super) struct TwitchTokenResponse {
    pub access_token: String,
    pub expires_in: u64,
}
