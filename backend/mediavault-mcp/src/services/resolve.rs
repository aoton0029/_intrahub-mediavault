//! Service層: 名前→ID解決サービス（`MasterResolver`）
//!
//! TASK-0012: ItemSummary 縮約と名前→ID解決サービス

use crate::api::client::ApiClient;
use crate::api::error::ApiClientError;
use crate::api::models::{CategoryWithCount, Mylist, TagWithCount};
use crate::result::summary::NamedRef;

/// 候補名の提示上限。
///
/// 🟡 Intent: タスクファイル「実装項目5」より。全件返すとレスポンスが膨らむため上位10件に制限する。
const AVAILABLE_NAMES_LIMIT: usize = 10;

/// 名前解決の結果。
///
/// 🔵 Intent: タスクファイル「実装項目2」の型定義そのまま。
///    `ApiClientError` とは別の型にすることで「候補なし」と「api がエラーを返した」を区別する（REQ-111）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    Found(NamedRef),
    NotFound {
        requested: String,
        available: Vec<String>,
    },
}

/// タグ・カテゴリ・マイリストの名前→ID解決を行うサービス。
///
/// 🔵 Intent: architecture.md 状態管理「名前→ID解決はキャッシュしない」より。
///    呼び出しをまたぐキャッシュは持たず（別経路で作られたマスタを取りこぼさないため）、
///    1回の呼び出し内（= このインスタンスの生存期間内）でのみ取得結果をメモ化する。
///    ツール呼び出しごとに `MasterResolver::new` で生成し、呼び出し終了で破棄する運用を前提とする。
pub struct MasterResolver<'a> {
    api: &'a ApiClient,
    tags: Option<Vec<NamedRef>>,
    categories: Option<Vec<NamedRef>>,
    mylists: Option<Vec<NamedRef>>,
}

impl<'a> MasterResolver<'a> {
    pub fn new(api: &'a ApiClient) -> Self {
        Self {
            api,
            tags: None,
            categories: None,
            mylists: None,
        }
    }

    /// 🔵 Intent: `GET /api/v1/tags` を1回の呼び出しにつき最大1回のみ実行する（完了条件）。
    pub async fn resolve_tag(&mut self, name: &str) -> Result<Resolution, ApiClientError> {
        if self.tags.is_none() {
            let response = self
                .api
                .get::<Vec<TagWithCount>>("/api/v1/tags", &[])
                .await?;
            self.tags = Some(
                response
                    .data
                    .into_iter()
                    .map(|tag| NamedRef {
                        id: tag.id,
                        name: tag.name,
                    })
                    .collect(),
            );
        }
        Ok(resolve_from_master(self.tags.as_ref().unwrap(), name))
    }

    pub async fn resolve_category(&mut self, name: &str) -> Result<Resolution, ApiClientError> {
        if self.categories.is_none() {
            let response = self
                .api
                .get::<Vec<CategoryWithCount>>("/api/v1/categories", &[])
                .await?;
            self.categories = Some(
                response
                    .data
                    .into_iter()
                    .map(|category| NamedRef {
                        id: category.id,
                        name: category.name,
                    })
                    .collect(),
            );
        }
        Ok(resolve_from_master(self.categories.as_ref().unwrap(), name))
    }

    pub async fn resolve_mylist(&mut self, name: &str) -> Result<Resolution, ApiClientError> {
        if self.mylists.is_none() {
            let response = self.api.get::<Vec<Mylist>>("/api/v1/mylists", &[]).await?;
            self.mylists = Some(
                response
                    .data
                    .into_iter()
                    .map(|mylist| NamedRef {
                        id: mylist.id,
                        name: mylist.name,
                    })
                    .collect(),
            );
        }
        Ok(resolve_from_master(self.mylists.as_ref().unwrap(), name))
    }
}

/// マスタ一覧から名前を解決する（前後空白除去 + 完全一致、大文字小文字は区別する）。
///
/// 🟡 Intent: dataflow.md 機能4「前後空白除去 + 完全一致」より。部分一致は誤付与の
///    リスクがあるため採用しない（タスクファイル「注意事項」）。
fn resolve_from_master(master: &[NamedRef], requested: &str) -> Resolution {
    let trimmed = requested.trim();

    if let Some(found) = master.iter().find(|entry| entry.name == trimmed) {
        return Resolution::Found(found.clone());
    }

    Resolution::NotFound {
        requested: requested.to_string(),
        available: similar_names(master, trimmed),
    }
}

/// 解決失敗時の候補名を、前方一致・部分一致を優先した順で上位N件返す。
///
/// 🟡 Intent: タスクファイル「類似名の優先」より。AI が誤字に気付きやすいよう、
///    完全に無関係な名前より近い名前を先に提示する。`sort_by_key` は安定ソートのため、
///    同順位内ではマスタの元の順序を保つ。
fn similar_names(master: &[NamedRef], trimmed: &str) -> Vec<String> {
    let mut entries: Vec<&NamedRef> = master.iter().collect();
    entries.sort_by_key(|entry| {
        if entry.name.starts_with(trimmed) {
            0
        } else if entry.name.contains(trimmed) {
            1
        } else {
            2
        }
    });
    entries
        .into_iter()
        .take(AVAILABLE_NAMES_LIMIT)
        .map(|entry| entry.name.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn master(names: &[&str]) -> Vec<NamedRef> {
        names
            .iter()
            .map(|name| NamedRef {
                id: Uuid::new_v4(),
                name: name.to_string(),
            })
            .collect()
    }

    /// テストケース4: 名前の完全一致（部分一致するより長い候補には引きずられない）
    #[test]
    fn resolves_exact_match_over_similar_longer_name() {
        let master = master(&["積読", "積読候補"]);
        let resolution = resolve_from_master(&master, "積読");

        match resolution {
            Resolution::Found(named_ref) => assert_eq!(named_ref.name, "積読"),
            Resolution::NotFound { .. } => panic!("expected Found"),
        }
    }

    /// テストケース5: 前後空白の除去
    #[test]
    fn trims_whitespace_before_matching() {
        let master = master(&["積読"]);
        let resolution = resolve_from_master(&master, " 積読 ");

        match resolution {
            Resolution::Found(named_ref) => assert_eq!(named_ref.name, "積読"),
            Resolution::NotFound { .. } => panic!("expected Found"),
        }
    }

    /// テストケース6: 大文字小文字の区別
    #[test]
    fn case_sensitive_match_does_not_resolve_lowercase() {
        let master = master(&["SF"]);
        let resolution = resolve_from_master(&master, "sf");

        assert!(matches!(resolution, Resolution::NotFound { .. }));
    }

    /// テストケース7: 解決失敗時の候補提示
    #[test]
    fn not_found_includes_available_names_within_limit() {
        let names: Vec<String> = (0..15).map(|i| format!("タグ{i}")).collect();
        let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        let master = master(&name_refs);

        let resolution = resolve_from_master(&master, "存在しない名前");

        match resolution {
            Resolution::NotFound {
                requested,
                available,
            } => {
                assert_eq!(requested, "存在しない名前");
                assert!(available.len() <= AVAILABLE_NAMES_LIMIT);
                assert!(!available.is_empty());
            }
            Resolution::Found(_) => panic!("expected NotFound"),
        }
    }

    /// 類似名（前方一致）が候補の先頭に来る
    #[test]
    fn similar_names_prioritize_prefix_match() {
        let master = master(&["映画", "積読候補", "積読メモ", "SF"]);
        let resolution = resolve_from_master(&master, "積読");

        match resolution {
            Resolution::NotFound { available, .. } => {
                assert_eq!(available[0], "積読候補");
                assert_eq!(available[1], "積読メモ");
            }
            Resolution::Found(_) => panic!("expected NotFound"),
        }
    }
}
