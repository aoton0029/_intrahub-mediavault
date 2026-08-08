//! 環境変数からの `Config` 構築と起動時検証
//!
//! TASK-0006: Config と起動時検証

use std::collections::HashMap;
use std::fmt;

/// `Debug` / `Display` でマスクされる秘密文字列。
///
/// ログや panic メッセージへの値漏洩を型レベルで防ぐ。値の取り出しは
/// [`SecretString::expose`] のみで、名前で意図を明示する。
#[derive(Clone)]
#[allow(
    dead_code,
    reason = "expose() は後続タスク（認証・APIクライアント）で使用する"
)]
pub struct SecretString(String);

impl SecretString {
    #[allow(dead_code, reason = "後続タスク（認証・APIクライアント）で使用する")]
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl From<String> for SecretString {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretString(***)")
    }
}

/// 既定値
const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8081";
const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 3;
const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 15;
const DEFAULT_EXTERNAL_TIMEOUT_SECS: u64 = 30;

/// トークン長の下限（下回る場合は警告のみ、起動は失敗させない）
const MIN_TOKEN_LEN: usize = 32;

#[derive(Debug, Clone)]
#[allow(
    dead_code,
    reason = "各フィールドは後続タスク（認証・APIクライアント）で使用する"
)]
pub struct Config {
    /// 待受アドレス。既定 `0.0.0.0:8081`
    pub bind_addr: String,
    /// Bearer 認証トークン。未設定・空文字・空白のみは起動失敗（REQ-122）
    pub auth_token: SecretString,
    /// api のベースURL
    pub api_base_url: String,
    /// 内部API用キー。MVPでは未使用ツールのみのため必須にしない
    pub internal_api_key: SecretString,
    /// 接続タイムアウト（秒）。既定 3
    pub connect_timeout_secs: u64,
    /// 全体タイムアウト（秒）。既定 15
    pub request_timeout_secs: u64,
    /// 外部カタログ検索用タイムアウト（秒）。既定 30
    pub external_timeout_secs: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("{0} が未設定です")]
    Missing(&'static str),
    #[error("{0} の値が不正です")]
    Invalid(&'static str),
}

impl Config {
    /// 環境変数から `Config` を構築する。
    pub fn from_env() -> Result<Config, ConfigError> {
        let env: HashMap<String, String> = std::env::vars().collect();
        Self::from_map(&env)
    }

    /// 値の集合から `Config` を構築する。環境変数を直接読まないことで、
    /// テストが `std::env::set_var` による並列干渉を受けないようにする。
    fn from_map(env: &HashMap<String, String>) -> Result<Config, ConfigError> {
        let auth_token = Self::require_non_blank(env, "MCP_AUTH_TOKEN")?;
        if auth_token.trim().len() < MIN_TOKEN_LEN {
            tracing::warn!(
                "MCP_AUTH_TOKEN が {} 文字未満です。総当たり耐性のため十分な長さを推奨します",
                MIN_TOKEN_LEN
            );
        }

        let api_base_url = Self::require_non_blank(env, "MEDIAVAULT_API_BASE_URL")?;
        url::Url::parse(&api_base_url)
            .map_err(|_| ConfigError::Invalid("MEDIAVAULT_API_BASE_URL"))?;

        let bind_addr = env
            .get("MCP_BIND_ADDR")
            .map(|s| s.as_str())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(DEFAULT_BIND_ADDR)
            .to_string();
        bind_addr
            .parse::<std::net::SocketAddr>()
            .map_err(|_| ConfigError::Invalid("MCP_BIND_ADDR"))?;

        let internal_api_key = env
            .get("INTERNAL_API_KEY")
            .cloned()
            .unwrap_or_default()
            .into();

        let connect_timeout_secs = Self::parse_timeout(
            env,
            "MCP_CONNECT_TIMEOUT_SECS",
            DEFAULT_CONNECT_TIMEOUT_SECS,
        )?;
        let request_timeout_secs = Self::parse_timeout(
            env,
            "MCP_REQUEST_TIMEOUT_SECS",
            DEFAULT_REQUEST_TIMEOUT_SECS,
        )?;
        let external_timeout_secs = Self::parse_timeout(
            env,
            "MCP_EXTERNAL_TIMEOUT_SECS",
            DEFAULT_EXTERNAL_TIMEOUT_SECS,
        )?;

        Ok(Config {
            bind_addr,
            auth_token: auth_token.into(),
            api_base_url,
            internal_api_key,
            connect_timeout_secs,
            request_timeout_secs,
            external_timeout_secs,
        })
    }

    fn require_non_blank(
        env: &HashMap<String, String>,
        key: &'static str,
    ) -> Result<String, ConfigError> {
        let value = env.get(key).ok_or(ConfigError::Missing(key))?;
        if value.trim().is_empty() {
            return Err(ConfigError::Missing(key));
        }
        Ok(value.clone())
    }

    /// 🟡 Intent: 他モジュール・他クレート（統合テスト）から `Config` を構築するための
    ///    ヘルパー。`env::set_var` によるテスト間の並列干渉を避けるため values map 経由にする。
    ///    `lib.rs` 分離（TASK-0008）により `main.rs`/`tests/` からも参照するため `pub` にする。
    pub fn from_map_for_test(env: &HashMap<String, String>) -> Config {
        Self::from_map(env).expect("config should build in tests")
    }

    fn parse_timeout(
        env: &HashMap<String, String>,
        key: &'static str,
        default: u64,
    ) -> Result<u64, ConfigError> {
        match env.get(key) {
            None => Ok(default),
            Some(raw) if raw.trim().is_empty() => Ok(default),
            Some(raw) => {
                let value: u64 = raw.trim().parse().map_err(|_| ConfigError::Invalid(key))?;
                if value == 0 {
                    return Err(ConfigError::Invalid(key));
                }
                Ok(value)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_env() -> HashMap<String, String> {
        let mut env = HashMap::new();
        env.insert("MCP_AUTH_TOKEN".to_string(), "a".repeat(40));
        env.insert(
            "MEDIAVAULT_API_BASE_URL".to_string(),
            "http://mediavault-api:8080".to_string(),
        );
        env
    }

    /// テストケース1: 全項目指定での構築
    #[test]
    fn builds_config_with_all_fields_set() {
        let mut env = base_env();
        env.insert("INTERNAL_API_KEY".to_string(), "internal-key".to_string());
        env.insert("MCP_BIND_ADDR".to_string(), "127.0.0.1:9000".to_string());
        env.insert("MCP_CONNECT_TIMEOUT_SECS".to_string(), "5".to_string());
        env.insert("MCP_REQUEST_TIMEOUT_SECS".to_string(), "20".to_string());
        env.insert("MCP_EXTERNAL_TIMEOUT_SECS".to_string(), "40".to_string());

        let config = Config::from_map(&env).expect("config should build");

        assert_eq!(config.auth_token.expose(), "a".repeat(40));
        assert_eq!(config.api_base_url, "http://mediavault-api:8080");
        assert_eq!(config.internal_api_key.expose(), "internal-key");
        assert_eq!(config.bind_addr, "127.0.0.1:9000");
        assert_eq!(config.connect_timeout_secs, 5);
        assert_eq!(config.request_timeout_secs, 20);
        assert_eq!(config.external_timeout_secs, 40);
    }

    /// テストケース2: MCP_AUTH_TOKEN 未設定
    #[test]
    fn missing_auth_token_fails() {
        let mut env = base_env();
        env.remove("MCP_AUTH_TOKEN");

        let err = Config::from_map(&env).unwrap_err();
        assert!(matches!(err, ConfigError::Missing("MCP_AUTH_TOKEN")));
        assert!(err.to_string().contains("MCP_AUTH_TOKEN"));
    }

    /// テストケース3: MCP_AUTH_TOKEN が空文字・空白
    #[test]
    fn blank_auth_token_fails() {
        for blank in ["", "   "] {
            let mut env = base_env();
            env.insert("MCP_AUTH_TOKEN".to_string(), blank.to_string());

            let err = Config::from_map(&env).unwrap_err();
            assert!(matches!(err, ConfigError::Missing("MCP_AUTH_TOKEN")));
        }
    }

    /// テストケース4: 既定値の適用
    #[test]
    fn defaults_apply_when_optional_fields_absent() {
        let env = base_env();

        let config = Config::from_map(&env).expect("config should build");

        assert_eq!(config.bind_addr, DEFAULT_BIND_ADDR);
        assert_eq!(config.connect_timeout_secs, DEFAULT_CONNECT_TIMEOUT_SECS);
        assert_eq!(config.request_timeout_secs, DEFAULT_REQUEST_TIMEOUT_SECS);
        assert_eq!(config.external_timeout_secs, DEFAULT_EXTERNAL_TIMEOUT_SECS);
        assert_eq!(config.internal_api_key.expose(), "");
    }

    /// テストケース5: 不正な API ベースURL
    #[test]
    fn invalid_api_base_url_fails() {
        let mut env = base_env();
        env.insert(
            "MEDIAVAULT_API_BASE_URL".to_string(),
            "not a url".to_string(),
        );

        let err = Config::from_map(&env).unwrap_err();
        assert!(matches!(
            err,
            ConfigError::Invalid("MEDIAVAULT_API_BASE_URL")
        ));
    }

    /// テストケース6: SecretString のマスク
    #[test]
    fn secret_string_is_masked_in_debug_output() {
        let secret: SecretString = "super-secret-value".to_string().into();
        let debug_output = format!("{:?}", secret);
        assert!(!debug_output.contains("super-secret-value"));

        let config = Config::from_map(&base_env()).expect("config should build");
        let config_debug = format!("{:?}", config);
        assert!(!config_debug.contains(&"a".repeat(40)));
    }

    /// テストケース7: 不正なタイムアウト値
    #[test]
    fn invalid_timeout_value_fails() {
        for value in ["0", "abc"] {
            let mut env = base_env();
            env.insert("MCP_REQUEST_TIMEOUT_SECS".to_string(), value.to_string());

            let err = Config::from_map(&env).unwrap_err();
            assert!(matches!(
                err,
                ConfigError::Invalid("MCP_REQUEST_TIMEOUT_SECS")
            ));
        }
    }

    /// MCP_BIND_ADDR が不正な場合は起動失敗する
    #[test]
    fn invalid_bind_addr_fails() {
        let mut env = base_env();
        env.insert("MCP_BIND_ADDR".to_string(), "not-an-addr".to_string());

        let err = Config::from_map(&env).unwrap_err();
        assert!(matches!(err, ConfigError::Invalid("MCP_BIND_ADDR")));
    }
}
