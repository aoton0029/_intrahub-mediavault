//! 統一APIレスポンス型・共通エラー型
//!
//! TASK-0005: 共通エラー型・統一APIレスポンス実装

use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Serialize;

use crate::models::external_search::ExternalSearchError;

/// 統一APIレスポンス（成功）
#[derive(Debug, Clone, Serialize)]
pub struct ApiOk<T> {
    pub success: bool,
    pub data: T,
}

impl<T> ApiOk<T> {
    /// success=true で ApiOk を構築するコンストラクタ
    pub fn new(data: T) -> Self {
        Self {
            success: true,
            data,
        }
    }
}

impl<T: Serialize> IntoResponse for ApiOk<T> {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

/// 統一エラーレスポンス
#[derive(Debug, Clone, Serialize)]
pub struct ApiError {
    pub success: bool,
    pub error: ApiErrorBody,
    #[serde(skip)]
    pub status: StatusCode,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
}

/// エラーコード一覧
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiErrorCode {
    ValidationError,
    Unauthorized,
    ItemNotFound,
    UnprocessableEntity,
    InternalError,
    ExternalApiError,
    /// タグ名の一意制約違反（TASK-0015）
    /// 🟡 信頼性レベル: TASK-0015.md 注意事項「例: DUPLICATE_TAG_NAME」より新規追加
    DuplicateTagName,
    /// タグが見つからない（TASK-0015のDELETE/付与時404）
    /// 🟡 信頼性レベル: 既存ItemNotFoundパターンからタグ専用に分離した妥当な推測
    TagNotFound,
    /// カテゴリ名の一意制約違反（TASK-0015）
    /// 🟡 信頼性レベル: TASK-0015.md 注意事項「例: DUPLICATE_CATEGORY_NAME」より新規追加
    DuplicateCategoryName,
    /// カテゴリが見つからない（TASK-0015のDELETE/付与時404）
    /// 🟡 信頼性レベル: TagNotFoundと対称な推測
    CategoryNotFound,
    /// マイリストが見つからない（TASK-0016のitem追加時404）
    /// 🟡 信頼性レベル: TagNotFound/CategoryNotFoundと対称な推測
    MylistNotFound,
    /// 関連付けの一意制約違反（TASK-0017、同一(item_id, related_item_id, relation_type)組み合わせ）
    /// 🟡 信頼性レベル: DuplicateTagName/DuplicateCategoryNameと対称な推測
    DuplicateRelation,
    /// グループが見つからない（TASK-0019のepisode登録・一覧時404）
    /// 🟡 信頼性レベル: TagNotFound/CategoryNotFoundと対称な推測
    GroupNotFound,
    /// volume配下のグループへのepisode登録拒否（TASK-0019、EDGE-101）
    /// 🔵 信頼性レベル: TASK-0019.md 完了条件「INVALID_GROUP_TYPE_FOR_EPISODES（400）」に直接対応
    InvalidGroupTypeForEpisodes,
    /// 同一group_id内のepisode_number重複（TASK-0019、uq_item_episodes制約違反）
    /// 🟡 信頼性レベル: TASK-0019.md テストケース3「一意制約違反に対応したエラー」より
    DuplicateEpisodeNumber,
    /// スタッフが見つからない（TASK-0020のitem紐付け時404）
    /// 🟡 信頼性レベル: staff-requirements.md 3 制約条件「STAFF_NOT_FOUND新規追加」に明記
    StaffNotFound,
    /// キャストが見つからない（cast管理のitem紐付け時404）
    CastNotFound,
    /// 不正なapi_credentialsプロバイダ指定（TASK-0022のPUT /settings/api-keys/:provider時400）
    /// 🔵 信頼性レベル: TASK-0022-requirements.md REQ-0022-102・note.md L19に直接対応
    InvalidProvider,
    /// キー必須プロバイダでDBにAPIキー未登録（TASK-0024のGET /items/search時422）
    /// 🔵 信頼性レベル: TASK-0024 item-search-requirements.md 第3章（422 API_KEY_NOT_CONFIGURED新規variant必要）に直接対応
    ApiKeyNotConfigured,
    /// 外部APIタイムアウト・障害（TASK-0024のGET /items/search時502。既存ExternalApiErrorとはコード文字列が異なる）
    /// 🔵 信頼性レベル: TASK-0024 item-search-requirements.md 第3章（502 EXTERNAL_API_TIMEOUT新規variant必要・既存と文字列不一致明記）に直接対応
    ExternalApiTimeout,
    /// 同一(media_type, external_id)のitemが既に存在する場合の重複インポート（TASK-0025のPOST /items/import時409）
    /// 🟡 信頼性レベル: item-import-requirements.md 第6章（決定: 409 ITEM_ALREADY_IMPORTED）・既存DuplicateTagName等409系パターンより
    ItemAlreadyImported,
    /// ファイルサーバーへの書込失敗（TASK-0027のPOST /items/:id/files/upload時500、EDGE-003）
    /// 🟡 信頼性レベル: TASK-0027 item-file-upload-requirements.md 第2章注記（response.rsに未定義・新規追加必要）・
    /// item-file-upload-testcases.md 第0章「エラーコード前提」より新規追加
    FileStorageWriteFailed,
    /// 指定されたitem_files行が見つからない（不存在・item_id/file_id紐付け不一致）（TASK-0028のPATCH calibre-link時404）
    /// 🟡 信頼性レベル: TASK-0028 calibre-link-requirements.md 2.2 L67-71・第6章「response.rsに未定義・新規追加必要」に対応
    FileNotFound,
    /// Steam APIキー未設定・無効（TASK-0031のPOST /import/steam時401、TC-017-E01）
    /// 🔵 信頼性レベル: TASK-0031.md「テストケース2（TC-017-E01）」・steam-import-requirements.md 3章「STEAM_API_KEY_INVALID（401）」に直接対応
    SteamApiKeyInvalid,
    /// 配信URLの一意制約違反（同一(item_id, platform)が既に登録済み）
    DuplicateStreamingLink,
    /// バックアップファイルのschema_versionが未対応（POST /backup/import時400）
    UnsupportedBackupVersion,
    /// 指定されたブクログインポートジョブが見つからない（GET /import/booklog/jobs/:job_id時404）
    ImportJobNotFound,
}

impl ApiErrorCode {
    /// エラーコード文字列とHTTPステータスコードの対応表
    fn code_and_status(&self) -> (&'static str, StatusCode) {
        match self {
            ApiErrorCode::ValidationError => ("VALIDATION_ERROR", StatusCode::BAD_REQUEST),
            ApiErrorCode::Unauthorized => ("UNAUTHORIZED", StatusCode::UNAUTHORIZED),
            ApiErrorCode::ItemNotFound => ("ITEM_NOT_FOUND", StatusCode::NOT_FOUND),
            ApiErrorCode::UnprocessableEntity => {
                ("UNPROCESSABLE_ENTITY", StatusCode::UNPROCESSABLE_ENTITY)
            }
            ApiErrorCode::InternalError => ("INTERNAL_ERROR", StatusCode::INTERNAL_SERVER_ERROR),
            ApiErrorCode::ExternalApiError => ("EXTERNAL_API_ERROR", StatusCode::BAD_GATEWAY),
            ApiErrorCode::DuplicateTagName => ("DUPLICATE_TAG_NAME", StatusCode::CONFLICT),
            ApiErrorCode::TagNotFound => ("TAG_NOT_FOUND", StatusCode::NOT_FOUND),
            ApiErrorCode::DuplicateCategoryName => {
                ("DUPLICATE_CATEGORY_NAME", StatusCode::CONFLICT)
            }
            ApiErrorCode::CategoryNotFound => ("CATEGORY_NOT_FOUND", StatusCode::NOT_FOUND),
            ApiErrorCode::MylistNotFound => ("MYLIST_NOT_FOUND", StatusCode::NOT_FOUND),
            ApiErrorCode::DuplicateRelation => ("DUPLICATE_RELATION", StatusCode::CONFLICT),
            ApiErrorCode::GroupNotFound => ("GROUP_NOT_FOUND", StatusCode::NOT_FOUND),
            ApiErrorCode::InvalidGroupTypeForEpisodes => {
                ("INVALID_GROUP_TYPE_FOR_EPISODES", StatusCode::BAD_REQUEST)
            }
            ApiErrorCode::DuplicateEpisodeNumber => {
                ("DUPLICATE_EPISODE_NUMBER", StatusCode::CONFLICT)
            }
            ApiErrorCode::StaffNotFound => ("STAFF_NOT_FOUND", StatusCode::NOT_FOUND),
            ApiErrorCode::CastNotFound => ("CAST_NOT_FOUND", StatusCode::NOT_FOUND),
            ApiErrorCode::InvalidProvider => ("INVALID_PROVIDER", StatusCode::BAD_REQUEST),
            ApiErrorCode::ApiKeyNotConfigured => {
                ("API_KEY_NOT_CONFIGURED", StatusCode::UNPROCESSABLE_ENTITY)
            }
            ApiErrorCode::ExternalApiTimeout => ("EXTERNAL_API_TIMEOUT", StatusCode::BAD_GATEWAY),
            ApiErrorCode::ItemAlreadyImported => ("ITEM_ALREADY_IMPORTED", StatusCode::CONFLICT),
            ApiErrorCode::FileStorageWriteFailed => (
                "FILE_STORAGE_WRITE_FAILED",
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            ApiErrorCode::FileNotFound => ("FILE_NOT_FOUND", StatusCode::NOT_FOUND),
            ApiErrorCode::SteamApiKeyInvalid => ("STEAM_API_KEY_INVALID", StatusCode::UNAUTHORIZED),
            ApiErrorCode::DuplicateStreamingLink => {
                ("DUPLICATE_STREAMING_LINK", StatusCode::CONFLICT)
            }
            ApiErrorCode::UnsupportedBackupVersion => {
                ("UNSUPPORTED_BACKUP_VERSION", StatusCode::BAD_REQUEST)
            }
            ApiErrorCode::ImportJobNotFound => ("IMPORT_JOB_NOT_FOUND", StatusCode::NOT_FOUND),
        }
    }

    /// エラーコード文字列（VALIDATION_ERROR等）
    pub fn as_code_str(&self) -> &'static str {
        self.code_and_status().0
    }

    /// 対応するHTTPステータスコード
    pub fn status_code(&self) -> StatusCode {
        self.code_and_status().1
    }
}

impl ApiError {
    /// エラーコードとメッセージから ApiError を構築するコンストラクタ
    pub fn new(code: ApiErrorCode, message: impl Into<String>) -> Self {
        Self {
            success: false,
            error: ApiErrorBody {
                code: code.as_code_str().to_string(),
                message: message.into(),
            },
            status: code.status_code(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = self.status;
        (status, Json(self)).into_response()
    }
}

/// `ExternalSearchError` を `ApiError` へ変換する（TASK-0024）
///
/// 【機能概要】: ExternalSearchService::search が返すエラーを、GET /items/search ハンドラの
/// `?` 演算子による自動伝播で統一APIエラー形式へマッピングする
/// 【実装方針】: ApiKeyNotConfigured(provider) → 422 API_KEY_NOT_CONFIGURED、
/// ExternalApiError(api_client_lib::ApiError) → 502 EXTERNAL_API_TIMEOUT（全variant一律集約）。
/// メッセージには外部API生エラー詳細・DB内部情報を含めない（情報漏洩防止NFR-0024-02）
/// 【テスト対応】: TC-0024-E01（422変換）, TC-0024-E02（502変換）, TC-0024-E03（全variant集約）
/// 🔵 信頼性レベル: 要件定義書 第3章・第4章・note.md L91・EDGE-0023-04に直接対応
impl From<ExternalSearchError> for ApiError {
    fn from(err: ExternalSearchError) -> Self {
        match err {
            ExternalSearchError::ApiKeyNotConfigured(_provider) => {
                ApiError::new(ApiErrorCode::ApiKeyNotConfigured, "APIキーが未設定です")
            }
            ExternalSearchError::ExternalApiError(_inner) => ApiError::new(
                ApiErrorCode::ExternalApiTimeout,
                "外部APIへの接続に失敗しました",
            ),
        }
    }
}

/// ページネーション情報（keyset/cursorページネーション）
///
/// 【機能概要】: 一覧取得APIで返すlimit/has_more/次カーソル（created_at, id）を保持する
/// 【実装方針】: page/offset方式からcursor方式へ変更し、COUNT(*)クエリを不要にする。
/// next_after_created_at/next_after_idはhas_more=falseの場合はNoneとなる
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Pagination {
    pub limit: u32,
    pub has_more: bool,
    pub next_after_created_at: Option<chrono::NaiveDateTime>,
    pub next_after_id: Option<uuid::Uuid>,
    /// sortがcreated_at以外の場合の次ページカーソル（ソートキー値の文字列表現）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_after_value: Option<String>,
    /// 検索条件に該当する全件数（TASK-0003）。`include_total=true`指定時のみSome。
    /// 未指定時はフィールドごと省略し、既存レスポンスの形を変えない
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}

/// ページネーション付き統一APIレスポンス（成功）
///
/// 【機能概要】: 一覧取得APIの成功レスポンスを `{ success, data, pagination }` 形式で返す
/// 【実装方針】: 既存`ApiOk<T>`と同様にIntoResponseで200固定とし、pagination情報を追加で保持する
/// 【テスト対応】: TC-0010-N09（シリアライズ形式）, TC-0010-N10（200応答）を通すための実装
/// 🟡 信頼性レベル: テストケース定義書 確定1・要件定義書2.2 のレスポンス形式に対応
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedOk<T> {
    pub success: bool,
    pub data: T,
    pub pagination: Pagination,
}

impl<T> PaginatedOk<T> {
    /// success=true で PaginatedOk を構築するコンストラクタ
    pub fn new(data: T, pagination: Pagination) -> Self {
        Self {
            success: true,
            data,
            pagination,
        }
    }
}

impl<T: Serialize> IntoResponse for PaginatedOk<T> {
    fn into_response(self) -> axum::response::Response {
        // 【レスポンス構築】: 一覧取得は常に200固定（空配列でも404等にしない） 🔵
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::api_credential::ApiProvider;
    use axum::response::IntoResponse;

    /// TC1: VALIDATION_ERROR が 400 を返す
    #[test]
    fn validation_error_returns_400() {
        let err = ApiError::new(ApiErrorCode::ValidationError, "invalid input");
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    /// TC2: UNAUTHORIZED が 401 を返す
    #[test]
    fn unauthorized_returns_401() {
        let err = ApiError::new(ApiErrorCode::Unauthorized, "unauthorized");
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// TC3: ITEM_NOT_FOUND が 404 を返す
    #[test]
    fn item_not_found_returns_404() {
        let err = ApiError::new(ApiErrorCode::ItemNotFound, "item not found");
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    /// TC4: UNPROCESSABLE_ENTITY が 422 を返す
    #[test]
    fn unprocessable_entity_returns_422() {
        let err = ApiError::new(ApiErrorCode::UnprocessableEntity, "unprocessable");
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    /// TC5: INTERNAL_ERROR が 500 を返す
    #[test]
    fn internal_error_returns_500() {
        let err = ApiError::new(ApiErrorCode::InternalError, "internal error");
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    /// TC6: EXTERNAL_API_ERROR が 502 を返す
    #[test]
    fn external_api_error_returns_502() {
        let err = ApiError::new(ApiErrorCode::ExternalApiError, "external api error");
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    }

    /// TC7: ApiOk の JSON形式が {"success": true, "data": ...} になる
    #[test]
    fn api_ok_serializes_to_expected_json() {
        let ok = ApiOk::new(serde_json::json!({"id": 1, "name": "test"}));
        let json = serde_json::to_value(&ok).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "success": true,
                "data": {"id": 1, "name": "test"}
            })
        );
    }

    /// TC-0010-N09: PaginatedOk<T> のJSONシリアライズ形式
    /// 🟡 信頼性レベル: 確定1・要件2.2 のレスポンス形式に対応
    #[test]
    fn paginated_ok_serializes_to_expected_json() {
        // 【テスト目的】: 新規 PaginatedOk<T> が {success, data, pagination} 形式でシリアライズされることを確認する
        // 【テスト内容】: PaginatedOk::new(data, Pagination{...}) を構築し、serde_json::to_value の結果を検証する
        // 【期待される動作】: トップレベルキーが success/data/pagination、pagination内が page/limit/total になる
        // 🟡 信頼性レベル: 要件定義書 2.2 補足・テストケース定義 確定1 に対応（PaginatedOk/Paginationは本タスクで新規追加予定の未実装型）

        // 【テストデータ準備】: ApiOkの既存テストに倣った最小データ（item 1件 + pagination情報）
        // 【初期条件設定】: PaginatedOk/Pagination はまだ models/response.rs に存在しないため、この呼び出し自体がコンパイルエラーとなる想定
        let body = PaginatedOk::new(
            vec![serde_json::json!({"id": 1})],
            Pagination {
                limit: 20,
                has_more: true,
                next_after_created_at: Some(
                    chrono::NaiveDate::from_ymd_opt(2026, 7, 1)
                        .unwrap()
                        .and_hms_opt(12, 0, 0)
                        .unwrap(),
                ),
                next_after_id: Some(
                    uuid::Uuid::parse_str("b2b5c1a0-0000-0000-0000-000000000000").unwrap(),
                ),
                next_after_value: None,
                total: None,
            },
        );

        // 【実際の処理実行】: serde_json::to_value でシリアライズする
        // 【処理内容】: PaginatedOk<T> の Serialize 実装による JSON 変換
        let json = serde_json::to_value(&body).unwrap();

        // 【結果検証】: トップレベルキーとpagination構造がcursorページネーション仕様通りであることを確認
        assert_eq!(
            json,
            serde_json::json!({
                "success": true,
                "data": [{"id": 1}],
                "pagination": {
                    "limit": 20,
                    "has_more": true,
                    "next_after_created_at": "2026-07-01T12:00:00",
                    "next_after_id": "b2b5c1a0-0000-0000-0000-000000000000"
                }
            })
        ); // 【確認内容】: success/data/paginationの3キー構成、pagination内のフィールド名がcursor仕様通りであることを確認
    }

    /// TC-NEW-10: `ApiErrorCode::InvalidProvider`が400ステータスにマッピングされる
    /// 🔵 信頼性レベル: 要件定義書REQ-0022-102・完了基準L125、note.md L19より
    #[test]
    fn invalid_provider_returns_400() {
        // 【テスト目的】: 新規追加ApiErrorCode::InvalidProviderがIntoResponseでHTTP 400を返すことを確認する
        // 【テスト内容】: ApiError::new(ApiErrorCode::InvalidProvider, ...)を構築し、into_response()のステータスを検証する
        // 【期待される動作】: StatusCode::BAD_REQUEST（400）が返る。既存400系（ValidationError等）と同一であること
        // 🔵 信頼性レベル: 要件定義書REQ-0022-102・note.md L19（既存DuplicateTagName等のパターン踏襲）より

        // 【テストデータ準備】: 不正provider時のエラー応答生成を想定したApiError構築
        // 【初期条件設定】: 特になし
        let err = ApiError::new(ApiErrorCode::InvalidProvider, "不正なproviderです");

        // 【実際の処理実行】: into_response()でAxumレスポンスへ変換する
        // 【処理内容】: IntoResponse実装によるHTTPレスポンス構築
        let response = err.into_response();

        // 【結果検証】: ステータスコードが400であることを確認する
        // 【期待値確認】: 新規バリアントが500（デフォルト）等に誤マッピングされていないことの確認
        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: InvalidProviderが既存400系と同一のBAD_REQUESTにマッピングされることを確認 🔵
    }

    /// TC-0010-N10: PaginatedOk<T> が HTTP 200 を返す
    /// 🔵 信頼性レベル: 要件 2.2・既存 ApiOk IntoResponse 規約に直接対応
    #[test]
    fn paginated_ok_returns_200_even_when_data_is_empty() {
        // 【テスト目的】: PaginatedOk<T>::into_response() が常にステータス200を返すことを確認する
        // 【テスト内容】: 空配列のdataとtotal=0のPaginationでPaginatedOkを構築し、レスポンスのステータスを検証する
        // 【期待される動作】: 空配列でも404等にせず200を返す
        // 🔵 信頼性レベル: 要件定義書 2.2「HTTPステータス 200 OK」、ApiOkのIntoResponse実装（200固定）に直接対応

        // 【テストデータ準備】: 0件データの境界ケース（TC-0010-B08とも対応する状況を想定）
        // 【初期条件設定】: PaginatedOkはまだ未実装のため、この呼び出し自体がコンパイルエラーとなる想定
        let body: PaginatedOk<Vec<serde_json::Value>> = PaginatedOk::new(
            Vec::new(),
            Pagination {
                limit: 20,
                has_more: false,
                next_after_created_at: None,
                next_after_id: None,
                next_after_value: None,
                total: None,
            },
        );

        // 【実際の処理実行】: into_response()でAxumレスポンスへ変換する
        // 【処理内容】: IntoResponse実装によるHTTPレスポンス構築
        let response = body.into_response();

        // 【結果検証】: ステータスコードが200であることを確認
        // 【期待値確認】: 空配列でも200を返す既存ApiOk規約の継続であることの確認
        assert_eq!(response.status(), StatusCode::OK); // 【確認内容】: 空データでも成功時ステータスが200で固定されていることを確認 🔵
    }

    /// TC-0024-E01: ApiKeyNotConfigured → ApiError(422) 変換
    /// 🔵 信頼性レベル: 要件 3（422 新規 variant 必要）・external_search.rs `ApiKeyNotConfigured` variant・note.md L91 より
    #[test]
    fn external_search_error_api_key_not_configured_converts_to_422() {
        // 【テスト目的】: ExternalSearchError::ApiKeyNotConfiguredがApiError(422 API_KEY_NOT_CONFIGURED)へ
        // 変換されることを確認する
        // 【テスト内容】: ApiKeyNotConfigured(ApiProvider::Tmdb)をFrom変換し、ステータス・コード文字列を検証する
        // 【期待される動作】: status==422 UNPROCESSABLE_ENTITY、error.code=="API_KEY_NOT_CONFIGURED"
        // 🔵 信頼性レベル: 要件 3（422 新規 variant 必要）・note.md L91 より
        // 【Red期待】: ApiErrorCode::ApiKeyNotConfigured/From<ExternalSearchError>が未実装のためコンパイルエラーとなる想定

        // 【テストデータ準備】: キー必須プロバイダ（Tmdb）にキー未登録の状況を再現する
        // 【初期条件設定】: ExternalSearchError::ApiKeyNotConfigured(Tmdb)を直接構築する
        let err = ExternalSearchError::ApiKeyNotConfigured(ApiProvider::Tmdb);

        // 【実際の処理実行】: From<ExternalSearchError> for ApiErrorによる変換を実行する
        // 【処理内容】: ExternalSearchError→ApiErrorのマッピングロジックを呼び出す
        let api_error: ApiError = err.into();

        // 【結果検証】: ステータス・エラーコード文字列が要件どおりであることを確認
        // 【期待値確認】: 既存UnprocessableEntity（"UNPROCESSABLE_ENTITY"）と文字列が異なる専用コードであることを保証
        assert_eq!(api_error.status, StatusCode::UNPROCESSABLE_ENTITY); // 【確認内容】: HTTPステータスが422であることを確認 🔵
        assert_eq!(api_error.error.code, "API_KEY_NOT_CONFIGURED"); // 【確認内容】: ワイヤーコードがAPI_KEY_NOT_CONFIGUREDであることを確認 🔵
    }

    /// TC-0024-E02: ExternalApiError(Timeout) → ApiError(502) 変換
    /// 🔵 信頼性レベル: 要件 3（502 新規 variant 必要・文字列不一致明記）・note.md L91 より
    #[test]
    fn external_search_error_timeout_converts_to_502() {
        // 【テスト目的】: ExternalSearchError::ExternalApiError(Timeout)がApiError(502 EXTERNAL_API_TIMEOUT)へ
        // 変換されることを確認する
        // 【テスト内容】: ExternalApiError(api_client_lib::ApiError::Timeout)をFrom変換し、ステータス・コード文字列を検証する
        // 【期待される動作】: status==502 BAD_GATEWAY、error.code=="EXTERNAL_API_TIMEOUT"
        // 🔵 信頼性レベル: 要件 3（502 新規 variant 必要・文字列不一致明記）・note.md L91 より
        // 【Red期待】: ApiErrorCode::ExternalApiTimeout/From<ExternalSearchError>が未実装のためコンパイルエラーとなる想定

        // 【テストデータ準備】: 外部APIタイムアウトの状況を再現する
        // 【初期条件設定】: ExternalSearchError::ExternalApiError(Timeout)を直接構築する
        let err = ExternalSearchError::ExternalApiError(api_client_lib::ApiError::Timeout);

        // 【実際の処理実行】: From<ExternalSearchError> for ApiErrorによる変換を実行する
        let api_error: ApiError = err.into();

        // 【結果検証】: ステータス・エラーコード文字列が要件どおりであることを確認
        // 【期待値確認】: 既存ExternalApiError（"EXTERNAL_API_ERROR"）とコード文字列が異なる新コードであることを保証
        assert_eq!(api_error.status, StatusCode::BAD_GATEWAY); // 【確認内容】: HTTPステータスが502であることを確認 🔵
        assert_eq!(api_error.error.code, "EXTERNAL_API_TIMEOUT"); // 【確認内容】: ワイヤーコードがEXTERNAL_API_TIMEOUTであることを確認 🔵
    }

    /// TC-0024-E03: api_client_lib::ApiError 6 variant全集約 → 502
    /// 🟡 信頼性レベル: 要件 3「全 variant を一律 502 へ集約」・EDGE-0023-04 より妥当推測
    #[test]
    fn external_search_error_all_six_api_error_variants_converge_to_502() {
        // 【テスト目的】: Http/Auth/RateLimit/Parse/Timeout/Networkのいずれの外部エラーも
        // 一律502 EXTERNAL_API_TIMEOUTへ集約されることを確認する
        // 【テスト内容】: 6つのapi_client_lib::ApiError variantをそれぞれExternalSearchError::ExternalApiErrorで
        // ラップし、From変換結果のステータス・コードを検証する（パラメータ化テスト）
        // 【期待される動作】: すべてstatus==502、error.code=="EXTERNAL_API_TIMEOUT"
        // 🟡 信頼性レベル: 要件 3「全 variant を一律 502 へ集約」・EDGE-0023-04 より妥当推測
        // 【Red期待】: ApiErrorCode::ExternalApiTimeout/From<ExternalSearchError>が未実装のためコンパイルエラーとなる想定

        // 【テストデータ準備】: api_client_lib::ApiErrorの全6 variantを用意する
        // 【初期条件設定】: レート制限超過・認証失敗・パース失敗など外部API側の多様な障害を再現する
        let variants = vec![
            api_client_lib::ApiError::Http {
                status: 500,
                body: "internal error".to_string(),
            },
            api_client_lib::ApiError::Auth("auth failed".to_string()),
            api_client_lib::ApiError::RateLimit { retry_after: None },
            api_client_lib::ApiError::Parse("parse failed".to_string()),
            api_client_lib::ApiError::Timeout,
            api_client_lib::ApiError::Network("network failed".to_string()),
        ];

        for variant in variants {
            // 【実際の処理実行】: 各variantをExternalApiErrorでラップし、ApiErrorへ変換する
            let err = ExternalSearchError::ExternalApiError(variant);
            let api_error: ApiError = err.into();

            // 【結果検証】: すべてのvariantで502・EXTERNAL_API_TIMEOUTへ収束することを確認
            // 【品質保証の観点】: 新variant追加時の取りこぼし（500へ落ちる等）を防ぐ
            assert_eq!(api_error.status, StatusCode::BAD_GATEWAY); // 【確認内容】: 全variantでHTTPステータスが502であることを確認 🟡
            assert_eq!(api_error.error.code, "EXTERNAL_API_TIMEOUT"); // 【確認内容】: 全variantでワイヤーコードがEXTERNAL_API_TIMEOUTであることを確認 🟡
        }
    }

    /// TC-0025-E07: ApiErrorCode::ItemAlreadyImportedが409 CONFLICT・ITEM_ALREADY_IMPORTEDへマッピングされる
    /// 【テスト目的】: 新規追加するエラーコードvariantのステータス・文字列マッピング検証
    /// 【テスト内容】: ApiError::new(ApiErrorCode::ItemAlreadyImported, ...)を構築し、into_response()のステータス・
    /// ワイヤーコードを検証する
    /// 【期待される動作】: status==409 CONFLICT、error.code=="ITEM_ALREADY_IMPORTED"
    /// 🔵 信頼性レベル: item-import-requirements.md 3.3、既存ApiErrorCodeの409系パターン
    /// （DuplicateTagName等）に直接対応
    #[test]
    fn item_already_imported_returns_409() {
        // 【テストデータ準備】: 重複インポート時のエラー応答生成を想定したApiError構築
        // 【初期条件設定】: 特になし
        let err = ApiError::new(ApiErrorCode::ItemAlreadyImported, "既にインポート済みです");

        // 【実際の処理実行】: into_response()でAxumレスポンスへ変換する
        // 【処理内容】: IntoResponse実装によるHTTPレスポンス構築
        let response = err.into_response();

        // 【結果検証】: ステータスコードが409であることを確認する
        // 【期待値確認】: 新規バリアントが500（デフォルト）等に誤マッピングされていないことの確認
        assert_eq!(response.status(), StatusCode::CONFLICT); // 【確認内容】: ItemAlreadyImportedが409 CONFLICTにマッピングされることを確認 🔵
    }

    /// TC-019-E01-CODE: ApiErrorCode::FileStorageWriteFailedが500・FILE_STORAGE_WRITE_FAILEDへマッピングされる
    /// 【テスト目的】: TASK-0027新規追加のFileStorageWriteFailedバリアントが、HTTP 500・
    /// ワイヤーコードFILE_STORAGE_WRITE_FAILEDに正しくマッピングされることを確認する
    /// 【テスト内容】: ApiError::new(ApiErrorCode::FileStorageWriteFailed, ...)を構築し、
    /// into_response()のステータスとerror.code文字列を検証する
    /// 【期待される動作】: status==500 INTERNAL_SERVER_ERROR、error.code=="FILE_STORAGE_WRITE_FAILED"
    /// 🟡 信頼性レベル: item-file-upload-testcases.md 第0章「エラーコード前提」・
    /// item-file-upload-requirements.md 第2章注記より新規追加（Red期待: バリアント未追加のためコンパイルエラー）
    #[test]
    fn file_storage_write_failed_returns_500_with_expected_wire_code() {
        // 【テストデータ準備】: ファイルサーバー書込失敗時のエラー応答生成を想定したApiError構築
        // 【初期条件設定】: 特になし
        let err = ApiError::new(
            ApiErrorCode::FileStorageWriteFailed,
            "ファイル書込に失敗しました",
        );

        // 【実際の処理実行】: into_response()でAxumレスポンスへ変換する
        // 【処理内容】: IntoResponse実装によるHTTPレスポンス構築
        let response = err.clone().into_response();

        // 【結果検証】: ステータスコードが500であることを確認する
        // 【期待値確認】: 新規バリアントが既存INTERNAL_ERROR(500)と同じHTTPステータスに
        // マッピングされるが、ワイヤーコード文字列は専用のFILE_STORAGE_WRITE_FAILEDであることを保証
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR); // 【確認内容】: FileStorageWriteFailedが500 INTERNAL_SERVER_ERRORにマッピングされることを確認 🟡
        assert_eq!(err.error.code, "FILE_STORAGE_WRITE_FAILED"); // 【確認内容】: ワイヤーコードがFILE_STORAGE_WRITE_FAILEDであることを確認 🟡
    }

    /// TC-020-U01: ApiErrorCode::FileNotFoundが404・ワイヤーコードFILE_NOT_FOUNDにマッピングされる
    /// 【テスト目的】: TASK-0028新規追加のFileNotFoundバリアントが、HTTP 404・
    /// ワイヤーコードFILE_NOT_FOUNDに正しくマッピングされ、既存ItemNotFound("ITEM_NOT_FOUND")と
    /// 文字列が異なる専用コードであることを確認する
    /// 【テスト内容】: ApiError::new(ApiErrorCode::FileNotFound, ...)を構築し、
    /// into_response()のステータスとerror.code文字列を検証する
    /// 【期待される動作】: status==404 NOT_FOUND、error.code=="FILE_NOT_FOUND"
    /// 🟡 信頼性レベル: calibre-link-testcases.md TC-020-U01・calibre-link-requirements.md 2.2 L67-71より新規追加
    /// 【Red期待】: ApiErrorCode::FileNotFoundバリアント未追加のためコンパイルエラーとなる想定
    #[test]
    fn file_not_found_returns_404_with_expected_wire_code() {
        // 【テストデータ準備】: file_id不存在時のエラー応答生成を想定したApiError構築
        // 【初期条件設定】: 特になし
        let err = ApiError::new(ApiErrorCode::FileNotFound, "ファイルが見つかりません");

        // 【実際の処理実行】: into_response()でAxumレスポンスへ変換する
        // 【処理内容】: IntoResponse実装によるHTTPレスポンス構築
        let response = err.clone().into_response();

        // 【結果検証】: ステータスコードが404、ワイヤーコードがFILE_NOT_FOUNDであることを確認する
        // 【期待値確認】: 新規バリアントが500（デフォルト誤マッピング）に落ちないこと、既存ITEM_NOT_FOUNDとは
        // 文字列が異なる専用コードであることを保証する
        assert_eq!(response.status(), StatusCode::NOT_FOUND); // 【確認内容】: FileNotFoundが404 NOT_FOUNDにマッピングされることを確認 🟡
        assert_eq!(err.error.code, "FILE_NOT_FOUND"); // 【確認内容】: ワイヤーコードがFILE_NOT_FOUND（既存ITEM_NOT_FOUNDとは異なる文字列）であることを確認 🟡
    }

    /// TC-0025-E07-B: ApiErrorCode::ItemAlreadyImportedのワイヤーコード文字列がITEM_ALREADY_IMPORTEDになる
    /// 【テスト目的】: HTTPステータスだけでなくワイヤーコード文字列自体も要件通りであることを確認する
    /// 【テスト内容】: ApiError::newで構築したエラーのerror.code文字列を直接検証する
    /// 【期待される動作】: error.code=="ITEM_ALREADY_IMPORTED"
    /// 🔵 信頼性レベル: item-import-requirements.md 3.3「ITEM_ALREADY_IMPORTED新規variant追加」に直接対応
    #[test]
    fn item_already_imported_has_correct_wire_code() {
        // 【テストデータ準備】: 重複インポートエラーの構築
        let err = ApiError::new(ApiErrorCode::ItemAlreadyImported, "既にインポート済みです");

        // 【結果検証】: ワイヤーコード文字列が仕様通りであることを確認
        assert_eq!(err.error.code, "ITEM_ALREADY_IMPORTED"); // 【確認内容】: ワイヤーコードがITEM_ALREADY_IMPORTEDであることを確認 🔵
    }

    /// TC-017-E01-CODE: ApiErrorCode::SteamApiKeyInvalidが401・STEAM_API_KEY_INVALIDへマッピングされる
    /// 【テスト目的】: TASK-0031新規追加のSteamApiKeyInvalidバリアントが、HTTP 401・
    /// ワイヤーコードSTEAM_API_KEY_INVALIDに正しくマッピングされることを確認する
    /// 【テスト内容】: ApiError::new(ApiErrorCode::SteamApiKeyInvalid, ...)を構築し、
    /// into_response()のステータスとerror.code文字列を検証する
    /// 【期待される動作】: status==401 UNAUTHORIZED、error.code=="STEAM_API_KEY_INVALID"
    /// 🔵 信頼性レベル: TASK-0031.md「テストケース2（TC-017-E01）」・steam-import-requirements.md 3章より新規追加
    /// 【Red期待】: ApiErrorCode::SteamApiKeyInvalidバリアント未追加の場合はコンパイルエラーとなる想定
    #[test]
    fn steam_api_key_invalid_returns_401_with_expected_wire_code() {
        // 【テストデータ準備】: Steam APIキー無効時のエラー応答生成を想定したApiError構築
        // 【初期条件設定】: 特になし
        let err = ApiError::new(ApiErrorCode::SteamApiKeyInvalid, "Steam APIキーが無効です");

        // 【実際の処理実行】: into_response()でAxumレスポンスへ変換する
        // 【処理内容】: IntoResponse実装によるHTTPレスポンス構築
        let response = err.clone().into_response();

        // 【結果検証】: ステータスコードが401、ワイヤーコードがSTEAM_API_KEY_INVALIDであることを確認する
        // 【期待値確認】: 既存Unauthorized（"UNAUTHORIZED"）とは異なる専用コード文字列であることを保証
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED); // 【確認内容】: SteamApiKeyInvalidが401 UNAUTHORIZEDにマッピングされることを確認 🔵
        assert_eq!(err.error.code, "STEAM_API_KEY_INVALID"); // 【確認内容】: ワイヤーコードがSTEAM_API_KEY_INVALID（既存UNAUTHORIZEDとは異なる文字列）であることを確認 🔵
    }

    /// ApiErrorCode::UnsupportedBackupVersionが400・UNSUPPORTED_BACKUP_VERSIONへマッピングされる
    #[test]
    fn unsupported_backup_version_returns_400_with_expected_wire_code() {
        let err = ApiError::new(
            ApiErrorCode::UnsupportedBackupVersion,
            "未対応のバックアップバージョンです",
        );

        let response = err.clone().into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(err.error.code, "UNSUPPORTED_BACKUP_VERSION");
    }
}
