//! 全プロバイダ共通のノーマライズ済みコアモデル
//!
//! docs/api-samples/ 配下の実レスポンス（Jikan / AniList / TMDb / IGDB /
//! OpenLibrary / NDL / Steam）を分析し、全プロバイダが実質的に共有する
//! 項目のみをコアに置く。プロバイダ・メディア種別固有の項目は
//! `domain/{anime,manga,movie,drama,game,novel}.rs` の拡張構造体側に持たせる。

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::api_credential::ApiProvider;
use crate::models::item::MediaType;

/// 共通コアモデル
///
/// - `title` は全プロバイダに存在（唯一の必須文字列）
/// - `description` / `image_url` / `rating` は 7 プロバイダ中 6 以上に存在
/// - `release_date` は精度がプロバイダごとに異なる（年のみ〜完全日付）ため文字列
/// - `rating` は 0–10 に正規化（AniList/IGDB/Steam metascore は 100 点満点→/10）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaCore {
    pub media_type: MediaType,
    /// 採用したプロバイダ。キー不要プロバイダ（Jikan）は `None`（旧 ExternalSearchResult からの慣例）
    pub provider: Option<ApiProvider>,
    /// プロバイダ固有 ID（mal_id / tmdb id / igdb id / OLID 等の元値）
    pub external_id: String,
    pub title: String,
    /// 原題（original_title / title_japanese / title.native 等）
    pub original_title: Option<String>,
    /// 別題（英題・シノニム等）
    #[serde(default)]
    pub alternative_titles: Vec<String>,
    /// あらすじ・概要（overview / synopsis / summary / description）
    pub description: Option<String>,
    /// リリース日（release_date / first_air_date / aired.from / publish_date）
    pub release_date: Option<String>,
    /// 代表画像の完全 URL（TMDb poster_path・IGDB cover 等は絶対 URL に解決済み）
    pub image_url: Option<String>,
    #[serde(default)]
    pub genres: Vec<String>,
    /// 0–10 に正規化した評価値
    pub rating: Option<f64>,
    /// プロバイダ側の作品ページ URL
    pub url: Option<String>,
}

// ── raw JSON 抽出ヘルパー（domain 配下のマッパー共用） ──────────────────────

/// 文字列フィールドを取り出す（空文字は None 扱い）
pub(super) fn json_str(v: &Value, key: &str) -> Option<String> {
    v.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
}

/// 数値フィールドを f64 で取り出す
pub(super) fn json_f64(v: &Value, key: &str) -> Option<f64> {
    v.get(key).and_then(Value::as_f64)
}

/// 数値フィールドを u32 で取り出す
pub(super) fn json_u32(v: &Value, key: &str) -> Option<u32> {
    v.get(key).and_then(Value::as_u64).map(|n| n as u32)
}

/// `[{"<name_key>": "..."}, ...]` 形式の配列から名前一覧を取り出す
pub(super) fn json_names(v: &Value, key: &str, name_key: &str) -> Vec<String> {
    v.get(key)
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(|e| json_str(e, name_key)).collect())
        .unwrap_or_default()
}

/// 文字列配列を取り出す
pub(super) fn json_str_array(v: &Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

/// 100 点満点の評価を 0–10 に正規化する
pub(super) fn normalize_score_100(score: f64) -> f64 {
    score / 10.0
}
