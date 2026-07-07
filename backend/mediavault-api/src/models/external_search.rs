//! ExternalSearchService のエラー型定義
//!
//! TASK-0023: ExternalSearchServiceラッパー実装（media_type→provider振り分け）
//!
//! 検索結果のワイヤ表現は `models::domain::MediaDetails`（ノーマライズ済みドメインモデル）に
//! 一本化されたため、旧 `ExternalSearchResult`（raw_data同梱DTO）は廃止した。

use crate::models::api_credential::ApiProvider;

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
