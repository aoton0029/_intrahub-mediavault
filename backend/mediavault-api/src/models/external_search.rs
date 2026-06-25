//! ExternalSearchService の共通DTO・エラー型定義
//!
//! TASK-0023: ExternalSearchServiceラッパー実装（media_type→provider振り分け）
//!
//! 【信頼性レベル】: 🔵 要件定義書 第3章・第4章より

use serde::Serialize;

use crate::models::api_credential::ApiProvider;
use crate::models::item::MediaType;

/// 外部API検索結果の共通DTO
///
/// 【機能概要】: 各プロバイダ固有のModel型をアダプタで変換した共通表現。
/// 【設計判断】: `provider` フィールドは `ApiProvider` enumにJikanが含まれない（キー不要のため）ことから
/// `Option<ApiProvider>` を採用する（Jikan時は `None`）。要件定義書 第3章注記の2案のうち(a)を採用。
/// 🟡 信頼性レベル: 要件定義書 REQ-0023-06・第3章 出力仕様（ラップ形式は実装詳細未確定）より
#[derive(Debug, Clone, Serialize)]
pub struct ExternalSearchResult {
    pub media_type: MediaType,
    /// 採用したプロバイダ。Jikan（キー不要）の場合は `None`
    pub provider: Option<ApiProvider>,
    /// プロバイダ固有ID（jikan_id/tmdb_id等の元値）
    pub external_id: String,
    pub title: String,
    /// プロバイダ固有の生データ（`ApiResponse.raw` 由来）
    pub raw_data: serde_json::Value,
}

/// ExternalSearchService が返すエラー型
///
/// 🔵 信頼性レベル: 要件定義書 第4章より
#[derive(Debug)]
pub enum ExternalSearchError {
    /// キー必須プロバイダでDBにキー未登録（後続TASK-0024で422へマッピング）
    ApiKeyNotConfigured(ApiProvider),
    /// api-client-lib の ApiError をラップ（後続TASK-0024で502へマッピング）
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

    // ============================================================
    // 正常系（ユニット・型定義/シリアライズ）
    // ============================================================

    /// TC-002-RESULT（型定義部分）: ExternalSearchResult が想定フィールドを保持しシリアライズできる
    /// 🟡 信頼性レベル: 要件定義書 REQ-0023-06・第3章 出力仕様より
    #[test]
    fn external_search_result_constructs_and_serializes_with_expected_fields() {
        // 【テスト目的】: ExternalSearchResultの構築とシリアライズが要件契約どおりに機能するかを確認する
        // 【テスト内容】: 全フィールドへ代表値を設定し、JSON化した結果に主要フィールドが含まれるかを確認する
        // 【期待される動作】: media_type/provider/external_id/title/raw_dataがすべてJSON中に現れる
        // 🟡 信頼性レベル: 要件定義書 第3章 出力仕様（ラップ形式は実装詳細未確定）より

        let result = ExternalSearchResult {
            media_type: MediaType::Movie,
            provider: Some(ApiProvider::Tmdb),
            external_id: "603".to_string(),
            title: "The Matrix".to_string(),
            raw_data: serde_json::json!({"id": 603, "title": "The Matrix"}),
        };

        let json = serde_json::to_value(&result).expect("シリアライズは成功するはず"); // 【確認内容】: シリアライズ自体が失敗しないこと 🟡
        assert_eq!(json["media_type"], "movie"); // 【確認内容】: media_typeがsnake_caseで出力されることを確認 🟡
        assert_eq!(json["external_id"], "603"); // 【確認内容】: external_idがそのまま保持されることを確認 🟡
        assert_eq!(json["title"], "The Matrix"); // 【確認内容】: titleがそのまま保持されることを確認 🟡
        assert!(json["raw_data"]["id"] == 603); // 【確認内容】: raw_dataが生JSONをそのまま保持することを確認 🟡
    }

    /// TC-002-RESULT（Jikan表現確定）: provider=NoneでJikan結果を表現できる
    /// 🟡 信頼性レベル: 要件定義書 第3章注記（ApiProviderにJikanが含まれない設計判断）より
    #[test]
    fn external_search_result_represents_jikan_with_none_provider() {
        // 【テスト目的】: providerフィールドのJikan表現として確定したOption<ApiProvider>::Noneが構築可能かを確認する
        // 【テスト内容】: Anime/JikanのケースをproviderにNoneを設定して構築する
        // 【期待される動作】: providerがNoneのまま構築・シリアライズできる
        // 🟡 信頼性レベル: 要件定義書 第3章注記・tdd-red確定方式（Option<ApiProvider>）より

        let result = ExternalSearchResult {
            media_type: MediaType::Anime,
            provider: None,
            external_id: "16498".to_string(),
            title: "進撃の巨人".to_string(),
            raw_data: serde_json::json!({"mal_id": 16498}),
        };

        let json = serde_json::to_value(&result).expect("シリアライズは成功するはず"); // 【確認内容】: provider=Noneでもシリアライズが失敗しないことを確認 🟡
        assert!(json["provider"].is_null()); // 【確認内容】: providerがJSON上null（Jikan表現）になることを確認 🟡
    }

    /// ExternalSearchError::ApiKeyNotConfigured がproviderを保持しDisplayできる
    /// 🔵 信頼性レベル: 要件定義書 第4章より
    #[test]
    fn external_search_error_api_key_not_configured_holds_provider() {
        // 【テスト目的】: ApiKeyNotConfigured variantが指定providerを保持し、Display実装が機能するかを確認する
        // 【テスト内容】: ApiKeyNotConfigured(ApiProvider::Tmdb)を構築しDisplay出力を確認する
        // 【期待される動作】: matchでprovider==Tmdbを取り出せる。Display文字列が空でない
        // 🔵 信頼性レベル: 要件定義書 第4章 ExternalSearchError仕様より

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
        // 【テスト目的】: ExternalApiError variantがapi_client_lib::ApiErrorをラップして保持できるかを確認する
        // 【テスト内容】: ApiError::Timeoutをラップしたexternal_search_errorを構築する
        // 【期待される動作】: matchで内側のApiErrorがTimeoutであることを取り出せる
        // 🔵 信頼性レベル: 要件定義書 第4章・EDGE-0023-04（全ApiError集約）より

        let err = ExternalSearchError::ExternalApiError(api_client_lib::ApiError::Timeout);

        match &err {
            ExternalSearchError::ExternalApiError(inner) => {
                assert!(matches!(inner, api_client_lib::ApiError::Timeout)); // 【確認内容】: 内側のApiErrorがTimeoutのまま保持されることを確認 🔵
            }
            _ => panic!("ExternalApiError variantであるはず"),
        }
    }
}
