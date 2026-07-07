//! ドラマ（TV シリーズ）のドメインモデル（共通コア + TV 固有項目）
//!
//! docs/api-samples/tmdb/tv_series.json・search_tv.json を根拠とする。

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::core::{MediaCore, json_f64, json_names, json_str, json_u32};
use super::movie::tmdb_image_url;
use crate::models::api_credential::ApiProvider;
use crate::models::item::MediaType;

/// ドラマ詳細モデル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DramaDetails {
    #[serde(flatten)]
    pub core: MediaCore,
    pub number_of_seasons: Option<u32>,
    pub number_of_episodes: Option<u32>,
    /// 放送局（TMDb `networks`）
    #[serde(default)]
    pub networks: Vec<String>,
    /// 放送ステータス（TMDb `status`: "Ended" 等）
    pub status: Option<String>,
    pub original_language: Option<String>,
    pub first_air_date: Option<String>,
    pub last_air_date: Option<String>,
}

impl DramaDetails {
    /// TMDb `GET /tv/{id}` レスポンス（または検索結果の1要素）から構築する。
    pub fn from_tmdb_tv(data: &Value) -> Self {
        let first_air_date = json_str(data, "first_air_date");
        let core = MediaCore {
            media_type: MediaType::Drama,
            provider: Some(ApiProvider::Tmdb),
            external_id: data
                .get("id")
                .and_then(Value::as_u64)
                .map(|id| id.to_string())
                .unwrap_or_default(),
            title: json_str(data, "name").unwrap_or_default(),
            original_title: json_str(data, "original_name"),
            alternative_titles: Vec::new(),
            description: json_str(data, "overview"),
            release_date: first_air_date.clone(),
            image_url: tmdb_image_url(data, "poster_path"),
            genres: json_names(data, "genres", "name"),
            rating: json_f64(data, "vote_average"),
            url: json_str(data, "homepage"),
        };
        DramaDetails {
            core,
            number_of_seasons: json_u32(data, "number_of_seasons"),
            number_of_episodes: json_u32(data, "number_of_episodes"),
            networks: json_names(data, "networks", "name"),
            status: json_str(data, "status"),
            original_language: json_str(data, "original_language"),
            first_air_date,
            last_air_date: json_str(data, "last_air_date"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_util::load_fixture;
    use super::*;

    #[test]
    fn from_tmdb_tv_maps_series_details() {
        let Some(json) = load_fixture("tmdb/tv_series.json") else {
            eprintln!("fixture missing, skipped");
            return;
        };
        let details = DramaDetails::from_tmdb_tv(&json);

        assert_eq!(details.core.media_type, MediaType::Drama);
        assert_eq!(details.core.provider, Some(ApiProvider::Tmdb));
        assert_eq!(details.core.external_id, "1396");
        assert_eq!(details.core.title, "ブレイキング・バッド");
        assert_eq!(details.core.original_title.as_deref(), Some("Breaking Bad"));
        assert_eq!(details.core.release_date.as_deref(), Some("2008-01-20"));
        assert!((0.0..=10.0).contains(&details.core.rating.unwrap()));
        assert_eq!(details.number_of_seasons, Some(5));
        assert_eq!(details.number_of_episodes, Some(62));
        assert_eq!(details.networks, vec!["AMC".to_string()]);
        assert_eq!(details.status.as_deref(), Some("Ended"));
    }
}
