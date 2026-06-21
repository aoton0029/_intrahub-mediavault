/// Jikan アニメモデル（フラット構造体）。
#[derive(Debug, Clone)]
pub struct JikanAnimeModel {
    pub mal_id: u32,
    pub title: Option<String>,
    pub title_english: Option<String>,
    pub title_japanese: Option<String>,
    pub episodes: Option<u32>,
    pub status: Option<String>,
    pub score: Option<f64>,
    pub synopsis: Option<String>,
    pub image_url: Option<String>,
    pub genres: Vec<String>,
    pub producers: Vec<String>,
    /// 放送開始日（ISO8601 文字列）
    pub aired_from: Option<String>,
}

/// Jikan エピソードモデル。
#[derive(Debug, Clone)]
pub struct JikanEpisodeModel {
    pub mal_id: u32,
    pub url: Option<String>,
    pub title: Option<String>,
    pub title_japanese: Option<String>,
    pub title_romanji: Option<String>,
    pub aired: Option<String>,
    pub score: Option<f64>,
    pub filler: bool,
    pub recap: bool,
    pub forum_url: Option<String>,
}

/// Jikan エピソード詳細モデル（/anime/{id}/episodes/{episode}）。
#[derive(Debug, Clone)]
pub struct JikanEpisodeDetailModel {
    pub mal_id: u32,
    pub url: Option<String>,
    pub title: Option<String>,
    pub title_japanese: Option<String>,
    pub title_romanji: Option<String>,
    pub duration: Option<u32>,
    pub aired: Option<String>,
    pub filler: bool,
    pub recap: bool,
    pub synopsis: Option<String>,
}

/// Jikan スタッフモデル。
#[derive(Debug, Clone)]
pub struct JikanStaffModel {
    pub mal_id: u32,
    pub name: Option<String>,
    pub image_url: Option<String>,
    pub positions: Vec<String>,
}

/// Jikan 画像モデル。
#[derive(Debug, Clone)]
pub struct JikanPictureModel {
    pub image_url: Option<String>,
}

/// Jikan 動画（プロモ）モデル。
#[derive(Debug, Clone)]
pub struct JikanVideoPromoModel {
    pub title: Option<String>,
    pub youtube_id: Option<String>,
    pub url: Option<String>,
    pub embed_url: Option<String>,
    pub image_url: Option<String>,
}

/// Jikan 動画レスポンス。
#[derive(Debug, Clone)]
pub struct JikanVideosModel {
    pub promo: Vec<JikanVideoPromoModel>,
    pub music_videos: Vec<JikanVideoPromoModel>,
}

/// Jikan 漫画モデル。
#[derive(Debug, Clone)]
pub struct JikanMangaModel {
    pub mal_id: u32,
    pub title: Option<String>,
    pub title_english: Option<String>,
    pub title_japanese: Option<String>,
    pub chapters: Option<u32>,
    pub volumes: Option<u32>,
    pub status: Option<String>,
    pub score: Option<f64>,
    pub synopsis: Option<String>,
    pub image_url: Option<String>,
    pub genres: Vec<String>,
    pub authors: Vec<String>,
}

/// Jikan キャラクターモデル。
#[derive(Debug, Clone)]
pub struct JikanCharacterModel {
    pub mal_id: u32,
    pub name: Option<String>,
    pub image_url: Option<String>,
    pub url: Option<String>,
}

/// Jikan プロデューサーモデル。
#[derive(Debug, Clone)]
pub struct JikanProducerModel {
    pub mal_id: u32,
    pub titles: Vec<String>,
    pub url: Option<String>,
    pub count: Option<u32>,
}

/// Jikan クライアントが返すモデル列挙型。
#[derive(Debug)]
pub enum JikanModel {
    /// `search_anime` の結果
    SearchResults(Vec<JikanAnimeModel>),
    /// `get_anime_details` の結果
    AnimeDetails(JikanAnimeModel),
    /// `get_anime_episodes` の結果
    EpisodeList(Vec<JikanEpisodeModel>),
    /// `get_anime_episode` の結果
    EpisodeDetail(JikanEpisodeDetailModel),
    /// `get_anime_staff` の結果
    StaffList(Vec<JikanStaffModel>),
    /// `get_anime_pictures` の結果
    PictureList(Vec<JikanPictureModel>),
    /// `get_anime_videos` の結果
    Videos(JikanVideosModel),
    /// `search_manga` の結果
    MangaSearchResults(Vec<JikanMangaModel>),
    /// `get_manga_details` の結果
    MangaDetails(JikanMangaModel),
    /// `search_characters` の結果
    CharacterSearchResults(Vec<JikanCharacterModel>),
    /// `search_producers` の結果
    ProducerSearchResults(Vec<JikanProducerModel>),
    /// `get_season_anime` の結果
    SeasonAnime(Vec<JikanAnimeModel>),
}

// ── 内部デシリアライズ用 Raw 型 ──────────────────────────────────────────

#[derive(serde::Deserialize)]
pub(super) struct JikanAiredRaw {
    pub from: Option<String>,
}

#[derive(serde::Deserialize)]
pub(super) struct JikanAnimeRaw {
    pub mal_id: u32,
    pub title: Option<String>,
    pub title_english: Option<String>,
    pub title_japanese: Option<String>,
    pub episodes: Option<u32>,
    pub status: Option<String>,
    pub score: Option<f64>,
    pub synopsis: Option<String>,
    pub images: Option<JikanImagesRaw>,
    #[serde(default)]
    pub genres: Vec<JikanNamedEntityRaw>,
    #[serde(default)]
    pub producers: Vec<JikanNamedEntityRaw>,
    pub aired: Option<JikanAiredRaw>,
}

#[derive(serde::Deserialize)]
pub(super) struct JikanImagesRaw {
    pub jpg: Option<JikanImageRaw>,
}

#[derive(serde::Deserialize)]
pub(super) struct JikanImageRaw {
    pub image_url: Option<String>,
}

#[derive(serde::Deserialize)]
pub(super) struct JikanNamedEntityRaw {
    #[allow(dead_code)]
    pub mal_id: Option<u32>,
    pub name: Option<String>,
}

impl From<JikanAnimeRaw> for JikanAnimeModel {
    fn from(raw: JikanAnimeRaw) -> Self {
        JikanAnimeModel {
            mal_id: raw.mal_id,
            title: raw.title,
            title_english: raw.title_english,
            title_japanese: raw.title_japanese,
            episodes: raw.episodes,
            status: raw.status,
            score: raw.score,
            synopsis: raw.synopsis,
            image_url: raw.images.and_then(|i| i.jpg).and_then(|j| j.image_url),
            genres: raw.genres.into_iter().filter_map(|g| g.name).collect(),
            producers: raw.producers.into_iter().filter_map(|p| p.name).collect(),
            aired_from: raw.aired.and_then(|a| a.from),
        }
    }
}

#[derive(serde::Deserialize)]
pub(super) struct JikanListResponse {
    pub data: Vec<JikanAnimeRaw>,
}

#[derive(serde::Deserialize)]
pub(super) struct JikanSingleResponse {
    pub data: JikanAnimeRaw,
}

// ── Episode ──

#[derive(serde::Deserialize)]
pub(super) struct JikanEpisodeRaw {
    pub mal_id: u32,
    pub url: Option<String>,
    pub title: Option<String>,
    pub title_japanese: Option<String>,
    pub title_romanji: Option<String>,
    pub aired: Option<String>,
    pub score: Option<f64>,
    #[serde(default)]
    pub filler: bool,
    #[serde(default)]
    pub recap: bool,
    pub forum_url: Option<String>,
}

impl From<JikanEpisodeRaw> for JikanEpisodeModel {
    fn from(r: JikanEpisodeRaw) -> Self {
        JikanEpisodeModel {
            mal_id: r.mal_id,
            url: r.url,
            title: r.title,
            title_japanese: r.title_japanese,
            title_romanji: r.title_romanji,
            aired: r.aired,
            score: r.score,
            filler: r.filler,
            recap: r.recap,
            forum_url: r.forum_url,
        }
    }
}

#[derive(serde::Deserialize)]
pub(super) struct JikanEpisodeListResponse {
    pub data: Vec<JikanEpisodeRaw>,
}

#[derive(serde::Deserialize)]
pub(super) struct JikanEpisodeDetailRaw {
    pub mal_id: u32,
    pub url: Option<String>,
    pub title: Option<String>,
    pub title_japanese: Option<String>,
    pub title_romanji: Option<String>,
    pub duration: Option<u32>,
    pub aired: Option<String>,
    #[serde(default)]
    pub filler: bool,
    #[serde(default)]
    pub recap: bool,
    pub synopsis: Option<String>,
}

impl From<JikanEpisodeDetailRaw> for JikanEpisodeDetailModel {
    fn from(r: JikanEpisodeDetailRaw) -> Self {
        JikanEpisodeDetailModel {
            mal_id: r.mal_id,
            url: r.url,
            title: r.title,
            title_japanese: r.title_japanese,
            title_romanji: r.title_romanji,
            duration: r.duration,
            aired: r.aired,
            filler: r.filler,
            recap: r.recap,
            synopsis: r.synopsis,
        }
    }
}

#[derive(serde::Deserialize)]
pub(super) struct JikanEpisodeDetailResponse {
    pub data: JikanEpisodeDetailRaw,
}

// ── Staff ──

#[derive(serde::Deserialize)]
pub(super) struct JikanStaffPersonRaw {
    pub mal_id: u32,
    pub name: Option<String>,
    pub images: Option<JikanImagesRaw>,
}

#[derive(serde::Deserialize)]
pub(super) struct JikanStaffRaw {
    pub person: JikanStaffPersonRaw,
    #[serde(default)]
    pub positions: Vec<String>,
}

impl From<JikanStaffRaw> for JikanStaffModel {
    fn from(r: JikanStaffRaw) -> Self {
        JikanStaffModel {
            mal_id: r.person.mal_id,
            name: r.person.name,
            image_url: r.person.images.and_then(|i| i.jpg).and_then(|j| j.image_url),
            positions: r.positions,
        }
    }
}

#[derive(serde::Deserialize)]
pub(super) struct JikanStaffListResponse {
    pub data: Vec<JikanStaffRaw>,
}

// ── Pictures ──

#[derive(serde::Deserialize)]
pub(super) struct JikanPictureRaw {
    pub images: Option<JikanImagesRaw>,
}

impl From<JikanPictureRaw> for JikanPictureModel {
    fn from(r: JikanPictureRaw) -> Self {
        JikanPictureModel {
            image_url: r.images.and_then(|i| i.jpg).and_then(|j| j.image_url),
        }
    }
}

#[derive(serde::Deserialize)]
pub(super) struct JikanPictureListResponse {
    pub data: Vec<JikanPictureRaw>,
}

// ── Videos ──

#[derive(serde::Deserialize)]
pub(super) struct JikanTrailerRaw {
    pub youtube_id: Option<String>,
    pub url: Option<String>,
    pub embed_url: Option<String>,
    pub images: Option<JikanVideoImagesRaw>,
}

#[derive(serde::Deserialize)]
pub(super) struct JikanVideoImagesRaw {
    pub image_url: Option<String>,
}

#[derive(serde::Deserialize)]
pub(super) struct JikanVideoPromoRaw {
    pub title: Option<String>,
    pub trailer: Option<JikanTrailerRaw>,
}

#[derive(serde::Deserialize)]
pub(super) struct JikanMusicVideoMetaRaw {
    pub title: Option<String>,
}

#[derive(serde::Deserialize)]
pub(super) struct JikanMusicVideoRaw {
    pub title: Option<String>,
    pub video: Option<JikanTrailerRaw>,
    pub meta: Option<JikanMusicVideoMetaRaw>,
}

#[derive(serde::Deserialize)]
pub(super) struct JikanVideosDataRaw {
    #[serde(default)]
    pub promo: Vec<JikanVideoPromoRaw>,
    #[serde(default)]
    pub music_videos: Vec<JikanMusicVideoRaw>,
}

#[derive(serde::Deserialize)]
pub(super) struct JikanVideosResponse {
    pub data: JikanVideosDataRaw,
}

fn trailer_to_promo(title: Option<String>, trailer: Option<JikanTrailerRaw>) -> JikanVideoPromoModel {
    let (youtube_id, url, embed_url, image_url) = trailer
        .map(|t| (t.youtube_id, t.url, t.embed_url, t.images.and_then(|i| i.image_url)))
        .unwrap_or_default();
    JikanVideoPromoModel { title, youtube_id, url, embed_url, image_url }
}

impl From<JikanVideosDataRaw> for JikanVideosModel {
    fn from(r: JikanVideosDataRaw) -> Self {
        JikanVideosModel {
            promo: r.promo.into_iter().map(|p| trailer_to_promo(p.title, p.trailer)).collect(),
            music_videos: r.music_videos.into_iter().map(|mv| {
                let title = mv.meta.and_then(|m| m.title).or(mv.title);
                trailer_to_promo(title, mv.video)
            }).collect(),
        }
    }
}

// ── Manga ──

#[derive(serde::Deserialize)]
pub(super) struct JikanMangaRaw {
    pub mal_id: u32,
    pub title: Option<String>,
    pub title_english: Option<String>,
    pub title_japanese: Option<String>,
    pub chapters: Option<u32>,
    pub volumes: Option<u32>,
    pub status: Option<String>,
    pub score: Option<f64>,
    pub synopsis: Option<String>,
    pub images: Option<JikanImagesRaw>,
    #[serde(default)]
    pub genres: Vec<JikanNamedEntityRaw>,
    #[serde(default)]
    pub authors: Vec<JikanNamedEntityRaw>,
}

impl From<JikanMangaRaw> for JikanMangaModel {
    fn from(r: JikanMangaRaw) -> Self {
        JikanMangaModel {
            mal_id: r.mal_id,
            title: r.title,
            title_english: r.title_english,
            title_japanese: r.title_japanese,
            chapters: r.chapters,
            volumes: r.volumes,
            status: r.status,
            score: r.score,
            synopsis: r.synopsis,
            image_url: r.images.and_then(|i| i.jpg).and_then(|j| j.image_url),
            genres: r.genres.into_iter().filter_map(|g| g.name).collect(),
            authors: r.authors.into_iter().filter_map(|a| a.name).collect(),
        }
    }
}

#[derive(serde::Deserialize)]
pub(super) struct JikanMangaListResponse {
    pub data: Vec<JikanMangaRaw>,
}

#[derive(serde::Deserialize)]
pub(super) struct JikanMangaSingleResponse {
    pub data: JikanMangaRaw,
}

// ── Characters ──

#[derive(serde::Deserialize)]
pub(super) struct JikanCharacterRaw {
    pub mal_id: u32,
    pub name: Option<String>,
    pub url: Option<String>,
    pub images: Option<JikanImagesRaw>,
}

impl From<JikanCharacterRaw> for JikanCharacterModel {
    fn from(r: JikanCharacterRaw) -> Self {
        JikanCharacterModel {
            mal_id: r.mal_id,
            name: r.name,
            image_url: r.images.and_then(|i| i.jpg).and_then(|j| j.image_url),
            url: r.url,
        }
    }
}

#[derive(serde::Deserialize)]
pub(super) struct JikanCharacterListResponse {
    pub data: Vec<JikanCharacterRaw>,
}

// ── Producers ──

#[derive(serde::Deserialize)]
pub(super) struct JikanProducerTitleRaw {
    pub title: Option<String>,
}

#[derive(serde::Deserialize)]
pub(super) struct JikanProducerRaw {
    pub mal_id: u32,
    #[serde(default)]
    pub titles: Vec<JikanProducerTitleRaw>,
    pub url: Option<String>,
    pub count: Option<u32>,
}

impl From<JikanProducerRaw> for JikanProducerModel {
    fn from(r: JikanProducerRaw) -> Self {
        JikanProducerModel {
            mal_id: r.mal_id,
            titles: r.titles.into_iter().filter_map(|t| t.title).collect(),
            url: r.url,
            count: r.count,
        }
    }
}

#[derive(serde::Deserialize)]
pub(super) struct JikanProducerListResponse {
    pub data: Vec<JikanProducerRaw>,
}
