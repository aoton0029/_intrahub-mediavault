//! Service層: `list_citations` / `add_citation`
//!
//! 第2段階。設計決定 D-11（api-tool-mapping.md §3）・REQ-903 / REQ-904 より。

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::{Deserialize, Serialize};

use crate::api::client::ApiClient;
use crate::api::models::{Citation, CreateCitationRequest, LocatorType};
use crate::result::outcome::{McpErrorCode, Outcome, ToolError, classify_api_error};
use crate::tools::citations::{
    AddCitationParams, AddCitationResult, CitationView, ListCitationsParams, ListCitationsResult,
};

/// `limit` の下限・上限。🔵 `search_library` と同じ規約（REQ-143）に揃える。
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

/// オフセット方式のカーソル。
///
/// 🟡 Intent: `GET /items/{id}/citations` は**ページネーションを持たず全件を返す**ため、
///    `services::cursor::Cursor`（keyset 方式・`after_created_at` + `after_id`）は使えない。
///    MCP 側で切り出す位置を保持する専用型を用意する。中身は Base64 で不透明化し、
///    AI が解釈して改変しないようにする（既存カーソルと同じ方針）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct OffsetCursor {
    offset: u32,
}

impl OffsetCursor {
    fn encode(self) -> String {
        let json = serde_json::to_vec(&self).expect("OffsetCursor は常にシリアライズ可能");
        URL_SAFE_NO_PAD.encode(json)
    }

    fn decode(encoded: &str) -> Result<Self, ()> {
        let bytes = URL_SAFE_NO_PAD.decode(encoded).map_err(|_| ())?;
        serde_json::from_slice(&bytes).map_err(|_| ())
    }
}

fn validate_limit(limit: Option<u32>) -> Result<u32, ToolError> {
    match limit {
        None => Ok(LIMIT_DEFAULT),
        Some(value) if (LIMIT_MIN..=LIMIT_MAX).contains(&value) => Ok(value),
        Some(value) => Err(invalid_argument(format!(
            "limit は {LIMIT_MIN}..={LIMIT_MAX} の範囲で指定してください（指定値: {value}）"
        ))),
    }
}

fn decode_cursor(cursor: &Option<String>) -> Result<u32, ToolError> {
    match cursor {
        None => Ok(0),
        Some(encoded) => OffsetCursor::decode(encoded)
            .map(|c| c.offset)
            .map_err(|_| invalid_argument("cursor が不正です")),
    }
}

fn to_view(citation: Citation) -> CitationView {
    CitationView {
        citation_id: citation.id,
        quote_text: citation.quote_text,
        note: citation.note,
        locator_type: citation.locator_type,
        page_number: citation.page_number,
        timestamp_seconds: citation.timestamp_seconds,
        location_number: citation.location_number,
        chapter: citation.chapter,
        created_at: citation.created_at,
    }
}

/// `list_citations` の本体。
///
/// 🟡 Intent: REQ-903 より。api が全件を返すため、MCP 側でオフセット切り出しを行う。
///    全件を取得している都合上 `total_count` は常に正確に返せる。
pub async fn list_citations(api: &ApiClient, params: ListCitationsParams) -> ListCitationsResult {
    let item_id = params.item_id;

    let limit = match validate_limit(params.limit) {
        Ok(limit) => limit,
        Err(error) => {
            return ListCitationsResult::early_return(item_id, Outcome::Error, Some(error));
        }
    };
    let offset = match decode_cursor(&params.cursor) {
        Ok(offset) => offset,
        Err(error) => {
            return ListCitationsResult::early_return(item_id, Outcome::Error, Some(error));
        }
    };

    let path = format!("/api/v1/items/{item_id}/citations");
    let all = match api.get::<Vec<Citation>>(&path, &[]).await {
        Ok(response) => response.data,
        Err(err) => {
            let (outcome, error) = classify_api_error(&err);
            return ListCitationsResult::early_return(item_id, outcome, Some(error));
        }
    };

    let total_count = all.len() as u32;
    let citations: Vec<CitationView> = all
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .map(to_view)
        .collect();

    // 🟡 Intent: 次ページが存在するときだけカーソルを返す。返した件数が limit 未満なら
    //    そこで打ち切り（末尾に到達している）。
    let next_offset = offset.saturating_add(citations.len() as u32);
    let next_cursor = (next_offset < total_count).then(|| {
        OffsetCursor {
            offset: next_offset,
        }
        .encode()
    });

    ListCitationsResult {
        outcome: Outcome::Success,
        item_id,
        total_count,
        citations,
        next_cursor,
        error: None,
    }
}

/// `locator_type` と位置フィールドの整合を検証する。
///
/// 🔵 Intent: 設計決定 D-11 / REQ-904 より。api 側は「対応する値の指定を推奨するが
///    **必須バリデーションはしない**（未指定は null のまま保存）」（`citations.md`）。
///    人間が UI から入力する分には許容できても、AI からの入力を受ける MCP でこれを緩くすると
///    **出典不明の引用が蓄積し、引用の追跡可能性が失われる**。api を呼ぶ前に弾く。
fn validate_locator(params: &AddCitationParams) -> Result<(), ToolError> {
    let required_missing = match params.locator_type {
        LocatorType::Page => params.page_number.is_none(),
        LocatorType::Timestamp => params.timestamp_seconds.is_none(),
        LocatorType::Location => params.location_number.is_none(),
        LocatorType::Chapter => params.chapter.as_ref().is_none_or(|s| s.trim().is_empty()),
        LocatorType::None => false,
    };
    if required_missing {
        let field = match params.locator_type {
            LocatorType::Page => "page_number",
            LocatorType::Timestamp => "timestamp_seconds",
            LocatorType::Location => "location_number",
            LocatorType::Chapter => "chapter",
            LocatorType::None => unreachable!("None は必須フィールドを持たない"),
        };
        return Err(invalid_argument(format!(
            "locator_type を指定した場合は対応する位置フィールドが必須です（不足: {field}）。\
             位置が分からない場合は locator_type: \"none\" を指定すること"
        )));
    }

    // 🔵 Intent: `none` は「位置情報なし」の意思表示。位置フィールドが同時に来ている場合、
    //    利用者の意図（種別の指定漏れか、値の指定漏れか）が判別できないため拒否する。
    if params.locator_type == LocatorType::None
        && (params.page_number.is_some()
            || params.timestamp_seconds.is_some()
            || params.location_number.is_some()
            || params.chapter.is_some())
    {
        return Err(invalid_argument(
            "locator_type: \"none\" のときは位置フィールドを指定できません。\
             位置を記録する場合は対応する locator_type を指定すること",
        ));
    }

    if params.quote_text.trim().is_empty() {
        return Err(invalid_argument("quote_text は空白のみを指定できません"));
    }

    Ok(())
}

/// `add_citation` の本体。
///
/// 🟡 Intent: REQ-904 より。**冪等ではない**（api に重複検出が無い）。
///    リトライは行わない。同一 `quote_text` の二重登録は防げないため、
///    ツール説明で重複登録を戒めることで担保する。
pub async fn add_citation(api: &ApiClient, params: AddCitationParams) -> AddCitationResult {
    if let Err(error) = validate_locator(&params) {
        return AddCitationResult::early_return(Outcome::Error, Some(error));
    }

    let item_id = params.item_id;
    let body = CreateCitationRequest {
        quote_text: params.quote_text,
        locator_type: params.locator_type,
        note: params.note,
        page_number: params.page_number,
        timestamp_seconds: params.timestamp_seconds,
        location_number: params.location_number,
        chapter: params.chapter,
    };

    let path = format!("/api/v1/items/{item_id}/citations");
    match api.post::<_, Citation>(&path, &body).await {
        Ok(citation) => AddCitationResult {
            outcome: Outcome::Success,
            citation: Some(to_view(citation)),
            error: None,
        },
        Err(err) => {
            let (outcome, error) = classify_api_error(&err);
            AddCitationResult::early_return(outcome, Some(error))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(locator_type: LocatorType) -> AddCitationParams {
        AddCitationParams {
            item_id: uuid::Uuid::new_v4(),
            quote_text: "引用本文".to_string(),
            locator_type,
            note: None,
            page_number: None,
            timestamp_seconds: None,
            location_number: None,
            chapter: None,
        }
    }

    #[test]
    fn offset_cursor_round_trips() {
        let cursor = OffsetCursor { offset: 40 };
        assert_eq!(OffsetCursor::decode(&cursor.encode()), Ok(cursor));
    }

    #[test]
    fn offset_cursor_rejects_garbage() {
        assert!(OffsetCursor::decode("not-base64").is_err());
    }

    /// カーソルは中身が読めない形にする（AI が解釈して改変しないため）。
    #[test]
    fn offset_cursor_is_opaque() {
        let encoded = OffsetCursor { offset: 40 }.encode();
        assert!(!encoded.contains("40"));
        assert!(!encoded.contains("offset"));
    }

    #[test]
    fn validate_limit_rejects_51_without_rounding() {
        assert_eq!(
            validate_limit(Some(51)).unwrap_err().code,
            "MCP_INVALID_ARGUMENT"
        );
    }

    #[test]
    fn validate_limit_defaults_to_20() {
        assert_eq!(validate_limit(None).unwrap(), 20);
    }

    // --- D-11 / REQ-904: locator_type と位置フィールドの整合 ---

    #[test]
    fn page_without_page_number_is_rejected() {
        let err = validate_locator(&params(LocatorType::Page)).unwrap_err();
        assert_eq!(err.code, "MCP_INVALID_ARGUMENT");
        assert!(
            err.message.contains("page_number"),
            "不足しているフィールド名を伝える: {}",
            err.message
        );
    }

    #[test]
    fn timestamp_without_seconds_is_rejected() {
        let err = validate_locator(&params(LocatorType::Timestamp)).unwrap_err();
        assert!(err.message.contains("timestamp_seconds"));
    }

    #[test]
    fn location_without_number_is_rejected() {
        let err = validate_locator(&params(LocatorType::Location)).unwrap_err();
        assert!(err.message.contains("location_number"));
    }

    #[test]
    fn chapter_without_value_is_rejected() {
        let err = validate_locator(&params(LocatorType::Chapter)).unwrap_err();
        assert!(err.message.contains("chapter"));
    }

    /// 空白のみの `chapter` は「指定した」とみなさない。
    #[test]
    fn chapter_with_whitespace_only_is_rejected() {
        let mut p = params(LocatorType::Chapter);
        p.chapter = Some("   ".to_string());
        assert!(validate_locator(&p).is_err());
    }

    #[test]
    fn page_with_page_number_is_accepted() {
        let mut p = params(LocatorType::Page);
        p.page_number = Some(128);
        assert!(validate_locator(&p).is_ok());
    }

    /// `none` は位置フィールドなしで受理される（位置が分からない引用の記録手段）。
    #[test]
    fn none_without_any_locator_field_is_accepted() {
        assert!(validate_locator(&params(LocatorType::None)).is_ok());
    }

    /// `none` なのに位置が来ている場合、利用者の意図が判別できないため拒否する。
    #[test]
    fn none_with_a_locator_field_is_rejected() {
        let mut p = params(LocatorType::None);
        p.page_number = Some(128);
        let err = validate_locator(&p).unwrap_err();
        assert_eq!(err.code, "MCP_INVALID_ARGUMENT");
    }

    #[test]
    fn empty_quote_text_is_rejected() {
        let mut p = params(LocatorType::None);
        p.quote_text = "   ".to_string();
        assert!(validate_locator(&p).is_err());
    }
}
