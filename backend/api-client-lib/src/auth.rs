use std::time::Instant;

/// 認証戦略。
///
/// シークレット値は `Debug` 出力でマスクされる。
#[derive(Clone)]
pub enum AuthStrategy {
    /// 認証なし（Jikan / NDL / Open Library）
    None,
    /// API キー（Steam / TMDb のクエリパラメータ方式）
    ApiKey(String),
    /// Bearer トークン（AniList 認証済み / TMDb Bearer 方式）
    Bearer(String),
    /// Twitch OAuth2（IGDB 専用。クライアント資格情報フロー）
    TwitchOAuth2 {
        client_id: String,
        client_secret: String,
    },
}

impl std::fmt::Debug for AuthStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthStrategy::None => write!(f, "AuthStrategy::None"),
            AuthStrategy::ApiKey(_) => write!(f, "AuthStrategy::ApiKey([REDACTED])"),
            AuthStrategy::Bearer(_) => write!(f, "AuthStrategy::Bearer([REDACTED])"),
            AuthStrategy::TwitchOAuth2 { client_id, .. } => write!(
                f,
                "AuthStrategy::TwitchOAuth2 {{ client_id: {client_id:?}, client_secret: [REDACTED] }}"
            ),
        }
    }
}

/// IGDB 用 Twitch アクセストークン状態（ライブラリ内部型）。
///
/// シリアライズ禁止（シークレット漏洩防止）。
#[derive(Debug)]
pub(crate) struct TwitchTokenState {
    /// アクセストークン文字列
    pub access_token: String,
    /// トークン有効期限（`Instant` ベース）
    pub expires_at: Instant,
}

impl TwitchTokenState {
    /// トークンが有効かどうかを確認する。
    pub fn is_valid(&self) -> bool {
        self.expires_at > Instant::now()
    }
}
