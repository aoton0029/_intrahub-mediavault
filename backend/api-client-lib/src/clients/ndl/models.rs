use crate::error::ApiError;

/// NDL 書誌モデル（フラット構造体）。
#[derive(Debug, Clone)]
pub struct NdlItemModel {
    pub title: Option<String>,
    pub creator: Option<Vec<String>>,
    pub publisher: Option<String>,
    pub isbn: Option<String>,
    pub isbn13: Option<String>,
    pub pub_date: Option<String>,
    pub description: Option<String>,
    /// NDL サムネイル URL（ISBN-13 から生成）
    pub thumbnail_url: Option<String>,
}

/// NDL クライアントが返すモデル列挙型。
#[derive(Debug)]
pub enum NdlModel {
    /// `search` の結果
    Items(Vec<NdlItemModel>),
}

// ── XML パーサー ──────────────────────────────────────────────────────────

const NDL_THUMBNAIL_BASE: &str = "https://ndlsearch.ndl.go.jp/thumbnail";

/// NDL OpenSearch RSS/XML レスポンスを `NdlItemModel` のリストにパースする。
pub(super) fn parse_ndl_xml(xml: &str) -> Result<Vec<NdlItemModel>, ApiError> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut items: Vec<NdlItemModel> = Vec::new();

    // ビルダー状態
    let mut in_item = false;
    let mut current_tag: Option<String> = None;
    let mut current_xsi_type: Option<String> = None;

    // 現在 item の各フィールド
    let mut title: Option<String> = None;
    let mut creators: Vec<String> = Vec::new();
    let mut publisher: Option<String> = None;
    let mut isbn: Option<String> = None;
    let mut isbn13: Option<String> = None;
    let mut pub_date: Option<String> = None;
    let mut description: Option<String> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag_bytes = e.name().into_inner();
                let tag = std::str::from_utf8(tag_bytes)
                    .map_err(|err| ApiError::Parse(err.to_string()))?
                    .to_string();

                if tag == "item" {
                    in_item = true;
                    // リセット
                    title = None;
                    creators = Vec::new();
                    publisher = None;
                    isbn = None;
                    isbn13 = None;
                    pub_date = None;
                    description = None;
                }

                if in_item {
                    // xsi:type 属性を取得
                    let xsi_type = e
                        .attributes()
                        .filter_map(|a| a.ok())
                        .find(|a| {
                            let key = std::str::from_utf8(a.key.into_inner()).unwrap_or("");
                            key == "xsi:type" || key.ends_with(":type")
                        })
                        .and_then(|a| {
                            std::str::from_utf8(a.value.as_ref())
                                .ok()
                                .map(|s| s.to_string())
                        });
                    current_xsi_type = xsi_type;
                    current_tag = Some(tag);
                }
            }

            Ok(Event::Text(ref e)) => {
                if !in_item {
                    continue;
                }
                let text = e
                    .unescape()
                    .map_err(|err| ApiError::Parse(err.to_string()))?
                    .trim()
                    .to_string();
                if text.is_empty() {
                    continue;
                }

                match current_tag.as_deref() {
                    Some("dc:title") => title = Some(text),
                    Some("dc:creator") => creators.push(text),
                    Some("dc:publisher") => publisher = Some(text),
                    Some("dc:date") => pub_date = Some(text),
                    Some("dc:description") => {
                        if description.is_none() {
                            description = Some(text);
                        }
                    }
                    Some("dc:identifier") => {
                        if let Some(ref xtype) = current_xsi_type {
                            if xtype.contains("ISBN13") {
                                isbn13 = Some(text);
                            } else if xtype.contains("ISBN") {
                                isbn = Some(text);
                            }
                        }
                    }
                    _ => {}
                }
            }

            Ok(Event::End(ref e)) => {
                let tag_bytes = e.name().into_inner();
                let tag = std::str::from_utf8(tag_bytes)
                    .map_err(|err| ApiError::Parse(err.to_string()))?;

                if tag == "item" {
                    in_item = false;
                    let thumbnail_url = isbn13
                        .as_ref()
                        .map(|id| format!("{NDL_THUMBNAIL_BASE}/{id}.jpg"));

                    items.push(NdlItemModel {
                        title: title.take(),
                        creator: if creators.is_empty() {
                            None
                        } else {
                            Some(std::mem::take(&mut creators))
                        },
                        publisher: publisher.take(),
                        isbn: isbn.take(),
                        isbn13: isbn13.take(),
                        pub_date: pub_date.take(),
                        description: description.take(),
                        thumbnail_url,
                    });
                }

                current_tag = None;
                current_xsi_type = None;
            }

            Ok(Event::Eof) => break,
            Err(e) => return Err(ApiError::Parse(e.to_string())),
            _ => {}
        }
    }

    Ok(items)
}
