//! theme_songs / theme_song_links / item_theme_songs のモデル・リクエストDTO・バリデーション
//!
//! 曲（theme_songs）はアイテムから独立したマスタとして持ち、item_theme_songsを介して
//! アイテムと多対多で紐づく。アーティスト・作曲・作詞は正規化せずtheme_songsの列として持つ。
//! models/staff.rs（マスタ + 紐付けテーブル）と対称な構造。

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::item::MediaType;
use crate::models::response::{ApiError, ApiErrorCode};

/// 曲がその作品においてどう使われているかを表す種別（固定7種類）
///
/// enumの定義順が`GET /items/:id/theme-songs`の並び順（op → ed → ... → other）に対応する。
/// PostgreSQL側のenum定義順と一致させること。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "theme_song_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ThemeSongType {
    /// オープニングテーマ
    Op,
    /// エンディングテーマ
    Ed,
    /// 挿入歌
    Insert,
    /// イメージソング（本編未使用のタイアップ曲など）
    Image,
    /// キャラクターソング
    Character,
    /// 主題歌（OP/EDの区別がない作品。映画向け）
    Theme,
    /// 上記に当てはまらないもの
    Other,
}

/// 曲の配信・視聴リンク種別（固定7種類）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "theme_song_link_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ThemeSongLinkType {
    Youtube,
    Spotify,
    AppleMusic,
    AmazonMusic,
    Niconico,
    Official,
    Other,
}

/// theme_songs本体（DB行マッピング用）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ThemeSong {
    pub id: Uuid,
    pub title: String,
    pub artist: Option<String>,
    pub composer: Option<String>,
    pub lyricist: Option<String>,
    pub arranger: Option<String>,
    pub note: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// theme_song_links本体
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ThemeSongLink {
    pub id: Uuid,
    pub theme_song_id: Uuid,
    pub link_type: ThemeSongLinkType,
    pub url: String,
    pub label: Option<String>,
    pub sort_order: i32,
    pub created_at: NaiveDateTime,
}

/// 曲本体 + そのリンク一覧（`GET /theme-songs`・`POST /theme-songs`等のレスポンス表現）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSongWithLinks {
    pub id: Uuid,
    pub title: String,
    pub artist: Option<String>,
    pub composer: Option<String>,
    pub lyricist: Option<String>,
    pub arranger: Option<String>,
    pub note: Option<String>,
    pub links: Vec<ThemeSongLink>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl ThemeSongWithLinks {
    /// DB行（ThemeSong）とリンク一覧を合成する
    pub fn from_parts(song: ThemeSong, links: Vec<ThemeSongLink>) -> Self {
        Self {
            id: song.id,
            title: song.title,
            artist: song.artist,
            composer: song.composer,
            lyricist: song.lyricist,
            arranger: song.arranger,
            note: song.note,
            links,
            created_at: song.created_at,
            updated_at: song.updated_at,
        }
    }
}

/// `GET /theme-songs/{id}`のitems欄。その曲が使われているアイテムの参照
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ThemeSongItemRef {
    pub item_id: Uuid,
    pub title: String,
    pub media_type: MediaType,
    pub theme_type: ThemeSongType,
}

/// `GET /theme-songs/{id}` レスポンス（ThemeSongWithLinks + 使用作品一覧）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSongDetail {
    pub id: Uuid,
    pub title: String,
    pub artist: Option<String>,
    pub composer: Option<String>,
    pub lyricist: Option<String>,
    pub arranger: Option<String>,
    pub note: Option<String>,
    pub links: Vec<ThemeSongLink>,
    pub items: Vec<ThemeSongItemRef>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl ThemeSongDetail {
    /// ThemeSongWithLinksと使用作品一覧を合成する
    pub fn from_parts(song: ThemeSongWithLinks, items: Vec<ThemeSongItemRef>) -> Self {
        Self {
            id: song.id,
            title: song.title,
            artist: song.artist,
            composer: song.composer,
            lyricist: song.lyricist,
            arranger: song.arranger,
            note: song.note,
            links: song.links,
            items,
            created_at: song.created_at,
            updated_at: song.updated_at,
        }
    }
}

/// item_theme_songs（アイテムと曲の紐付け）。レスポンスには曲本体とそのリンクをネストして返す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemThemeSong {
    pub id: Uuid,
    pub item_id: Uuid,
    pub theme_type: ThemeSongType,
    pub display_order: i32,
    pub created_at: NaiveDateTime,
    pub theme_song: ThemeSongWithLinks,
}

/// `POST /theme-songs`のlinks要素・`POST /theme-songs/{id}/links` リクエストDTO
#[derive(Debug, Clone, Deserialize)]
pub struct CreateThemeSongLinkRequest {
    pub link_type: ThemeSongLinkType,
    pub url: String,
    pub label: Option<String>,
    pub sort_order: Option<i32>,
}

/// `POST /theme-songs` リクエストDTO
#[derive(Debug, Clone, Deserialize)]
pub struct CreateThemeSongRequest {
    pub title: String,
    pub artist: Option<String>,
    pub composer: Option<String>,
    pub lyricist: Option<String>,
    pub arranger: Option<String>,
    pub note: Option<String>,
    #[serde(default)]
    pub links: Vec<CreateThemeSongLinkRequest>,
}

/// `PATCH /theme-songs/{id}` リクエストDTO（指定されたフィールドのみ更新）
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateThemeSongRequest {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub composer: Option<String>,
    pub lyricist: Option<String>,
    pub arranger: Option<String>,
    pub note: Option<String>,
}

/// `GET /theme-songs` クエリパラメータDTO
#[derive(Debug, Clone, Deserialize)]
pub struct ListThemeSongsQuery {
    /// 曲名・artistの部分一致（大文字小文字を区別しない）
    pub q: Option<String>,
    pub limit: Option<u32>,
}

/// `POST /items/{id}/theme-songs` リクエストDTO
#[derive(Debug, Clone, Deserialize)]
pub struct CreateItemThemeSongRequest {
    pub theme_song_id: Uuid,
    pub theme_type: ThemeSongType,
    pub display_order: Option<i32>,
}

/// title列の最大長（VARCHAR(500)）
pub const TITLE_MAX_LEN: usize = 500;

/// UpdateThemeSongRequestに更新対象フィールドが1つでも指定されているかを判定する
pub fn has_any_update_field(request: &UpdateThemeSongRequest) -> bool {
    request.title.is_some()
        || request.artist.is_some()
        || request.composer.is_some()
        || request.lyricist.is_some()
        || request.arranger.is_some()
        || request.note.is_some()
}

/// 曲名が空白のみでないこと・長さ上限内であることを検証する
fn validate_title(title: &str) -> Result<(), ApiError> {
    if title.trim().is_empty() {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "titleは必須です",
        ));
    }
    if title.chars().count() > TITLE_MAX_LEN {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            format!("titleは{TITLE_MAX_LEN}文字以内で指定してください"),
        ));
    }
    Ok(())
}

/// リンクのurlが空文字（trim後）でないことを検証する
fn validate_link_url(url: &str) -> Result<(), ApiError> {
    if url.trim().is_empty() {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "urlは必須です",
        ));
    }
    Ok(())
}

/// `POST /theme-songs/{id}/links` リクエストのバリデーション（url空文字拒否）を行う
pub fn parse_create_theme_song_link_request(
    request: CreateThemeSongLinkRequest,
) -> Result<CreateThemeSongLinkRequest, ApiError> {
    validate_link_url(&request.url)?;
    Ok(request)
}

/// `POST /theme-songs` リクエストのバリデーションを行う
///
/// title空文字・link url空文字は400、links内のURL重複は409 DUPLICATE_THEME_SONG_LINKとする。
/// URL重複はDB側のuq_theme_song_links制約でも検出できるが、単一トランザクション内での
/// ロールバック理由を明示するためリクエスト段階でも判定する。
pub fn parse_create_theme_song_request(
    request: CreateThemeSongRequest,
) -> Result<CreateThemeSongRequest, ApiError> {
    validate_title(&request.title)?;

    let mut seen_urls: Vec<&str> = Vec::with_capacity(request.links.len());
    for link in &request.links {
        validate_link_url(&link.url)?;
        if seen_urls.contains(&link.url.as_str()) {
            return Err(ApiError::new(
                ApiErrorCode::DuplicateThemeSongLink,
                "linksに同一URLが重複しています",
            ));
        }
        seen_urls.push(&link.url);
    }

    Ok(request)
}

/// `PATCH /theme-songs/{id}` リクエストのバリデーション（title指定時のみ空白拒否）を行う
pub fn parse_update_theme_song_request(
    request: UpdateThemeSongRequest,
) -> Result<UpdateThemeSongRequest, ApiError> {
    if let Some(title) = &request.title {
        validate_title(title)?;
    }
    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_song_type_deserializes_all_seven_values() {
        let values = [
            ("op", ThemeSongType::Op),
            ("ed", ThemeSongType::Ed),
            ("insert", ThemeSongType::Insert),
            ("image", ThemeSongType::Image),
            ("character", ThemeSongType::Character),
            ("theme", ThemeSongType::Theme),
            ("other", ThemeSongType::Other),
        ];

        for (raw, expected) in values {
            let parsed: ThemeSongType = serde_json::from_value(serde_json::json!(raw)).unwrap();
            assert_eq!(parsed, expected);
        }
    }

    #[test]
    fn unknown_theme_type_fails_deserialization() {
        let result: Result<ThemeSongType, _> = serde_json::from_value(serde_json::json!("ost"));
        assert!(result.is_err());
    }

    #[test]
    fn theme_song_link_type_deserializes_snake_case_values() {
        let parsed: ThemeSongLinkType =
            serde_json::from_value(serde_json::json!("apple_music")).unwrap();
        assert_eq!(parsed, ThemeSongLinkType::AppleMusic);
    }

    #[test]
    fn create_theme_song_request_defaults_links_to_empty() {
        let value = serde_json::json!({ "title": "残酷な天使のテーゼ" });
        let request: CreateThemeSongRequest = serde_json::from_value(value).unwrap();
        assert!(request.links.is_empty());
    }

    #[test]
    fn parse_create_theme_song_request_rejects_blank_title() {
        let request = CreateThemeSongRequest {
            title: "   ".to_string(),
            artist: None,
            composer: None,
            lyricist: None,
            arranger: None,
            note: None,
            links: Vec::new(),
        };

        let err = parse_create_theme_song_request(request).unwrap_err();
        assert_eq!(err.error.code, ApiErrorCode::ValidationError.as_code_str());
    }

    #[test]
    fn parse_create_theme_song_request_rejects_empty_link_url() {
        let request = CreateThemeSongRequest {
            title: "曲名".to_string(),
            artist: None,
            composer: None,
            lyricist: None,
            arranger: None,
            note: None,
            links: vec![CreateThemeSongLinkRequest {
                link_type: ThemeSongLinkType::Youtube,
                url: "".to_string(),
                label: None,
                sort_order: None,
            }],
        };

        let err = parse_create_theme_song_request(request).unwrap_err();
        assert_eq!(err.error.code, ApiErrorCode::ValidationError.as_code_str());
    }

    #[test]
    fn parse_create_theme_song_request_rejects_duplicate_link_urls() {
        let url = "https://www.youtube.com/watch?v=xxxxxxxxxxx";
        let request = CreateThemeSongRequest {
            title: "曲名".to_string(),
            artist: None,
            composer: None,
            lyricist: None,
            arranger: None,
            note: None,
            links: vec![
                CreateThemeSongLinkRequest {
                    link_type: ThemeSongLinkType::Youtube,
                    url: url.to_string(),
                    label: Some("MV".to_string()),
                    sort_order: None,
                },
                CreateThemeSongLinkRequest {
                    link_type: ThemeSongLinkType::Official,
                    url: url.to_string(),
                    label: None,
                    sort_order: None,
                },
            ],
        };

        let err = parse_create_theme_song_request(request).unwrap_err();
        assert_eq!(
            err.error.code,
            ApiErrorCode::DuplicateThemeSongLink.as_code_str()
        );
    }

    #[test]
    fn parse_create_theme_song_request_accepts_same_link_type_twice() {
        let request = CreateThemeSongRequest {
            title: "曲名".to_string(),
            artist: Some("高橋洋子".to_string()),
            composer: None,
            lyricist: None,
            arranger: None,
            note: None,
            links: vec![
                CreateThemeSongLinkRequest {
                    link_type: ThemeSongLinkType::Youtube,
                    url: "https://example.com/mv".to_string(),
                    label: Some("MV".to_string()),
                    sort_order: Some(0),
                },
                CreateThemeSongLinkRequest {
                    link_type: ThemeSongLinkType::Youtube,
                    url: "https://example.com/tv-size".to_string(),
                    label: Some("TVサイズ".to_string()),
                    sort_order: Some(1),
                },
            ],
        };

        assert!(parse_create_theme_song_request(request).is_ok());
    }

    #[test]
    fn parse_update_theme_song_request_rejects_blank_title() {
        let request = UpdateThemeSongRequest {
            title: Some("　".to_string()),
            ..UpdateThemeSongRequest::default()
        };

        let err = parse_update_theme_song_request(request).unwrap_err();
        assert_eq!(err.error.code, ApiErrorCode::ValidationError.as_code_str());
    }

    #[test]
    fn parse_update_theme_song_request_allows_omitted_title() {
        let request = UpdateThemeSongRequest {
            artist: Some("アーティスト".to_string()),
            ..UpdateThemeSongRequest::default()
        };

        assert!(parse_update_theme_song_request(request).is_ok());
    }

    #[test]
    fn has_any_update_field_detects_empty_request() {
        assert!(!has_any_update_field(&UpdateThemeSongRequest::default()));
        assert!(has_any_update_field(&UpdateThemeSongRequest {
            note: Some("フル尺".to_string()),
            ..UpdateThemeSongRequest::default()
        }));
    }

    #[test]
    fn create_item_theme_song_request_with_invalid_theme_type_fails_deserialization() {
        let value = serde_json::json!({
            "theme_song_id": Uuid::new_v4(),
            "theme_type": "opening"
        });

        let result: Result<CreateItemThemeSongRequest, _> = serde_json::from_value(value);
        assert!(result.is_err());
    }
}
