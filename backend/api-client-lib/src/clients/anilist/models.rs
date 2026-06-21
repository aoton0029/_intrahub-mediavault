/// アニメ・マンガ メディアモデル（フラット構造体）。
#[derive(Debug, Clone)]
pub struct MediaModel {
    pub id: u32,
    pub title_romaji: Option<String>,
    pub title_english: Option<String>,
    pub title_native: Option<String>,
    pub media_type: Option<String>,
    pub episodes: Option<u32>,
    pub chapters: Option<u32>,
    pub status: Option<String>,
    pub genres: Option<Vec<String>>,
    pub average_score: Option<u32>,
    pub cover_image_large: Option<String>,
    pub season: Option<String>,
    pub season_year: Option<u32>,
}

/// メディアリストエントリ（視聴履歴）。
#[derive(Debug, Clone)]
pub struct MediaListEntryModel {
    pub id: u32,
    pub status: String,
    pub score: Option<f32>,
    pub progress: Option<u32>,
    pub media: MediaModel,
}

/// AniList クライアントが返すモデル列挙型。
#[derive(Debug)]
pub enum AniListModel {
    /// `search_media` の結果
    MediaList(Vec<MediaModel>),
    /// `get_media_details` の結果
    MediaDetail(Box<MediaModel>),
    /// `export_media_list` の結果
    MediaListEntries(Vec<MediaListEntryModel>),
}

// ────────────────────────────────────────────────
// 内部デシリアライズ用 Raw 型（GraphQL ネスト構造）
// ────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub(super) struct GraphQlError {
    pub message: String,
    pub status: Option<u16>,
}

#[derive(serde::Deserialize)]
pub(super) struct TitleRaw {
    pub romaji: Option<String>,
    pub native: Option<String>,
    pub english: Option<String>,
}

#[derive(serde::Deserialize)]
pub(super) struct CoverImageRaw {
    pub large: Option<String>,
}

#[derive(serde::Deserialize)]
pub(super) struct MediaRaw {
    pub id: u32,
    pub title: Option<TitleRaw>,
    #[serde(rename = "coverImage")]
    pub cover_image: Option<CoverImageRaw>,
    pub episodes: Option<u32>,
    pub chapters: Option<u32>,
    pub genres: Option<Vec<String>>,
    pub status: Option<String>,
    #[serde(rename = "averageScore")]
    pub average_score: Option<u32>,
    pub season: Option<String>,
    #[serde(rename = "seasonYear")]
    pub season_year: Option<u32>,
    #[serde(rename = "type")]
    pub media_type: Option<String>,
}

impl From<MediaRaw> for MediaModel {
    fn from(raw: MediaRaw) -> Self {
        MediaModel {
            id: raw.id,
            title_romaji: raw.title.as_ref().and_then(|t| t.romaji.clone()),
            title_english: raw.title.as_ref().and_then(|t| t.english.clone()),
            title_native: raw.title.as_ref().and_then(|t| t.native.clone()),
            media_type: raw.media_type,
            episodes: raw.episodes,
            chapters: raw.chapters,
            status: raw.status,
            genres: raw.genres,
            average_score: raw.average_score,
            cover_image_large: raw.cover_image.and_then(|c| c.large),
            season: raw.season,
            season_year: raw.season_year,
        }
    }
}

// ── Search / Page ──

#[derive(serde::Deserialize)]
pub(super) struct SearchPageData {
    #[serde(rename = "Page")]
    pub page: SearchPage,
}

#[derive(serde::Deserialize)]
pub(super) struct SearchPage {
    pub media: Vec<MediaRaw>,
}

#[derive(serde::Deserialize)]
pub(super) struct SearchResponse {
    pub data: Option<SearchPageData>,
    pub errors: Option<Vec<GraphQlError>>,
}

// ── Details ──

#[derive(serde::Deserialize)]
pub(super) struct DetailsData {
    #[serde(rename = "Media")]
    pub media: MediaRaw,
}

#[derive(serde::Deserialize)]
pub(super) struct DetailsResponse {
    pub data: Option<DetailsData>,
    pub errors: Option<Vec<GraphQlError>>,
}

// ── MediaList ──

#[derive(serde::Deserialize)]
pub(super) struct MediaListEntryRaw {
    pub id: u32,
    pub status: String,
    pub score: Option<f32>,
    pub progress: Option<u32>,
    pub media: MediaRaw,
}

#[derive(serde::Deserialize)]
pub(super) struct MediaListGroup {
    pub entries: Vec<MediaListEntryRaw>,
}

#[derive(serde::Deserialize)]
pub(super) struct MediaListCollection {
    pub lists: Vec<MediaListGroup>,
}

#[derive(serde::Deserialize)]
pub(super) struct MediaListCollectionData {
    #[serde(rename = "MediaListCollection")]
    pub collection: MediaListCollection,
}

#[derive(serde::Deserialize)]
pub(super) struct MediaListResponse {
    pub data: Option<MediaListCollectionData>,
    pub errors: Option<Vec<GraphQlError>>,
}
