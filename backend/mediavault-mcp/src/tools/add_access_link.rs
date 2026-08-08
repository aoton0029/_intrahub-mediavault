//! Tool層: `add_access_link`
//!
//! TASK-0022: add_access_link ツールの実装

use uuid::Uuid;

use crate::result::outcome::{Outcome, ToolError};

/// `kind` の3値。
///
/// 🔵 Intent: interfaces.rs `LinkKind`・mcp-tools.md 9. パラメータ表より。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LinkKind {
    Link,
    Streaming,
    Trailer,
}

/// `add_access_link` ツールの引数。
///
/// 🔵 Intent: interfaces.rs `AddAccessLinkParams`・mcp-tools.md 9. のパラメータ表より。
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
pub struct AddAccessLinkParams {
    /// 対象作品。UUIDのみ 🔵 REQ-142
    pub item_id: Uuid,
    /// http / https のみ許可 🔵 REQ-116
    pub url: String,
    pub kind: LinkKind,
    /// `kind = Streaming` のときのプラットフォーム名。
    /// 対応5種以外を指定した場合は通常リンクへフォールバックする 🔵 REQ-081
    pub platform: Option<String>,
    /// 通常リンクのラベル 🔵 `item-links.md` の label 必須仕様より
    pub label: Option<String>,
}

/// 実際に登録された先。
///
/// 🔵 Intent: interfaces.rs `RegisteredAs`・REQ-081「どちらに登録したかを結果で返す」より。
#[derive(Debug, Clone, Copy, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RegisteredAs {
    Link,
    StreamingLink,
    Trailer,
}

/// 🔵 Intent: interfaces.rs `AddAccessLinkResult`・REQ-080 / REQ-081 より。
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct AddAccessLinkResult {
    pub outcome: Outcome,
    pub link_id: Option<Uuid>,
    pub registered_as: Option<RegisteredAs>,
    /// `Streaming` を要求したが通常リンクへフォールバックした場合に `"streaming"` が入る
    /// 🔵 REQ-081「フォールバックが起きたことを結果に含める」
    pub fallback_from: Option<String>,
    /// 409 `DUPLICATE_STREAMING_LINK` により既存リンクとみなした場合 true 🟡
    pub already_registered: bool,
    pub error: Option<ToolError>,
}

impl AddAccessLinkResult {
    /// api を一切呼ばずに終了する場合（バリデーションエラー等）の共通コンストラクタ。
    pub fn early_return(outcome: Outcome, error: ToolError) -> Self {
        AddAccessLinkResult {
            outcome,
            link_id: None,
            registered_as: None,
            fallback_from: None,
            already_registered: false,
            error: Some(error),
        }
    }
}

/// api が受け付ける配信プラットフォーム5種 🔵 item-streaming-links.md より。
#[cfg(test)]
const SUPPORTED_PLATFORMS: &[&str] = &[
    "netflix",
    "amazon_prime",
    "disney_plus",
    "dmm_tv",
    "apple_tv",
];

/// プラットフォーム名の正規化 + 対応表照合。
///
/// 🟡 Intent: 実装項目5より。小文字化 + 空白をアンダースコアへ変換してから照合する。
/// 過度な正規化（意訳・翻訳）は行わない。
pub fn normalize_platform(platform: &str) -> Option<crate::api::models::StreamingPlatform> {
    let normalized = platform.trim().to_lowercase().replace(' ', "_");
    match normalized.as_str() {
        "netflix" => Some(crate::api::models::StreamingPlatform::Netflix),
        "amazon_prime" => Some(crate::api::models::StreamingPlatform::AmazonPrime),
        "disney_plus" => Some(crate::api::models::StreamingPlatform::DisneyPlus),
        "dmm_tv" => Some(crate::api::models::StreamingPlatform::DmmTv),
        "apple_tv" => Some(crate::api::models::StreamingPlatform::AppleTv),
        _ => None,
    }
}

/// URL が不正である（パース失敗、または http / https 以外のスキーム）ことを示す。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidUrl;

/// URL が http / https スキームか検証する 🔵 REQ-116。
pub fn validate_url(url: &str) -> Result<url::Url, InvalidUrl> {
    let parsed = url::Url::parse(url).map_err(|_| InvalidUrl)?;
    match parsed.scheme() {
        "http" | "https" => Ok(parsed),
        _ => Err(InvalidUrl),
    }
}

/// フォールバック時のラベル決定。
///
/// 🟡 Intent: 実装項目4「明示ラベル > platform名 > URLホスト名」の優先順より。
/// api の `label` は必須のため、決められない場合は `None` を返し呼び出し側で
/// バリデーションエラーとする。
pub fn resolve_fallback_label(
    label: Option<&str>,
    platform: Option<&str>,
    url: &url::Url,
) -> Option<String> {
    if let Some(label) = label
        && !label.trim().is_empty()
    {
        return Some(label.to_string());
    }
    if let Some(platform) = platform
        && !platform.trim().is_empty()
    {
        return Some(platform.to_string());
    }
    url.host_str().map(|host| host.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テストケース1: URL のバリデーション
    #[test]
    fn validates_url_scheme() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://jellyfin.local:8096/web/index.html").is_ok());
        assert!(validate_url("not-a-url").is_err());
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("javascript:alert(1)").is_err());
    }

    /// テストケース2: プラットフォーム判定
    #[test]
    fn normalizes_and_matches_supported_platforms() {
        assert!(normalize_platform("netflix").is_some());
        assert!(normalize_platform("Netflix").is_some());
        assert!(normalize_platform("NETFLIX").is_some());
        assert!(normalize_platform("u-next").is_none());
    }

    #[test]
    fn all_five_supported_platforms_normalize() {
        for platform in SUPPORTED_PLATFORMS {
            assert!(
                normalize_platform(platform).is_some(),
                "{platform} should match"
            );
        }
    }

    /// テストケース3: フォールバック時のラベル決定
    #[test]
    fn resolves_fallback_label_by_priority() {
        let url = url::Url::parse("http://jellyfin.local:8096/").unwrap();

        assert_eq!(
            resolve_fallback_label(Some("マイJellyfin"), Some("jellyfin"), &url),
            Some("マイJellyfin".to_string())
        );
        assert_eq!(
            resolve_fallback_label(None, Some("u-next"), &url),
            Some("u-next".to_string())
        );
        assert_eq!(
            resolve_fallback_label(None, None, &url),
            Some("jellyfin.local".to_string())
        );
    }

    #[test]
    fn resolve_fallback_label_ignores_blank_values() {
        let url = url::Url::parse("http://jellyfin.local/").unwrap();

        assert_eq!(
            resolve_fallback_label(Some("  "), None, &url),
            Some("jellyfin.local".to_string())
        );
    }

    /// early_return は書き込みフィールドがすべて空
    #[test]
    fn early_return_has_empty_fields() {
        let error = ToolError {
            code: "MCP_INVALID_ARGUMENT".to_string(),
            message: "不正なURLです".to_string(),
            retriable: false,
        };
        let result = AddAccessLinkResult::early_return(Outcome::Error, error);

        assert!(result.link_id.is_none());
        assert!(result.registered_as.is_none());
        assert!(result.fallback_from.is_none());
        assert!(!result.already_registered);
        assert!(result.error.is_some());
    }
}
