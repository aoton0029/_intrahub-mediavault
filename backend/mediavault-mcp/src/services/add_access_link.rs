//! Service層: `add_access_link`
//!
//! TASK-0022: add_access_link ツールの実装
//!
//! **要点はフォールバック**（REQ-081）。api の配信プラットフォームは5種固定であり、
//! Jellyfin・Calibre-Web・U-NEXT などは表現できない。非対応の場合は通常リンクへ
//! 登録し、そのことを結果で通知する。

use uuid::Uuid;

use crate::api::client::ApiClient;
use crate::api::error::ApiClientError;
use crate::api::models::{ItemLink, ItemTrailer, StreamingPlatform};
use crate::result::outcome::{McpErrorCode, Outcome, ToolError, classify_api_error};
use crate::tools::add_access_link::{
    AddAccessLinkParams, AddAccessLinkResult, LinkKind, RegisteredAs, normalize_platform,
    resolve_fallback_label, validate_url,
};

/// `POST /items/{id}/streaming-links` が返す409の重複判定コード
/// 🔵 item-streaming-links.md より。
const DUPLICATE_STREAMING_LINK: &str = "DUPLICATE_STREAMING_LINK";

#[derive(Debug, Clone, serde::Serialize)]
struct CreateItemLinkRequest {
    url: String,
    label: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct CreateItemTrailerRequest {
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct CreateItemStreamingLinkRequest {
    platform: StreamingPlatform,
    url: String,
}

/// `add_access_link` の本体。
///
/// 🔵 Intent: mcp-tools.md 9. のとおり。処理順序は
///    (1) URL のバリデーション（api を呼ばない）
///    (2) `kind` による振り分け（`streaming` はプラットフォーム対応表と照合しフォールバック判定）
///    (3) 対応する api への `POST`（409 は冪等化）
///    の3段階。**書き込みは自動リトライしない**（architecture.md 可用性方針より）。
pub async fn add_access_link(api: &ApiClient, params: AddAccessLinkParams) -> AddAccessLinkResult {
    let url = match validate_url(&params.url) {
        Ok(url) => url,
        Err(_) => {
            let error = ToolError {
                code: McpErrorCode::InvalidArgument.as_str().to_string(),
                message: "url は http / https のみ受け付けます。".to_string(),
                retriable: false,
            };
            return AddAccessLinkResult::early_return(Outcome::Error, error);
        }
    };

    match params.kind {
        LinkKind::Trailer => add_trailer(api, params.item_id, &url, params.label).await,
        LinkKind::Link => match resolve_fallback_label(params.label.as_deref(), None, &url) {
            Some(label) => add_link(api, params.item_id, &url, label, None).await,
            None => label_missing_error(),
        },
        LinkKind::Streaming => {
            match params.platform.as_deref().and_then(normalize_platform) {
                Some(platform) => add_streaming_link(api, params.item_id, &url, platform).await,
                None => {
                    // 🔵 Intent: REQ-081。非対応プラットフォーム（U-NEXT・Jellyfin等）または
                    //    platform 未指定は通常リンクへフォールバックする。
                    match resolve_fallback_label(
                        params.label.as_deref(),
                        params.platform.as_deref(),
                        &url,
                    ) {
                        Some(label) => {
                            add_link(api, params.item_id, &url, label, Some("streaming")).await
                        }
                        None => label_missing_error(),
                    }
                }
            }
        }
    }
}

/// 🟡 Intent: 実装項目4「ラベルが決められない場合はバリデーションエラーとする」より。
///    api の必須項目を空文字で埋めない。
fn label_missing_error() -> AddAccessLinkResult {
    let error = ToolError {
        code: McpErrorCode::InvalidArgument.as_str().to_string(),
        message: "通常リンクの label を決定できません。label または platform を指定するか、URL にホスト名を含めてください。".to_string(),
        retriable: false,
    };
    AddAccessLinkResult::early_return(Outcome::Error, error)
}

async fn add_link(
    api: &ApiClient,
    item_id: Uuid,
    url: &url::Url,
    label: String,
    fallback_from: Option<&str>,
) -> AddAccessLinkResult {
    let body = CreateItemLinkRequest {
        url: url.to_string(),
        label,
    };

    // 🟡 Intent: 実装項目6「通常リンクには一意制約がない可能性がある」より。
    //    冪等化は行わず、api の応答をそのまま反映する。
    match api
        .post::<_, ItemLink>(&format!("/api/v1/items/{item_id}/links"), &body)
        .await
    {
        Ok(link) => AddAccessLinkResult {
            outcome: Outcome::Success,
            link_id: Some(link.id),
            registered_as: Some(RegisteredAs::Link),
            fallback_from: fallback_from.map(str::to_string),
            already_registered: false,
            error: None,
        },
        Err(err) => {
            let (outcome, error) = classify_api_error(&err);
            AddAccessLinkResult::early_return(outcome, error)
        }
    }
}

async fn add_trailer(
    api: &ApiClient,
    item_id: Uuid,
    url: &url::Url,
    label: Option<String>,
) -> AddAccessLinkResult {
    let body = CreateItemTrailerRequest {
        url: url.to_string(),
        label,
    };

    match api
        .post::<_, ItemTrailer>(&format!("/api/v1/items/{item_id}/trailers"), &body)
        .await
    {
        Ok(trailer) => AddAccessLinkResult {
            outcome: Outcome::Success,
            link_id: Some(trailer.id),
            registered_as: Some(RegisteredAs::Trailer),
            fallback_from: None,
            already_registered: false,
            error: None,
        },
        Err(err) => {
            let (outcome, error) = classify_api_error(&err);
            AddAccessLinkResult::early_return(outcome, error)
        }
    }
}

async fn add_streaming_link(
    api: &ApiClient,
    item_id: Uuid,
    url: &url::Url,
    platform: StreamingPlatform,
) -> AddAccessLinkResult {
    let body = CreateItemStreamingLinkRequest {
        platform,
        url: url.to_string(),
    };

    #[derive(Debug, Clone, serde::Deserialize)]
    struct CreatedStreamingLink {
        id: Uuid,
    }

    match api
        .post::<_, CreatedStreamingLink>(&format!("/api/v1/items/{item_id}/streaming-links"), &body)
        .await
    {
        Ok(link) => AddAccessLinkResult {
            outcome: Outcome::Success,
            link_id: Some(link.id),
            registered_as: Some(RegisteredAs::StreamingLink),
            fallback_from: None,
            already_registered: false,
            error: None,
        },
        // 🔵 Intent: REQ-113 系と同様に409を冪等化する（実装項目6）。既存リンクの
        //    link_id はレスポンスに含まれないため `None` とする。
        Err(ApiClientError::Api {
            code, status: 409, ..
        }) if code == DUPLICATE_STREAMING_LINK => AddAccessLinkResult {
            outcome: Outcome::Success,
            link_id: None,
            registered_as: Some(RegisteredAs::StreamingLink),
            fallback_from: None,
            already_registered: true,
            error: None,
        },
        Err(err) => {
            let (outcome, error) = classify_api_error(&err);
            AddAccessLinkResult::early_return(outcome, error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_streaming_link_code_matches_api_error() {
        let err = ApiClientError::Api {
            code: "DUPLICATE_STREAMING_LINK".to_string(),
            message: "既に登録されています".to_string(),
            status: 409,
        };
        let matched = matches!(
            &err,
            ApiClientError::Api { code, status: 409, .. } if code == DUPLICATE_STREAMING_LINK
        );
        assert!(matched);
    }
}
