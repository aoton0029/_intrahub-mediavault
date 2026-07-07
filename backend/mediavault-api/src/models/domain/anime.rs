//! アニメのドメインモデル（共通コア + アニメ固有項目）
//!
//! docs/api-samples/jikan/anime_details.json・anilist/media_details.json を根拠とする。

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::core::{
    MediaCore, json_f64, json_names, json_str, json_str_array, json_u32, normalize_score_100,
};
use crate::models::api_credential::ApiProvider;
use crate::models::item::MediaType;

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
}

#[cfg(test)]
mod tests {
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
}
