//! アニメのドメインモデル（共通コア + アニメ固有項目）
//!
//! docs/api-samples/jikan/anime_details.json・anilist/media_details.json を根拠とする。

use api_client_lib::clients::annict::models::WorkModel;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::core::{
    MediaCore, json_f64, json_names, json_str, json_str_array, json_u32, normalize_score_100,
};
use crate::models::api_credential::ApiProvider;
use crate::models::item::MediaType;

/// 空文字列をNone扱いにする（Annictは未設定項目を`""`で返すため）
fn non_empty(s: Option<String>) -> Option<String> {
    s.filter(|v| !v.trim().is_empty())
}

/// アニメ詳細モデル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeDetails {
    #[serde(flatten)]
    pub core: MediaCore,
    pub episodes: Option<u32>,
    /// 放送ステータス（"Finished Airing" / "FINISHED" 等、プロバイダ表記のまま）
    pub status: Option<String>,
    /// 放送シーズン（"spring" / "SPRING" 等）
    pub season: Option<String>,
    pub year: Option<u32>,
    #[serde(default)]
    pub studios: Vec<String>,
    /// 原作種別（Jikan `source`: "Visual novel" 等）
    pub source: Option<String>,
    /// 1話あたりの長さ（Jikan `duration`: "24 min per ep" 等）
    pub duration: Option<String>,
    pub trailer_url: Option<String>,
}

impl AnimeDetails {
    /// Jikan `GET /anime/{id}/full` の `data` オブジェクトから構築する。
    pub fn from_jikan_details(data: &Value) -> Self {
        let core = MediaCore {
            media_type: MediaType::Anime,
            provider: None, // Jikan はキー不要のため ApiProvider 対象外
            external_id: data
                .get("mal_id")
                .and_then(Value::as_u64)
                .map(|id| id.to_string())
                .unwrap_or_default(),
            title: json_str(data, "title").unwrap_or_default(),
            original_title: json_str(data, "title_japanese"),
            alternative_titles: json_str(data, "title_english").into_iter().collect(),
            description: json_str(data, "synopsis"),
            release_date: data
                .get("aired")
                .and_then(|a| json_str(a, "from"))
                .map(|d| d.chars().take(10).collect()),
            image_url: data
                .get("images")
                .and_then(|i| i.get("jpg"))
                .and_then(|j| json_str(j, "image_url")),
            genres: json_names(data, "genres", "name"),
            rating: json_f64(data, "score"),
            url: json_str(data, "url"),
        };
        AnimeDetails {
            core,
            episodes: json_u32(data, "episodes"),
            status: json_str(data, "status"),
            season: json_str(data, "season"),
            year: json_u32(data, "year"),
            studios: json_names(data, "studios", "name"),
            source: json_str(data, "source"),
            duration: json_str(data, "duration"),
            trailer_url: data.get("trailer").and_then(|t| json_str(t, "url")),
        }
    }

    /// AniList GraphQL の `Media` オブジェクトから構築する。
    pub fn from_anilist_media(media: &Value) -> Self {
        let title = media.get("title").cloned().unwrap_or(Value::Null);
        let core = MediaCore {
            media_type: MediaType::Anime,
            provider: Some(ApiProvider::AniList),
            external_id: media
                .get("id")
                .and_then(Value::as_u64)
                .map(|id| id.to_string())
                .unwrap_or_default(),
            title: json_str(&title, "romaji")
                .or_else(|| json_str(&title, "english"))
                .or_else(|| json_str(&title, "native"))
                .unwrap_or_default(),
            original_title: json_str(&title, "native"),
            alternative_titles: json_str(&title, "english").into_iter().collect(),
            description: json_str(media, "description"),
            release_date: None, // 取得クエリに startDate を含めていない
            image_url: media.get("coverImage").and_then(|c| json_str(c, "large")),
            genres: json_str_array(media, "genres"),
            rating: json_f64(media, "averageScore").map(normalize_score_100),
            url: None,
        };
        AnimeDetails {
            core,
            episodes: json_u32(media, "episodes"),
            status: json_str(media, "status"),
            season: json_str(media, "season"),
            year: json_u32(media, "seasonYear"),
            studios: Vec::new(),
            source: None,
            duration: None,
            trailer_url: None,
        }
    }

    /// Annict `GET /v1/works` の`WorkModel`から構築する（検索結果用の軽量マッピング）。
    ///
    /// id/title/images.recommended_urlのみを保持し、あらすじ等の詳細はインポート確定時に
    /// [`AnimeDetails::from_annict_and_jikan`]で補完する。
    pub fn from_annict_work(work: &WorkModel) -> Self {
        let core = MediaCore {
            media_type: MediaType::Anime,
            provider: Some(ApiProvider::Annict),
            external_id: work.id.to_string(),
            title: non_empty(work.title.clone()).unwrap_or_default(),
            original_title: None,
            alternative_titles: Vec::new(),
            description: None,
            release_date: None,
            image_url: work
                .images
                .as_ref()
                .and_then(|i| non_empty(i.recommended_url.clone())),
            genres: Vec::new(),
            rating: None,
            url: None,
        };
        AnimeDetails {
            core,
            episodes: None,
            status: None,
            season: None,
            year: None,
            studios: Vec::new(),
            source: None,
            duration: None,
            trailer_url: None,
        }
    }

    /// AnnictとJikanの情報をマージして構築する（インポート確定時用）。
    ///
    /// id/title/画像/話数等の作品識別・掲載情報はAnnict優先、あらすじ・ジャンル・評価・
    /// スタジオ等の詳細はJikan（`GET /anime/{id}/full`の`data`オブジェクト）由来とする。
    /// Annict側が空の場合はJikanの値でフォールバックする。
    pub fn from_annict_and_jikan(work: &WorkModel, jikan_data: &Value) -> Self {
        let jikan = AnimeDetails::from_jikan_details(jikan_data);

        let title = non_empty(work.title.clone()).unwrap_or(jikan.core.title);
        let image_url = work
            .images
            .as_ref()
            .and_then(|i| non_empty(i.recommended_url.clone()))
            .or(jikan.core.image_url);
        let episodes = work.episodes_count.or(jikan.episodes);
        let season = non_empty(work.season_name.clone()).or(jikan.season);

        let core = MediaCore {
            media_type: MediaType::Anime,
            provider: Some(ApiProvider::Annict),
            external_id: work.id.to_string(),
            title,
            original_title: jikan.core.original_title,
            alternative_titles: jikan.core.alternative_titles,
            description: jikan.core.description,
            release_date: non_empty(work.released_on.clone()).or(jikan.core.release_date),
            image_url,
            genres: jikan.core.genres,
            rating: jikan.core.rating,
            url: non_empty(work.official_site_url.clone()).or(jikan.core.url),
        };
        AnimeDetails {
            core,
            episodes,
            status: jikan.status,
            season,
            year: jikan.year,
            studios: jikan.studios,
            source: jikan.source,
            duration: jikan.duration,
            trailer_url: jikan.trailer_url,
        }
    }
}

#[cfg(test)]
mod tests {
    use api_client_lib::clients::annict::models::WorkImagesModel;

    use super::super::test_util::load_fixture;
    use super::*;

    #[test]
    fn from_jikan_details_maps_core_and_extension_fields() {
        let Some(json) = load_fixture("jikan/anime_details.json") else {
            eprintln!("fixture missing, skipped");
            return;
        };
        let details = AnimeDetails::from_jikan_details(&json["data"]);

        assert_eq!(details.core.media_type, MediaType::Anime);
        assert_eq!(details.core.provider, None);
        assert_eq!(details.core.external_id, "9253");
        assert!(!details.core.title.is_empty());
        assert_eq!(details.core.original_title.as_deref(), Some("STEINS;GATE"));
        assert!(
            details
                .core
                .image_url
                .as_deref()
                .unwrap()
                .starts_with("https://")
        );
        assert!(details.core.genres.contains(&"Sci-Fi".to_string()));
        let rating = details.core.rating.unwrap();
        assert!((0.0..=10.0).contains(&rating));
        assert_eq!(details.core.release_date.as_deref(), Some("2011-04-06"));
        assert_eq!(details.episodes, Some(24));
        assert!(details.status.is_some());
    }

    #[test]
    fn from_anilist_media_maps_core_and_extension_fields() {
        let Some(json) = load_fixture("anilist/media_details.json") else {
            eprintln!("fixture missing, skipped");
            return;
        };
        let details = AnimeDetails::from_anilist_media(&json["data"]["Media"]);

        assert_eq!(details.core.provider, Some(ApiProvider::AniList));
        assert_eq!(details.core.external_id, "9253");
        assert_eq!(details.core.title, "Steins;Gate");
        assert!((0.0..=10.0).contains(&details.core.rating.unwrap()));
        assert_eq!(details.episodes, Some(24));
        assert_eq!(details.season.as_deref(), Some("SPRING"));
        assert_eq!(details.year, Some(2011));
    }

    fn sample_work() -> WorkModel {
        WorkModel {
            id: 6607,
            title: Some("メイドインアビス 深き魂の黎明".to_string()),
            title_kana: None,
            title_en: Some("Made in Abyss Movie 3".to_string()),
            media: Some("movie".to_string()),
            media_text: None,
            released_on: Some("2020-01-17".to_string()),
            official_site_url: Some("http://miabyss.com/movie/index.html#1".to_string()),
            wikipedia_url: None,
            twitter_username: None,
            episodes_count: Some(1),
            watchers_count: None,
            reviews_count: None,
            season_name: Some("2020-winter".to_string()),
            season_name_text: None,
            mal_anime_id: Some("36862".to_string()),
            images: Some(WorkImagesModel {
                recommended_url: Some("http://miabyss.com/images/ogp.jpg".to_string()),
            }),
        }
    }

    #[test]
    fn from_annict_work_maps_lightweight_search_fields_only() {
        let work = sample_work();
        let details = AnimeDetails::from_annict_work(&work);

        assert_eq!(details.core.media_type, MediaType::Anime);
        assert_eq!(details.core.provider, Some(ApiProvider::Annict));
        assert_eq!(details.core.external_id, "6607");
        assert_eq!(details.core.title, "メイドインアビス 深き魂の黎明");
        assert_eq!(
            details.core.image_url.as_deref(),
            Some("http://miabyss.com/images/ogp.jpg")
        );
        // 検索結果はid/title/imageのみ保持し、詳細項目は空のままであることを確認する
        assert!(details.core.description.is_none());
        assert!(details.core.genres.is_empty());
        assert!(details.core.rating.is_none());
    }

    #[test]
    fn from_annict_work_treats_empty_strings_as_none() {
        let mut work = sample_work();
        work.images = Some(WorkImagesModel {
            recommended_url: Some(String::new()),
        });
        let details = AnimeDetails::from_annict_work(&work);
        assert!(details.core.image_url.is_none());
    }

    #[test]
    fn from_annict_and_jikan_prefers_annict_for_identity_fields_and_jikan_for_details() {
        let Some(json) = load_fixture("jikan/anime_details.json") else {
            eprintln!("fixture missing, skipped");
            return;
        };
        let work = sample_work();
        let details = AnimeDetails::from_annict_and_jikan(&work, &json["data"]);

        // Annict優先: id/title/画像/話数/シーズン
        assert_eq!(details.core.provider, Some(ApiProvider::Annict));
        assert_eq!(details.core.external_id, "6607");
        assert_eq!(details.core.title, "メイドインアビス 深き魂の黎明");
        assert_eq!(
            details.core.image_url.as_deref(),
            Some("http://miabyss.com/images/ogp.jpg")
        );
        assert_eq!(details.episodes, Some(1));
        assert_eq!(details.season.as_deref(), Some("2020-winter"));

        // Jikan優先: あらすじ・ジャンル・評価
        assert_eq!(details.core.original_title.as_deref(), Some("STEINS;GATE"));
        assert!(details.core.description.is_some());
        assert!(details.core.genres.contains(&"Sci-Fi".to_string()));
        assert!(details.core.rating.is_some());
    }

    #[test]
    fn from_annict_and_jikan_falls_back_to_jikan_when_annict_fields_are_empty() {
        let Some(json) = load_fixture("jikan/anime_details.json") else {
            eprintln!("fixture missing, skipped");
            return;
        };
        let mut work = sample_work();
        work.title = Some(String::new());
        work.images = None;
        work.episodes_count = None;
        work.season_name = None;

        let details = AnimeDetails::from_annict_and_jikan(&work, &json["data"]);

        // Annict側が空の項目はJikanの値へフォールバックする
        assert!(!details.core.title.is_empty());
        assert!(details.core.image_url.is_some());
        assert_eq!(details.episodes, Some(24));
        assert!(details.season.is_some());
    }
}
