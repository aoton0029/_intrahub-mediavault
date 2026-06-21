/// 生レスポンスデータ。
#[derive(Debug, Clone)]
pub enum RawData {
    /// REST / GraphQL / Apicalypse レスポンス（JSON 文字列）
    Json(String),
    /// NDL OpenSearch レスポンス（XML 文字列）
    Xml(String),
}

/// HTTP リクエスト結果メタデータ。
#[derive(Debug, Clone)]
pub struct RequestResult {
    /// HTTP ステータスコード
    pub status: u16,
    /// リクエスト URL
    pub url: String,
    /// レイテンシ（ミリ秒）
    pub latency_ms: u64,
}

/// API レスポンス統一型。
#[derive(Debug)]
pub struct ApiResponse<T> {
    /// HTTP メタデータ（status / url / latency）
    pub request: RequestResult,
    /// 生レスポンス（JSON または XML 文字列）
    pub raw: RawData,
    /// 変換済みドメインモデル
    pub model: T,
}
