use serde_json::Value;

use crate::error::ApiError;

/// OpenBD 書誌1件分のうち、ジャンル判定に必要な最小限のデータ。
#[derive(Debug, Clone, Default)]
pub struct OpenBdItemModel {
    /// Cコード（日本図書コード、4桁）。取得できない場合はNone。
    pub c_code: Option<String>,
}

/// OpenBD クライアントが返すモデル列挙型。
#[derive(Debug)]
pub enum OpenBdModel {
    /// `get`の結果。リクエストしたISBN順に並び、該当なしはNone。
    Items(Vec<Option<OpenBdItemModel>>),
}

/// OpenBD `GET /get` のJSONレスポンス（配列、要素はnull可）をパースする。
pub(super) fn parse_openbd_response(json: &str) -> Result<Vec<Option<OpenBdItemModel>>, ApiError> {
    let raw: Vec<Option<Value>> =
        serde_json::from_str(json).map_err(|e| ApiError::Parse(e.to_string()))?;

    Ok(raw
        .into_iter()
        .map(|item| {
            item.map(|value| OpenBdItemModel {
                c_code: extract_c_code(&value),
            })
        })
        .collect())
}

fn extract_c_code(value: &Value) -> Option<String> {
    let subjects = value
        .pointer("/onix/DescriptiveDetail/Subject")?
        .as_array()?;

    subjects.iter().find_map(|subject| {
        let scheme = subject.get("SubjectSchemeIdentifier")?.as_str()?;
        if scheme == "78" {
            subject.get("SubjectCode")?.as_str().map(str::to_string)
        } else {
            None
        }
    })
}

/// Cコードの内容分類桁（3桁目、0始まりindex 2）を取り出す。
///
/// 日本図書コードのCコードは4桁固定（1桁目=読者対象、2桁目=発行形態、
/// 3〜4桁目=内容分類）で構成されており、3桁目が内容分類の十の位
/// （0=総記〜9=文芸）を表す。9のとき文芸（小説）、それ以外は専門分野を示す。
pub fn content_category_digit(c_code: &str) -> Option<u8> {
    c_code.chars().nth(2)?.to_digit(10).map(|d| d as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_c_code_from_subject_array() {
        let json = r#"[
            {
                "onix": {
                    "DescriptiveDetail": {
                        "Subject": [
                            {"SubjectSchemeIdentifier": "78", "SubjectCode": "0293"},
                            {"SubjectSchemeIdentifier": "01", "SubjectCode": "999"}
                        ]
                    }
                }
            },
            null
        ]"#;

        let items = parse_openbd_response(json).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].as_ref().unwrap().c_code, Some("0293".to_string()));
        assert!(items[1].is_none());
    }

    #[test]
    fn returns_none_c_code_when_subject_missing() {
        let json = r#"[{"onix": {"DescriptiveDetail": {}}}]"#;
        let items = parse_openbd_response(json).unwrap();
        assert_eq!(items[0].as_ref().unwrap().c_code, None);
    }

    #[test]
    fn content_category_digit_reads_third_character() {
        assert_eq!(content_category_digit("0293"), Some(9));
        assert_eq!(content_category_digit("1000"), Some(0));
        assert_eq!(content_category_digit(""), None);
    }
}
