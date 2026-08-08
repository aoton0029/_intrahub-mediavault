//! `ApiClient` のエラー分類
//!
//! TASK-0008: ApiClient層の実装

/// MediaVault-api 呼び出し時のエラー分類。
///
/// 🔵 Intent: タスクファイルのエラー分類表に基づく4分類。
///    `Connection` / `Api` は上位で `retriable` の判定に使われる
///    （`Connection: true`、それ以外は `false`）。
#[derive(Debug, thiserror::Error)]
pub enum ApiClientError {
    /// 接続失敗・タイムアウト・DNS失敗
    #[error("MediaVault-api への接続に失敗しました: {0}")]
    Connection(String),

    /// 内部API呼び出しで 401
    ///
    /// 🔵 Intent: REQ-145「APIキーをログに出力してはならない」のため、
    ///    キー値やヘッダー内容を一切含めない固定メッセージにする。
    #[error("内部APIの認証に失敗しました")]
    Auth,

    /// api の業務エラー。code/message/status をそのまま透過する（REQ-146）
    #[error("MediaVault-api がエラーを返しました: {code} {message}")]
    Api {
        code: String,
        message: String,
        status: u16,
    },

    /// デシリアライズ失敗
    #[error("レスポンスの解析に失敗しました: {0}")]
    Decode(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Auth バリアントの Display にキー値等が含まれない
    #[test]
    fn auth_error_display_has_no_secret() {
        let err = ApiClientError::Auth;
        assert_eq!(err.to_string(), "内部APIの認証に失敗しました");
    }

    /// Api バリアントは code/message/status を保持する
    #[test]
    fn api_error_keeps_code_message_status() {
        let err = ApiClientError::Api {
            code: "TAG_NOT_FOUND".to_string(),
            message: "タグが見つかりません".to_string(),
            status: 404,
        };

        match err {
            ApiClientError::Api {
                code,
                message,
                status,
            } => {
                assert_eq!(code, "TAG_NOT_FOUND");
                assert_eq!(message, "タグが見つかりません");
                assert_eq!(status, 404);
            }
            _ => panic!("expected Api variant"),
        }
    }
}
