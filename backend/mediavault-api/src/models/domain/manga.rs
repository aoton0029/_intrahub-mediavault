//! 漫画のドメインモデル（共通コア + 漫画固有項目）
//!
//! docs/api-samples/anilist/search_media_manga.json（および Jikan manga レスポンス形式）を根拠とする。

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::core::{
    MediaCore, json_f64, json_names, json_str, json_str_array, json_u32, normalize_score_100,
};
use crate::models::api_credential::ApiProvider;
use crate::models::item::MediaType;

/// 漫画詳細モデル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaDetails {
    #[serde(flatten)]
    pub core: MediaCore,
    pub chapters: Option<u32>,
    pub volumes: Option<u32>,
    /// 連載ステータス（"Publishing" / "RELEASING" 等、プロバイダ表記のまま）
    pub status: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    /// 掲載誌（Jikan `serializations`）
    #[serde(default)]
    pub serializations: Vec<String>,
}

impl MangaDetails {
    /// Jikan `GET /manga/{id}/full` の `data` オブジェクトから構築する。
    pub fn from_jikan_details(data: &Value) -> Self {
        let core = MediaCore {
            media_type: MediaType::Manga,
            provider: None,
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
                .get("published")
                .and_then(|p| json_str(p, "from"))
                .map(|d| d.chars().take(10).collect()),
            image_url: data
                .get("images")
                .and_then(|i| i.get("jpg"))
                .and_then(|j| json_str(j, "image_url")),
            genres: json_names(data, "genres", "name"),
            rating: json_f64(data, "score"),
            url: json_str(data, "url"),
        };
        MangaDetails {
            core,
            chapters: json_u32(data, "chapters"),
            volumes: json_u32(data, "volumes"),
            status: json_str(data, "status"),
            authors: json_names(data, "authors", "name"),
            serializations: json_names(data, "serializations", "name"),
        }
    }

    /// AniList GraphQL の `Media` オブジェクト（type=MANGA）から構築する。
    pub fn from_anilist_media(media: &Value) -> Self {
        let title = media.get("title").cloned().unwrap_or(Value::Null);
        let core = MediaCore {
            media_type: MediaType::Manga,
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
            release_date: None,
            image_url: media.get("coverImage").and_then(|c| json_str(c, "large")),
            genres: json_str_array(media, "genres"),
            rating: json_f64(media, "averageScore").map(normalize_score_100),
            url: None,
        };
        MangaDetails {
            core,
            chapters: json_u32(media, "chapters"),
            volumes: json_u32(media, "volumes"),
            status: json_str(media, "status"),
            authors: Vec::new(),
            serializations: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_util::load_fixture;
    use super::*;

    #[test]
    fn from_anilist_media_maps_manga_search_result() {
        let Some(json) = load_fixture("anilist/search_media_manga.json") else {
            eprintln!("fixture missing, skipped");
            return;
        };
        let media = &json["data"]["Page"]["media"][0];
        let details = MangaDetails::from_anilist_media(media);

        assert_eq!(details.core.media_type, MediaType::Manga);
        assert_eq!(details.core.provider, Some(ApiProvider::AniList));
        assert_eq!(details.core.title, "Berserk");
        assert_eq!(details.core.original_title.as_deref(), Some("ベルセルク"));
        assert!((0.0..=10.0).contains(&details.core.rating.unwrap()));
        assert!(details.core.genres.contains(&"Action".to_string()));
        assert_eq!(details.status.as_deref(), Some("RELEASING"));
    }

    #[test]
    fn from_jikan_details_maps_manga_fixture_when_available() {
        // Jikan 側の障害でサンプル未取得の場合はスキップ（fetch_samples 再実行で有効化）
        let Some(json) = load_fixture("jikan/manga_details.json") else {
            eprintln!("fixture missing, skipped");
            return;
        };
        let details = MangaDetails::from_jikan_details(&json["data"]);
        assert!(!details.core.title.is_empty());
        assert!(!details.core.external_id.is_empty());
        assert!(!details.authors.is_empty());
    }
}
