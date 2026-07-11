//! ExternalSearchService のエラー型・検索結果DTO定義
//!
//! TASK-0023: ExternalSearchServiceラッパー実装（media_type→provider振り分け）
//!
//! 検索結果のワイヤ表現は、旧 `models::domain::MediaDetails`（フル情報の正規化モデル）を廃止し、
//! カード表示に必要な最小限のフィールドのみを持つ [`SearchResultItem`] に一本化した。
//! 選択後の詳細取得・DB保存用JSON構築は `services::external_search` 側の
//! プロバイダ別変換関数が`CreateItemRequest`を直接構築する形へ移行した。

use serde::Serialize;

use crate::models::api_credential::ApiProvider;
use crate::models::item::MediaType;

/// `GET /items/search` のレスポンス要素（軽量DTO）
///
/// id/title/サムネイルURL + media_type/providerのみを保持する。詳細情報は
/// `POST /items/import`（`{media_type, provider, external_id}`）確定時にサーバー側で
/// 再取得する。
#[derive(Debug, Clone, Serialize)]
pub struct SearchResultItem {
    /// プロバイダ固有ID（mal_id/tmdb id/steam appid/annict work id/isbn等）
    pub id: String,
    pub media_type: MediaType,
    /// 採用したプロバイダ。キー不要プロバイダ（Jikan）は`None`
    pub provider: Option<ApiProvider>,
    /// タイトル（日本語優先）
    pub title: String,
    pub thumbnail_url: Option<String>,
    /// 発売・公開年（取得できない場合は`None`）
    pub year: Option<i32>,
}

/// ExternalSearchService が返すエラー型
///
/// 🔵 信頼性レベル: 要件定義書 第4章より
#[derive(Debug)]
pub enum ExternalSearchError {
    /// キー必須プロバイダでDBにキー未登録（TASK-0024で422へマッピング）
    ApiKeyNotConfigured(ApiProvider),
    /// api-client-lib の ApiError をラップ（TASK-0024で502へマッピング）
    ExternalApiError(api_client_lib::ApiError),
}

impl std::fmt::Display for ExternalSearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExternalSearchError::ApiKeyNotConfigured(provider) => {
                write!(f, "APIキーが未設定です: {provider:?}")
            }
            ExternalSearchError::ExternalApiError(err) => {
                write!(f, "外部APIエラー: {err}")
            }
        }
    }
}

impl std::error::Error for ExternalSearchError {}

#[cfg(test)]
mod tests {
    use super::*;

    /// ExternalSearchError::ApiKeyNotConfigured がproviderを保持しDisplayできる
    /// 🔵 信頼性レベル: 要件定義書 第4章より
    #[test]
    fn external_search_error_api_key_not_configured_holds_provider() {
        let err = ExternalSearchError::ApiKeyNotConfigured(ApiProvider::Tmdb);

        match &err {
            ExternalSearchError::ApiKeyNotConfigured(provider) => {
                assert_eq!(*provider, ApiProvider::Tmdb); // 【確認内容】: variantがproviderを正しく保持することを確認 🔵
            }
            _ => panic!("ApiKeyNotConfigured variantであるはず"),
        }
        assert!(!err.to_string().is_empty()); // 【確認内容】: Display実装が空文字を返さないことを確認 🔵
    }

    /// ExternalSearchError::ExternalApiError が内側のApiErrorを保持する
    /// 🔵 信頼性レベル: 要件定義書 第4章・EDGE-0023-04より
    #[test]
    fn external_search_error_external_api_error_holds_inner_error() {
        let err = ExternalSearchError::ExternalApiError(api_client_lib::ApiError::Timeout);

        match &err {
            ExternalSearchError::ExternalApiError(inner) => {
                assert!(matches!(inner, api_client_lib::ApiError::Timeout)); // 【確認内容】: 内側のApiErrorがTimeoutのまま保持されることを確認 🔵
            }
            _ => panic!("ExternalApiError variantであるはず"),
        }
    }
}
