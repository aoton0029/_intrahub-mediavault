use std::time::Duration;

/// 全 API クライアントが返す統一エラー型。
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// HTTP エラーレスポンス（4xx / 5xx）
    #[error("HTTP error: status={status}")]
    Http { status: u16, body: String },

    /// 認証エラー（Twitch トークン取得失敗など）
    #[error("Auth error: {0}")]
    Auth(String),

    /// レート制限超過
    #[error("Rate limit exceeded, retry after: {retry_after:?}")]
    RateLimit { retry_after: Option<Duration> },

    /// JSON / XML パースエラー
    #[error("Parse error: {0}")]
    Parse(String),

    /// タイムアウト（3 秒超過）
    #[error("Request timed out")]
    Timeout,

    /// ネットワーク障害
    #[error("Network error: {0}")]
    Network(String),
}
