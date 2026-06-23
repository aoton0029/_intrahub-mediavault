//! 統一APIレスポンス型・共通エラー型
//!
//! TASK-0005: 共通エラー型・統一APIレスポンス実装

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;

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

/// ページネーション情報
///
/// 【機能概要】: 一覧取得APIで返す page/limit/total を保持する
/// 【実装方針】: 要件定義書2.2・テストケース定義書 確定1 に従い、page/limitはu32、totalはCOUNT(*)結果を
/// そのまま保持できるi64とする
/// 【テスト対応】: TC-0010-N09（PaginatedOkシリアライズ）を通すための実装
/// 🟡 信頼性レベル: テストケース定義書 確定1（PaginatedOk型の新規定義）に対応
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Pagination {
    pub page: u32,
    pub limit: u32,
    pub total: i64,
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
                page: 1,
                limit: 20,
                total: 100,
            },
        );

        // 【実際の処理実行】: serde_json::to_value でシリアライズする
        // 【処理内容】: PaginatedOk<T> の Serialize 実装による JSON 変換
        let json = serde_json::to_value(&body).unwrap();

        // 【結果検証】: トップレベルキーとpagination構造が要件通りであることを確認
        // 【期待値確認】: 要件定義書 2.2 のレスポンス例 { success, data, pagination: {page, limit, total} } に一致するか
        assert_eq!(
            json,
            serde_json::json!({
                "success": true,
                "data": [{"id": 1}],
                "pagination": {"page": 1, "limit": 20, "total": 100}
            })
        ); // 【確認内容】: success/data/paginationの3キー構成、pagination内のpage/limit/totalキー名が要件通りであることを確認 🟡
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
                page: 1,
                limit: 20,
                total: 0,
            },
        );

        // 【実際の処理実行】: into_response()でAxumレスポンスへ変換する
        // 【処理内容】: IntoResponse実装によるHTTPレスポンス構築
        let response = body.into_response();

        // 【結果検証】: ステータスコードが200であることを確認
        // 【期待値確認】: 空配列でも200を返す既存ApiOk規約の継続であることの確認
        assert_eq!(response.status(), StatusCode::OK); // 【確認内容】: 空データでも成功時ステータスが200で固定されていることを確認 🔵
    }
}
