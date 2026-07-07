//! メディア種別を横断する詳細モデルのワイヤ表現
//!
//! `GET /items/search` のレスポンス要素・`POST /items/import` のリクエストボディとして
//! 使用する。各 `*Details` は `MediaCore` を `#[serde(flatten)]` しており、
//! ワイヤ上はフラットな1オブジェクトで `media_type` フィールドが判別子を兼ねる。
//!
//! - Serialize: `#[serde(untagged)]` — 内側の構造体をそのまま出力（`media_type` は
//!   `MediaCore` 由来の1回のみ現れる）
//! - Deserialize: 手動実装 — `media_type` を先読みして対応する構造体へディスパッチ
//!   （untagged 派生は Anime/Manga 等の形状が重なるため使えない）

use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize};

use super::core::MediaCore;
use super::{AnimeDetails, DramaDetails, GameDetails, MangaDetails, MovieDetails, NovelDetails};

/// 全メディア種別の詳細モデルを束ねるワイヤ表現
///
/// `academic_book` / `paper` は書誌形状が小説と同一のため `NovelDetails` を共有する。
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum MediaDetails {
    Anime(AnimeDetails),
    Manga(MangaDetails),
    Movie(MovieDetails),
    Drama(DramaDetails),
    Game(GameDetails),
    Novel(NovelDetails),
    AcademicBook(NovelDetails),
    Paper(NovelDetails),
}

impl MediaDetails {
    /// 全プロバイダ共通のコア部分への参照を返す
    pub fn core(&self) -> &MediaCore {
        match self {
            MediaDetails::Anime(d) => &d.core,
            MediaDetails::Manga(d) => &d.core,
            MediaDetails::Movie(d) => &d.core,
            MediaDetails::Drama(d) => &d.core,
            MediaDetails::Game(d) => &d.core,
            MediaDetails::Novel(d) | MediaDetails::AcademicBook(d) | MediaDetails::Paper(d) => {
                &d.core
            }
        }
    }
}

impl<'de> Deserialize<'de> for MediaDetails {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let media_type = value
            .get("media_type")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| D::Error::missing_field("media_type"))?
            .to_string();
        // media_type（MediaCoreのフィールド）を判別子として対応する詳細構造体へ振り分ける
        let details = match media_type.as_str() {
            "anime" => MediaDetails::Anime(from_value(value)?),
            "manga" => MediaDetails::Manga(from_value(value)?),
            "movie" => MediaDetails::Movie(from_value(value)?),
            "drama" => MediaDetails::Drama(from_value(value)?),
            "game" => MediaDetails::Game(from_value(value)?),
            "novel" => MediaDetails::Novel(from_value(value)?),
            "academic_book" => MediaDetails::AcademicBook(from_value(value)?),
            "paper" => MediaDetails::Paper(from_value(value)?),
            other => {
                return Err(D::Error::custom(format!("未知のmedia_type: {other}")));
            }
        };
        Ok(details)
    }
}

fn from_value<T: serde::de::DeserializeOwned, E: DeError>(
    value: serde_json::Value,
) -> Result<T, E> {
    serde_json::from_value(value).map_err(E::custom)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::item::MediaType;

    fn minimal_json(media_type: &str) -> serde_json::Value {
        serde_json::json!({
            "media_type": media_type,
            "provider": null,
            "external_id": "1",
            "title": "タイトル"
        })
    }

    #[test]
    fn deserializes_each_media_type_into_matching_variant() {
        let cases = [
            ("anime", MediaType::Anime),
            ("manga", MediaType::Manga),
            ("movie", MediaType::Movie),
            ("drama", MediaType::Drama),
            ("game", MediaType::Game),
            ("novel", MediaType::Novel),
            ("academic_book", MediaType::AcademicBook),
            ("paper", MediaType::Paper),
        ];
        for (tag, expected) in cases {
            let details: MediaDetails = serde_json::from_value(minimal_json(tag))
                .unwrap_or_else(|e| panic!("media_type={tag} のデシリアライズに失敗: {e}"));
            assert_eq!(details.core().media_type, expected);
            match (tag, &details) {
                ("anime", MediaDetails::Anime(_))
                | ("manga", MediaDetails::Manga(_))
                | ("movie", MediaDetails::Movie(_))
                | ("drama", MediaDetails::Drama(_))
                | ("game", MediaDetails::Game(_))
                | ("novel", MediaDetails::Novel(_))
                | ("academic_book", MediaDetails::AcademicBook(_))
                | ("paper", MediaDetails::Paper(_)) => {}
                _ => panic!("media_type={tag} が想定外のvariantへディスパッチされた"),
            }
        }
    }

    #[test]
    fn serialize_emits_media_type_exactly_once_and_round_trips() {
        let details: MediaDetails =
            serde_json::from_value(minimal_json("anime")).expect("デシリアライズは成功するはず");
        let text = serde_json::to_string(&details).expect("シリアライズは成功するはず");
        assert_eq!(text.matches("\"media_type\"").count(), 1);

        let round_tripped: MediaDetails =
            serde_json::from_str(&text).expect("ラウンドトリップは成功するはず");
        assert_eq!(round_tripped.core().external_id, "1");
        assert!(matches!(round_tripped, MediaDetails::Anime(_)));
    }

    #[test]
    fn unknown_media_type_returns_error() {
        let result: Result<MediaDetails, _> = serde_json::from_value(minimal_json("invalid_type"));
        assert!(result.is_err());
    }

    #[test]
    fn missing_media_type_returns_error() {
        let result: Result<MediaDetails, _> = serde_json::from_value(serde_json::json!({
            "external_id": "1",
            "title": "タイトル"
        }));
        assert!(result.is_err());
    }
}
