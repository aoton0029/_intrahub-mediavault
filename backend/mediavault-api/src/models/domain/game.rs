//! ゲームのドメインモデル（共通コア + ゲーム固有項目）
//!
//! docs/api-samples/igdb/query_games.json・steam/app_details.json を根拠とする。

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::core::{MediaCore, json_f64, json_names, json_str, json_str_array, normalize_score_100};
use crate::models::api_credential::ApiProvider;
use crate::models::item::MediaType;

/// IGDB の画像 URL（`//images.igdb.com/...`）を完全 URL に解決する。
fn igdb_image_url(v: &Value, key: &str) -> Option<String> {
    json_str(v, key).map(|url| {
        if let Some(rest) = url.strip_prefix("//") {
            format!("https://{rest}")
        } else {
            url
        }
    })
}

/// UNIX 秒を "YYYY-MM-DD" に変換する。
fn unix_to_date(secs: i64) -> Option<String> {
    chrono::DateTime::from_timestamp(secs, 0).map(|dt| dt.format("%Y-%m-%d").to_string())
}

/// ゲーム詳細モデル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameDetails {
    #[serde(flatten)]
    pub core: MediaCore,
    #[serde(default)]
    pub platforms: Vec<String>,
    #[serde(default)]
    pub developers: Vec<String>,
    #[serde(default)]
    pub publishers: Vec<String>,
    #[serde(default)]
    pub screenshots: Vec<String>,
    /// Metacritic スコア（Steam のみ、0–100 のまま保持）
    pub metacritic: Option<u32>,
    /// Steam AppID（IGDB 由来の場合は None）
    pub steam_appid: Option<u64>,
    /// 詳細ストーリー（IGDB `storyline`）
    pub storyline: Option<String>,
}

impl GameDetails {
    /// IGDB `/games` クエリ結果の1要素から構築する。
    ///
    /// fetch_samples の Apicalypse クエリ
    /// （name, summary, storyline, first_release_date, cover.url, genres.name,
    /// platforms.name, involved_companies.{company.name,developer,publisher},
    /// rating, screenshots.url, url, alternative_names.name）を前提とする。
    pub fn from_igdb(data: &Value) -> Self {
        let companies = data
            .get("involved_companies")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let company_names = |role: &str| -> Vec<String> {
            companies
                .iter()
                .filter(|c| c.get(role).and_then(Value::as_bool).unwrap_or(false))
                .filter_map(|c| c.get("company").and_then(|co| json_str(co, "name")))
                .collect()
        };
        let core = MediaCore {
            media_type: MediaType::Game,
            provider: Some(ApiProvider::Igdb),
            external_id: data
                .get("id")
                .and_then(Value::as_u64)
                .map(|id| id.to_string())
                .unwrap_or_default(),
            title: json_str(data, "name").unwrap_or_default(),
            original_title: None,
            alternative_titles: json_names(data, "alternative_names", "name"),
            description: json_str(data, "summary"),
            release_date: data
                .get("first_release_date")
                .and_then(Value::as_i64)
                .and_then(unix_to_date),
            image_url: data.get("cover").and_then(|c| igdb_image_url(c, "url")),
            genres: json_names(data, "genres", "name"),
            rating: json_f64(data, "rating").map(normalize_score_100),
            url: json_str(data, "url"),
        };
        GameDetails {
            core,
            platforms: json_names(data, "platforms", "name"),
            developers: company_names("developer"),
            publishers: company_names("publisher"),
            screenshots: data
                .get("screenshots")
                .and_then(Value::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter_map(|s| igdb_image_url(s, "url"))
                        .collect()
                })
                .unwrap_or_default(),
            metacritic: None,
            steam_appid: None,
            storyline: json_str(data, "storyline"),
        }
    }

    /// Steam `storesearch` の検索結果1件（`{"id":.., "name":.., "tiny_image":..}`）から構築する。
    ///
    /// 検索結果には説明・評価・画像等の詳細情報が含まれないため、一覧表示に必要な
    /// id/name/tiny_image（サムネイル用）のみをマッピングする軽量版。詳細情報はインポート確定時に
    /// `get_app_details` + `from_steam_app` で別途取得する想定。
    pub fn from_steam_search(data: &Value) -> Self {
        let core = MediaCore {
            media_type: MediaType::Game,
            provider: Some(ApiProvider::Steam),
            external_id: data
                .get("id")
                .and_then(Value::as_u64)
                .map(|id| id.to_string())
                .unwrap_or_default(),
            title: json_str(data, "name").unwrap_or_default(),
            original_title: None,
            alternative_titles: Vec::new(),
            description: None,
            release_date: None,
            image_url: json_str(data, "tiny_image"),
            genres: Vec::new(),
            rating: None,
            url: None,
        };
        GameDetails {
            core,
            platforms: Vec::new(),
            developers: Vec::new(),
            publishers: Vec::new(),
            screenshots: Vec::new(),
            metacritic: None,
            steam_appid: data.get("id").and_then(Value::as_u64),
            storyline: None,
        }
    }

    /// Steam `appdetails` の `data` オブジェクトから構築する。
    pub fn from_steam_app(data: &Value) -> Self {
        let metacritic = data
            .get("metacritic")
            .and_then(|m| m.get("score"))
            .and_then(Value::as_u64)
            .map(|s| s as u32);
        let platforms = data
            .get("platforms")
            .and_then(Value::as_object)
            .map(|obj| {
                obj.iter()
                    .filter(|(_, v)| v.as_bool().unwrap_or(false))
                    .map(|(k, _)| k.clone())
                    .collect()
            })
            .unwrap_or_default();
        let steam_appid = data.get("steam_appid").and_then(Value::as_u64);
        let core = MediaCore {
            media_type: MediaType::Game,
            provider: Some(ApiProvider::Steam),
            external_id: steam_appid.map(|id| id.to_string()).unwrap_or_default(),
            title: json_str(data, "name").unwrap_or_default(),
            original_title: None,
            alternative_titles: Vec::new(),
            description: json_str(data, "short_description"),
            release_date: data.get("release_date").and_then(|r| json_str(r, "date")),
            image_url: json_str(data, "header_image"),
            genres: json_names(data, "genres", "description"),
            rating: metacritic.map(|s| normalize_score_100(f64::from(s))),
            url: json_str(data, "website"),
        };
        GameDetails {
            core,
            platforms,
            developers: json_str_array(data, "developers"),
            publishers: json_str_array(data, "publishers"),
            screenshots: data
                .get("screenshots")
                .and_then(Value::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter_map(|s| json_str(s, "path_thumbnail"))
                        .collect()
                })
                .unwrap_or_default(),
            metacritic,
            steam_appid,
            storyline: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_util::load_fixture;
    use super::*;

    #[test]
    fn from_igdb_maps_game_query_result() {
        let Some(json) = load_fixture("igdb/query_games.json") else {
            eprintln!("fixture missing, skipped");
            return;
        };
        let details = GameDetails::from_igdb(&json[0]);

        assert_eq!(details.core.media_type, MediaType::Game);
        assert_eq!(details.core.provider, Some(ApiProvider::Igdb));
        assert!(!details.core.external_id.is_empty());
        assert!(!details.core.title.is_empty());
        assert!(
            details
                .core
                .image_url
                .as_deref()
                .unwrap()
                .starts_with("https://")
        );
        assert!((0.0..=10.0).contains(&details.core.rating.unwrap()));
        assert!(details.core.release_date.as_deref().unwrap().len() == 10);
        assert!(!details.platforms.is_empty());
        assert!(!details.publishers.is_empty());
        assert!(!details.screenshots.is_empty());
    }

    #[test]
    fn from_steam_search_maps_id_name_tiny_image_only() {
        let json = serde_json::json!({
            "id": 400,
            "name": "Portal",
            "type": "game",
            "tiny_image": "https://example.com/tiny/400.jpg"
        });
        let details = GameDetails::from_steam_search(&json);

        assert_eq!(details.core.provider, Some(ApiProvider::Steam));
        assert_eq!(details.core.external_id, "400");
        assert_eq!(details.core.title, "Portal");
        assert_eq!(
            details.core.image_url.as_deref(),
            Some("https://example.com/tiny/400.jpg")
        );
        assert_eq!(details.steam_appid, Some(400));
        assert!(details.core.description.is_none());
        assert!(details.core.rating.is_none());
        assert!(details.platforms.is_empty());
    }

    #[test]
    fn from_steam_app_maps_app_details() {
        let Some(json) = load_fixture("steam/app_details.json") else {
            eprintln!("fixture missing, skipped");
            return;
        };
        let details = GameDetails::from_steam_app(&json["400"]["data"]);

        assert_eq!(details.core.provider, Some(ApiProvider::Steam));
        assert_eq!(details.core.external_id, "400");
        assert_eq!(details.core.title, "Portal");
        assert_eq!(details.steam_appid, Some(400));
        assert!(details.core.description.is_some());
        assert!(details.core.image_url.is_some());
        assert!(details.platforms.contains(&"windows".to_string()));
        assert!(!details.developers.is_empty());
        if let Some(rating) = details.core.rating {
            assert!((0.0..=10.0).contains(&rating));
        }
    }
}
