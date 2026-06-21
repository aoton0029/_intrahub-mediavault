/// メディア種別（ANIME / MANGA）。
#[derive(Debug, Clone, Copy)]
pub enum MediaType {
    Anime,
    Manga,
}

impl MediaType {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            MediaType::Anime => "ANIME",
            MediaType::Manga => "MANGA",
        }
    }
}

/// アニメシーズン。
#[derive(Debug, Clone, Copy)]
pub enum AnimeSeason {
    Winter,
    Spring,
    Summer,
    Fall,
}

impl AnimeSeason {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            AnimeSeason::Winter => "WINTER",
            AnimeSeason::Spring => "SPRING",
            AnimeSeason::Summer => "SUMMER",
            AnimeSeason::Fall => "FALL",
        }
    }
}

/// アニメ・マンガ検索リクエスト。
#[derive(Debug, Clone, Default)]
pub struct MediaSearchRequest {
    pub search: Option<String>,
    pub media_type: Option<MediaType>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub genre_in: Option<Vec<String>>,
    pub season: Option<AnimeSeason>,
    pub season_year: Option<u32>,
}

/// アニメ・マンガ詳細取得リクエスト。
#[derive(Debug, Clone)]
pub struct MediaDetailsRequest {
    pub id: u32,
}

/// ユーザーメディアリスト（視聴履歴）取得リクエスト（Bearer 認証必須）。
#[derive(Debug, Clone)]
pub struct MediaListExportRequest {
    pub user_name: String,
}

/// AniList クライアントへのリクエスト列挙型。
#[derive(Debug, Clone)]
pub enum AniListRequest {
    SearchMedia(MediaSearchRequest),
    GetMediaDetails(MediaDetailsRequest),
    ExportMediaList(MediaListExportRequest),
}
