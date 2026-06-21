/// Jikan アニメ検索リクエスト（GET /anime）。
#[derive(Debug, Clone, Default)]
pub struct JikanAnimeSearchRequest {
    pub q: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
    /// アニメタイプフィルタ（例: "tv", "movie", "ova"）
    pub anime_type: Option<String>,
    /// ステータスフィルタ（例: "airing", "complete"）
    pub status: Option<String>,
}

/// Jikan アニメ詳細取得リクエスト（GET /anime/{id}/full）。
#[derive(Debug, Clone)]
pub struct JikanAnimeDetailsRequest {
    pub id: u32,
}

/// Jikan エピソード一覧取得リクエスト（GET /anime/{id}/episodes）。
#[derive(Debug, Clone)]
pub struct JikanAnimeEpisodesRequest {
    pub id: u32,
    pub page: Option<u32>,
}

/// Jikan 特定エピソード取得リクエスト（GET /anime/{id}/episodes/{episode}）。
#[derive(Debug, Clone)]
pub struct JikanAnimeEpisodeRequest {
    pub id: u32,
    pub episode: u32,
}

/// Jikan スタッフ取得リクエスト（GET /anime/{id}/staff）。
#[derive(Debug, Clone)]
pub struct JikanAnimeStaffRequest {
    pub id: u32,
}

/// Jikan 画像一覧取得リクエスト（GET /anime/{id}/pictures）。
#[derive(Debug, Clone)]
pub struct JikanAnimePicturesRequest {
    pub id: u32,
}

/// Jikan 動画取得リクエスト（GET /anime/{id}/videos）。
#[derive(Debug, Clone)]
pub struct JikanAnimeVideosRequest {
    pub id: u32,
}

/// Jikan 漫画検索リクエスト（GET /manga）。
#[derive(Debug, Clone, Default)]
pub struct JikanMangaSearchRequest {
    pub q: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
    /// タイプフィルタ（例: "manga", "novel", "lightnovel"）
    pub manga_type: Option<String>,
}

/// Jikan 漫画詳細取得リクエスト（GET /manga/{id}/full）。
#[derive(Debug, Clone)]
pub struct JikanMangaDetailsRequest {
    pub id: u32,
}

/// Jikan キャラクター検索リクエスト（GET /characters）。
#[derive(Debug, Clone, Default)]
pub struct JikanCharacterSearchRequest {
    pub q: Option<String>,
    pub page: Option<u32>,
}

/// Jikan プロデューサー検索リクエスト（GET /producers）。
#[derive(Debug, Clone, Default)]
pub struct JikanProducerSearchRequest {
    pub q: Option<String>,
    pub page: Option<u32>,
}

/// Jikan 季節アニメ取得リクエスト（GET /seasons/{year}/{season}）。
#[derive(Debug, Clone)]
pub struct JikanSeasonRequest {
    pub year: u32,
    /// "winter" | "spring" | "summer" | "fall"
    pub season: String,
    pub page: Option<u32>,
}

/// Jikan クライアントへのリクエスト列挙型。
#[derive(Debug, Clone)]
pub enum JikanRequest {
    SearchAnime(JikanAnimeSearchRequest),
    GetAnimeDetails(JikanAnimeDetailsRequest),
    GetAnimeEpisodes(JikanAnimeEpisodesRequest),
    GetAnimeEpisode(JikanAnimeEpisodeRequest),
    GetAnimeStaff(JikanAnimeStaffRequest),
    GetAnimePictures(JikanAnimePicturesRequest),
    GetAnimeVideos(JikanAnimeVideosRequest),
    SearchManga(JikanMangaSearchRequest),
    GetMangaDetails(JikanMangaDetailsRequest),
    SearchCharacters(JikanCharacterSearchRequest),
    SearchProducers(JikanProducerSearchRequest),
    GetSeasonAnime(JikanSeasonRequest),
}
