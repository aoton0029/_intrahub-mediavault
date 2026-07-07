//! 映画のドメインモデル（共通コア + 映画固有項目）
//!
//! docs/api-samples/tmdb/movie_details.json・search_movie.json を根拠とする。

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::core::{MediaCore, json_f64, json_names, json_str, json_u32};
use crate::models::api_credential::ApiProvider;
use crate::models::item::MediaType;

/// TMDb ポスター画像のベース URL（services/external_search.rs と同じ幅を使用）
const TMDB_IMAGE_BASE: &str = "https://image.tmdb.org/t/p/w342";

/// TMDb の poster_path を完全 URL に解決する。
pub(super) fn tmdb_image_url(v: &Value, key: &str) -> Option<String> {
    json_str(v, key).map(|path| format!("{TMDB_IMAGE_BASE}{path}"))
}

/// 映画詳細モデル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieDetails {
    #[serde(flatten)]
    pub core: MediaCore,
    pub runtime_minutes: Option<u32>,
    pub original_language: Option<String>,
    pub vote_count: Option<u32>,
    /// 所属コレクション名（TMDb `belongs_to_collection.name`）
    pub collection: Option<String>,
    #[serde(default)]
    pub production_companies: Vec<String>,
}

impl MovieDetails {
    /// TMDb `GET /movie/{id}` レスポンス（または検索結果の1要素）から構築する。
    pub fn from_tmdb(data: &Value) -> Self {
        let core = MediaCore {
            media_type: MediaType::Movie,
            provider: Some(ApiProvider::Tmdb),
            external_id: data
                .get("id")
                .and_then(Value::as_u64)
                .map(|id| id.to_string())
                .unwrap_or_default(),
            title: json_str(data, "title").unwrap_or_default(),
            original_title: json_str(data, "original_title"),
            alternative_titles: Vec::new(),
            description: json_str(data, "overview"),
            release_date: json_str(data, "release_date"),
            image_url: tmdb_image_url(data, "poster_path"),
            genres: json_names(data, "genres", "name"),
            rating: json_f64(data, "vote_average"),
            url: json_str(data, "homepage"),
        };
        MovieDetails {
            core,
            runtime_minutes: json_u32(data, "runtime"),
            original_language: json_str(data, "original_language"),
            vote_count: json_u32(data, "vote_count"),
            collection: data
                .get("belongs_to_collection")
                .and_then(|c| json_str(c, "name")),
            production_companies: json_names(data, "production_companies", "name"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_util::load_fixture;
    use super::*;

    #[test]
    fn from_tmdb_maps_movie_details() {
        let Some(json) = load_fixture("tmdb/movie_details.json") else {
            eprintln!("fixture missing, skipped");
            return;
        };
        let details = MovieDetails::from_tmdb(&json);

        assert_eq!(details.core.media_type, MediaType::Movie);
        assert_eq!(details.core.provider, Some(ApiProvider::Tmdb));
        assert_eq!(details.core.external_id, "603");
        assert_eq!(details.core.title, "マトリックス");
        assert_eq!(details.core.original_title.as_deref(), Some("The Matrix"));
        assert!(
            details
                .core
                .image_url
                .as_deref()
                .unwrap()
                .starts_with("https://image.tmdb.org/t/p/")
        );
        assert_eq!(details.core.release_date.as_deref(), Some("1999-03-31"));
        assert!((0.0..=10.0).contains(&details.core.rating.unwrap()));
        assert_eq!(details.runtime_minutes, Some(136));
        assert_eq!(details.original_language.as_deref(), Some("en"));
        assert!(details.collection.is_some());
        assert!(!details.production_companies.is_empty());
    }
}
