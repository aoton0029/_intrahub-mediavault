//! Service層: `search_library`
//!
//! TASK-0013: search_library ツールの実装

use serde_json::json;

use crate::api::client::ApiClient;
use crate::api::models::ItemWithRefs;
use crate::result::outcome::{McpErrorCode, Outcome, ToolError, classify_api_error};
use crate::result::summary::ItemSummary;
use crate::services::cursor::Cursor;
use crate::services::resolve::{MasterResolver, Resolution};
use crate::tools::search_library::{SearchLibraryParams, SearchLibraryResult};

/// `limit` の下限・上限。🔵 REQ-143「1..=50、既定20」
const LIMIT_MIN: u32 = 1;
const LIMIT_MAX: u32 = 50;
const LIMIT_DEFAULT: u32 = 20;

fn invalid_argument(message: impl Into<String>) -> ToolError {
    ToolError {
        code: McpErrorCode::InvalidArgument.as_str().to_string(),
        message: message.into(),
        retriable: false,
    }
}

/// `limit` を検証する。**丸めない**（タスクファイル実装項目2）。
///
/// 🔵 Intent: 丸めると AI に上限が伝わらず同じ誤りを繰り返すため、範囲外は
///    エラーにしエラーメッセージへ上限値を含める。
fn validate_limit(limit: Option<u32>) -> Result<u32, ToolError> {
    match limit {
        None => Ok(LIMIT_DEFAULT),
        Some(value) if (LIMIT_MIN..=LIMIT_MAX).contains(&value) => Ok(value),
        Some(value) => Err(invalid_argument(format!(
            "limit は {LIMIT_MIN}..={LIMIT_MAX} の範囲で指定してください（指定値: {value}）"
        ))),
    }
}

/// 検索語が空白のみでないことを検証する（EDGE-102）。
fn validate_title(title: &Option<String>) -> Result<(), ToolError> {
    match title {
        Some(value) if value.trim().is_empty() => {
            Err(invalid_argument("title は空白のみを指定できません"))
        }
        _ => Ok(()),
    }
}

/// カーソルをデコードする。不正な場合は `MCP_INVALID_ARGUMENT`（実装項目5）。
fn decode_cursor(cursor: &Option<String>) -> Result<Option<Cursor>, ToolError> {
    match cursor {
        None => Ok(None),
        Some(encoded) => Cursor::decode(encoded)
            .map(Some)
            .map_err(|_| invalid_argument("cursor が不正です")),
    }
}

/// `search_library` の本体。
///
/// 🔵 Intent: dataflow.md 機能1 のステップ順（バリデーション→名前解決→api呼び出し→整形）に従う。
pub async fn search(api: &ApiClient, params: SearchLibraryParams) -> SearchLibraryResult {
    let limit = match validate_limit(params.limit) {
        Ok(limit) => limit,
        Err(error) => return SearchLibraryResult::early_return(Outcome::Error, Some(error)),
    };

    if let Err(error) = validate_title(&params.title) {
        return SearchLibraryResult::early_return(Outcome::Error, Some(error));
    }

    let cursor = match decode_cursor(&params.cursor) {
        Ok(cursor) => cursor,
        Err(error) => return SearchLibraryResult::early_return(Outcome::Error, Some(error)),
    };

    let mut resolver = MasterResolver::new(api);
    let mut applied_filters = json!({});

    if let Some(title) = &params.title {
        applied_filters["title"] = json!(title);
    }
    if let Some(media_type) = params.media_type {
        applied_filters["media_type"] = serde_json::to_value(media_type).unwrap();
    }
    if let Some(status) = params.status {
        applied_filters["status"] = serde_json::to_value(status).unwrap();
    }
    if let Some(is_favorite) = params.is_favorite {
        applied_filters["is_favorite"] = json!(is_favorite);
    }
    if let Some(year) = params.year {
        applied_filters["year"] = json!(year);
    }
    if let Some(sort) = params.sort {
        applied_filters["sort"] = serde_json::to_value(sort).unwrap();
    }

    let tag_id = if let Some(tag) = &params.tag {
        applied_filters["tag"] = json!(tag);
        match resolver.resolve_tag(tag).await {
            Ok(Resolution::Found(named_ref)) => {
                applied_filters["tag_id"] = json!(named_ref.id);
                Some(named_ref.id)
            }
            Ok(Resolution::NotFound {
                requested,
                available,
            }) => {
                return name_not_resolved(requested, available);
            }
            Err(err) => {
                let (outcome, error) = classify_api_error(&err);
                return SearchLibraryResult::early_return(outcome, Some(error));
            }
        }
    } else {
        None
    };

    let category_id = if let Some(category) = &params.category {
        applied_filters["category"] = json!(category);
        match resolver.resolve_category(category).await {
            Ok(Resolution::Found(named_ref)) => {
                applied_filters["category_id"] = json!(named_ref.id);
                Some(named_ref.id)
            }
            Ok(Resolution::NotFound {
                requested,
                available,
            }) => {
                return name_not_resolved(requested, available);
            }
            Err(err) => {
                let (outcome, error) = classify_api_error(&err);
                return SearchLibraryResult::early_return(outcome, Some(error));
            }
        }
    } else {
        None
    };

    let query = build_query(&params, limit, tag_id, category_id, cursor.as_ref());

    match api.get::<Vec<ItemWithRefs>>("/api/v1/items", &query).await {
        Ok(response) => {
            let items: Vec<ItemSummary> = response.data.iter().map(ItemSummary::from).collect();
            let pagination = response.pagination;
            let total_count = pagination.as_ref().and_then(|p| p.total).unwrap_or(0);
            let next_cursor = pagination.as_ref().and_then(|pagination| {
                if pagination.has_more {
                    let (created_at, id) =
                        (pagination.next_after_created_at?, pagination.next_after_id?);
                    Some(
                        Cursor {
                            after_created_at: created_at,
                            after_id: id,
                        }
                        .encode(),
                    )
                } else {
                    None
                }
            });

            SearchLibraryResult {
                outcome: Outcome::Success,
                source: crate::tools::search_library::SOURCE,
                total_count,
                items,
                next_cursor,
                applied_filters,
                error: None,
            }
        }
        Err(err) => {
            let (outcome, error) = classify_api_error(&err);
            SearchLibraryResult::early_return(outcome, Some(error))
        }
    }
}

/// タグ・カテゴリ名が解決できない場合の結果を組み立てる。
///
/// 🟡 Intent: タスクファイル「検索を実行しない」「`available_names` 付きで返す」より。
///    `SearchLibraryResult` に専用フィールドがないため、`applied_filters` へ含める。
fn name_not_resolved(requested: String, available: Vec<String>) -> SearchLibraryResult {
    SearchLibraryResult {
        outcome: Outcome::NotFound,
        source: crate::tools::search_library::SOURCE,
        total_count: 0,
        items: Vec::new(),
        next_cursor: None,
        applied_filters: json!({
            "requested_name": requested,
            "available_names": available,
        }),
        error: Some(ToolError {
            code: McpErrorCode::NameNotResolved.as_str().to_string(),
            message: format!("「{requested}」に一致する名前が見つかりませんでした"),
            retriable: false,
        }),
    }
}

/// api へ渡すクエリパラメータを組み立てる。
///
/// 🔵 Intent: タスクファイル実装項目4のクエリ列挙・`include_total=true` を常に付ける（🟡）に従う。
fn build_query(
    params: &SearchLibraryParams,
    limit: u32,
    tag_id: Option<uuid::Uuid>,
    category_id: Option<uuid::Uuid>,
    cursor: Option<&Cursor>,
) -> Vec<(&'static str, String)> {
    let mut query: Vec<(&'static str, String)> = Vec::new();

    if let Some(title) = &params.title {
        query.push(("title", title.clone()));
    }
    if let Some(media_type) = params.media_type {
        query.push(("media_type", as_query_string(media_type)));
    }
    if let Some(status) = params.status {
        query.push(("status", as_query_string(status)));
    }
    if let Some(tag_id) = tag_id {
        query.push(("tag_id", tag_id.to_string()));
    }
    if let Some(category_id) = category_id {
        query.push(("category_id", category_id.to_string()));
    }
    if let Some(is_favorite) = params.is_favorite {
        query.push(("is_favorite", is_favorite.to_string()));
    }
    if let Some(year) = params.year {
        query.push(("year", year.to_string()));
    }
    if let Some(sort) = params.sort {
        query.push(("sort", sort.as_api_sort().to_string()));
    }
    query.push(("limit", limit.to_string()));
    if let Some(cursor) = cursor {
        query.push((
            "after_created_at",
            cursor
                .after_created_at
                .format("%Y-%m-%dT%H:%M:%S")
                .to_string(),
        ));
        query.push(("after_id", cursor.after_id.to_string()));
    }
    query.push(("include_total", "true".to_string()));

    query
}

/// `#[serde(rename_all = "snake_case")]` な enum をクエリ文字列へ変換する。
fn as_query_string<T: serde::Serialize>(value: T) -> String {
    match serde_json::to_value(value).expect("enum は常にシリアライズ可能") {
        serde_json::Value::String(s) => s,
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テストケース1: limit のバリデーション
    #[test]
    fn validate_limit_uses_default_when_unspecified() {
        assert_eq!(validate_limit(None).unwrap(), 20);
    }

    #[test]
    fn validate_limit_accepts_lower_bound() {
        assert_eq!(validate_limit(Some(1)).unwrap(), 1);
    }

    #[test]
    fn validate_limit_accepts_upper_bound() {
        assert_eq!(validate_limit(Some(50)).unwrap(), 50);
    }

    #[test]
    fn validate_limit_rejects_51_without_rounding() {
        let err = validate_limit(Some(51)).unwrap_err();
        assert_eq!(err.code, "MCP_INVALID_ARGUMENT");
    }

    #[test]
    fn validate_limit_rejects_zero() {
        let err = validate_limit(Some(0)).unwrap_err();
        assert_eq!(err.code, "MCP_INVALID_ARGUMENT");
    }

    /// テストケース2: 空白のみの検索語
    #[test]
    fn validate_title_rejects_whitespace_only() {
        let err = validate_title(&Some("   ".to_string())).unwrap_err();
        assert_eq!(err.code, "MCP_INVALID_ARGUMENT");
    }

    #[test]
    fn validate_title_accepts_none() {
        assert!(validate_title(&None).is_ok());
    }

    #[test]
    fn validate_title_accepts_non_empty() {
        assert!(validate_title(&Some("作品A".to_string())).is_ok());
    }

    /// テストケース4: 不正なカーソル
    #[test]
    fn decode_cursor_rejects_invalid_cursor() {
        let err = decode_cursor(&Some("not-base64".to_string())).unwrap_err();
        assert_eq!(err.code, "MCP_INVALID_ARGUMENT");
    }

    #[test]
    fn decode_cursor_accepts_none() {
        assert_eq!(decode_cursor(&None).unwrap(), None);
    }

    #[test]
    fn as_query_string_converts_snake_case_enum() {
        assert_eq!(
            as_query_string(crate::api::models::MediaType::Anime),
            "anime"
        );
    }
}
