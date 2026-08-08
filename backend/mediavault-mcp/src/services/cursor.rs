//! Service層: `search_library` の keyset カーソルの不透明化
//!
//! TASK-0013: search_library ツールの実装

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// api の keyset カーソル値。`next_cursor` としてエンコードして返し、
/// 受け取ったカーソルはデコードして api のクエリへ戻す。
///
/// 🟡 Intent: タスクファイル「実装項目5」より。JSON → Base64（URL-safe, パディングなし）で
///    不透明化する。AI が中身を解釈して改変しないようにする狙い。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cursor {
    pub after_created_at: NaiveDateTime,
    pub after_id: Uuid,
}

/// カーソルのデコードに失敗したことを表すエラー。
///
/// 🟡 Intent: `MCP_INVALID_ARGUMENT` への変換は呼び出し側（tools層）の責務とし、
///    このモジュールは api 呼び出しに依存しない純粋な変換のみを担う。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidCursor;

impl Cursor {
    /// 🟡 Intent: 実装項目5「エンコード方式: JSON → Base64 など」より。
    pub fn encode(&self) -> String {
        let json = serde_json::to_vec(self).expect("Cursor は常にシリアライズ可能");
        URL_SAFE_NO_PAD.encode(json)
    }

    /// 🟡 Intent: 不正なカーソルは `MCP_INVALID_ARGUMENT` にする（実装項目5）。
    ///    api 側は不正カーソルを無視して先頭ページを返す仕様だが、MCP では明示的にエラーにする。
    pub fn decode(encoded: &str) -> Result<Self, InvalidCursor> {
        let bytes = URL_SAFE_NO_PAD.decode(encoded).map_err(|_| InvalidCursor)?;
        serde_json::from_slice(&bytes).map_err(|_| InvalidCursor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テストケース3: カーソルの往復
    #[test]
    fn encode_then_decode_round_trips() {
        let cursor = Cursor {
            after_created_at: NaiveDateTime::parse_from_str(
                "2026-07-01T12:00:00",
                "%Y-%m-%dT%H:%M:%S",
            )
            .unwrap(),
            after_id: Uuid::new_v4(),
        };

        let encoded = cursor.encode();
        let decoded = Cursor::decode(&encoded).unwrap();

        assert_eq!(decoded, cursor);
    }

    /// テストケース4: 不正なカーソル
    #[test]
    fn decode_rejects_invalid_base64() {
        let result = Cursor::decode("not-base64");
        assert_eq!(result, Err(InvalidCursor));
    }

    #[test]
    fn decode_rejects_valid_base64_with_wrong_shape() {
        let encoded = URL_SAFE_NO_PAD.encode(b"{\"foo\":1}");
        let result = Cursor::decode(&encoded);
        assert_eq!(result, Err(InvalidCursor));
    }
}
