use crate::{error::ApiError, response::ApiResponse};

/// 全 API クライアントが実装する統一インターフェース。
///
/// 各クライアントは `Request` 列挙型と `Model` 列挙型を associated type として定義し、
/// `execute()` を通じて統一的に呼び出せる。
/// 個別のエンドポイントには型付きメソッド（例: `search_media`）も提供される。
pub trait ApiClient {
    type Request;
    type Model;

    /// リクエストを実行し、統一レスポンス型を返す。
    fn execute(
        &self,
        request: Self::Request,
    ) -> impl std::future::Future<Output = Result<ApiResponse<Self::Model>, ApiError>> + Send;
}
