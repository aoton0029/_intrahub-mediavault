/// Annict 作品モデル。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct WorkModel {
    pub id: u64,
    pub title: Option<String>,
    pub title_kana: Option<String>,
    pub title_en: Option<String>,
    pub media: Option<String>,
    pub media_text: Option<String>,
    pub released_on: Option<String>,
    pub official_site_url: Option<String>,
    pub wikipedia_url: Option<String>,
    pub twitter_username: Option<String>,
    pub episodes_count: Option<u32>,
    pub watchers_count: Option<u32>,
    pub reviews_count: Option<u32>,
    pub season_name: Option<String>,
    pub season_name_text: Option<String>,
    /// MyAnimeList作品ID（Annict側で紐付けが無い場合は空文字）
    pub mal_anime_id: Option<String>,
    pub images: Option<WorkImagesModel>,
}

/// 作品の画像情報。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct WorkImagesModel {
    pub recommended_url: Option<String>,
}

/// エピソードの前後関係で使う簡略化された作品/エピソード情報。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct EpisodeSummaryModel {
    pub id: u64,
    pub number: Option<i64>,
    pub number_text: Option<String>,
    pub sort_number: Option<i64>,
    pub title: Option<String>,
}

/// Annict エピソードモデル。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct EpisodeModel {
    pub id: u64,
    pub number: Option<i64>,
    pub number_text: Option<String>,
    pub sort_number: Option<i64>,
    pub title: Option<String>,
    pub records_count: Option<u32>,
    pub record_comments_count: Option<u32>,
    pub work: Option<WorkModel>,
    pub prev_episode: Option<EpisodeSummaryModel>,
    pub next_episode: Option<EpisodeSummaryModel>,
}

/// Annict シリーズモデル。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SeriesModel {
    pub id: u64,
    pub name: Option<String>,
    pub name_ro: Option<String>,
    pub name_en: Option<String>,
}

/// Annict キャラクターモデル。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CharacterModel {
    pub id: u64,
    pub name: Option<String>,
    pub name_kana: Option<String>,
    pub name_en: Option<String>,
    pub nickname: Option<String>,
    pub nickname_en: Option<String>,
    pub birthday: Option<String>,
    pub age: Option<String>,
    pub blood_type: Option<String>,
    pub height: Option<String>,
    pub weight: Option<String>,
    pub nationality: Option<String>,
    pub occupation: Option<String>,
    pub description: Option<String>,
    pub description_source: Option<String>,
    pub favorite_characters_count: Option<u32>,
    pub series: Option<SeriesModel>,
}

/// 都道府県情報（人物の出身地）。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PrefectureModel {
    pub id: u64,
    pub name: Option<String>,
}

/// Annict 人物モデル。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PersonModel {
    pub id: u64,
    pub name: Option<String>,
    pub name_kana: Option<String>,
    pub name_en: Option<String>,
    pub nickname: Option<String>,
    pub gender_text: Option<String>,
    pub url: Option<String>,
    pub wikipedia_url: Option<String>,
    pub twitter_username: Option<String>,
    pub birthday: Option<String>,
    pub blood_type: Option<String>,
    pub height: Option<String>,
    pub favorite_people_count: Option<u32>,
    pub casts_count: Option<u32>,
    pub staffs_count: Option<u32>,
    pub prefecture: Option<PrefectureModel>,
}

/// Annict 団体モデル。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OrganizationModel {
    pub id: u64,
    pub name: Option<String>,
    pub name_kana: Option<String>,
    pub name_en: Option<String>,
    pub url: Option<String>,
    pub wikipedia_url: Option<String>,
    pub twitter_username: Option<String>,
    pub favorite_organizations_count: Option<u32>,
    pub staffs_count: Option<u32>,
}

/// Annict キャストモデル（作品・キャラクター・人物の紐付け）。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CastModel {
    pub id: u64,
    pub name: Option<String>,
    pub name_en: Option<String>,
    pub sort_number: Option<i64>,
    pub work: Option<WorkModel>,
    pub character: Option<CharacterModel>,
    pub person: Option<PersonModel>,
}

/// Annict スタッフモデル（作品と人物/団体の紐付け）。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct StaffModel {
    pub id: u64,
    pub name: Option<String>,
    pub name_en: Option<String>,
    pub role_text: Option<String>,
    pub role_other: Option<String>,
    pub role_other_en: Option<String>,
    pub sort_number: Option<i64>,
    pub work: Option<WorkModel>,
    pub person: Option<PersonModel>,
    pub organization: Option<OrganizationModel>,
}

/// Annict クライアントが返すモデル列挙型。
#[derive(Debug)]
pub enum AnnictModel {
    Works(Vec<WorkModel>),
    Episodes(Vec<EpisodeModel>),
    Series(Vec<SeriesModel>),
    Characters(Vec<CharacterModel>),
    People(Vec<PersonModel>),
    Organizations(Vec<OrganizationModel>),
    Casts(Vec<CastModel>),
    Staffs(Vec<StaffModel>),
}

// ── 内部デシリアライズ用ラッパー（ページネーション情報を含む） ──

#[derive(serde::Deserialize)]
pub(super) struct WorksResponse {
    pub works: Vec<WorkModel>,
}

#[derive(serde::Deserialize)]
pub(super) struct EpisodesResponse {
    pub episodes: Vec<EpisodeModel>,
}

#[derive(serde::Deserialize)]
pub(super) struct SeriesResponse {
    pub series: Vec<SeriesModel>,
}

#[derive(serde::Deserialize)]
pub(super) struct CharactersResponse {
    pub characters: Vec<CharacterModel>,
}

#[derive(serde::Deserialize)]
pub(super) struct PeopleResponse {
    pub people: Vec<PersonModel>,
}

#[derive(serde::Deserialize)]
pub(super) struct OrganizationsResponse {
    pub organizations: Vec<OrganizationModel>,
}

#[derive(serde::Deserialize)]
pub(super) struct CastsResponse {
    pub casts: Vec<CastModel>,
}

#[derive(serde::Deserialize)]
pub(super) struct StaffsResponse {
    pub staffs: Vec<StaffModel>,
}
