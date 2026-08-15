//! MediaVault-api のレスポンス封筒（`ApiEnvelope<T>`）
//!
//! TASK-0008: ApiClient層の実装

use chrono::NaiveDateTime;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use uuid::Uuid;

/// 一覧系レスポンスのページネーション情報（keyset ページネーション）。
///
/// 🔵 Intent: `items.md` GET /items の `pagination` レスポンス形
///    `{ limit, has_more, next_after_created_at, next_after_id, total? }` に準拠する。
///    TASK-0013 実装時に offset 方式の誤った定義（total/limit/offset）から修正した。
#[derive(Debug, Clone, Deserialize)]
pub struct ApiPagination {
    pub limit: u32,
    pub has_more: bool,
    pub next_after_created_at: Option<NaiveDateTime>,
    pub next_after_id: Option<Uuid>,
    /// `include_total=true` を指定したときのみ返る 🔵 `items.md` より
    #[serde(default)]
    pub total: Option<u64>,
}

/// MediaVault-api のエラーボディ（`code` / `message`）
///
/// 🔵 Intent: REQ-146 により code/message を失わず上位へ透過する必要があるため、
///    それぞれ独立したフィールドとして保持する。
#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub candidates: Option<serde_json::Value>,
}

/// MediaVault-api が返す成功/失敗レスポンスの封筒。
///
/// 🟡 Intent: `#[serde(untagged)]` はバリアントを順に試すため、`T` が
///    `serde_json::Value` のような緩い型だとエラー形式のJSONが `Ok` として
///    誤って通ってしまう（タスクファイルの落とし穴の指摘）。そのため
///    `success` フィールドを先に見るカスタム `Deserialize` を実装し、
///    型に依存せず正しいバリアントへ振り分ける。
#[derive(Debug, Clone)]
pub enum ApiEnvelope<T> {
    Ok {
        data: T,
        pagination: Option<ApiPagination>,
    },
    Err {
        error: ApiErrorBody,
    },
}

impl<'de, T> Deserialize<'de> for ApiEnvelope<T>
where
    T: DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Probe {
            success: bool,
        }

        let value = serde_json::Value::deserialize(deserializer)?;
        let probe: Probe =
            serde_json::from_value(value.clone()).map_err(serde::de::Error::custom)?;

        if probe.success {
            #[derive(Deserialize)]
            struct OkShape<T> {
                data: T,
                #[serde(default)]
                pagination: Option<ApiPagination>,
            }
            let shape: OkShape<T> =
                serde_json::from_value(value).map_err(serde::de::Error::custom)?;
            Ok(ApiEnvelope::Ok {
                data: shape.data,
                pagination: shape.pagination,
            })
        } else {
            #[derive(Deserialize)]
            struct ErrShape {
                error: ApiErrorBody,
            }
            let shape: ErrShape =
                serde_json::from_value(value).map_err(serde::de::Error::custom)?;
            Ok(ApiEnvelope::Err { error: shape.error })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Deserialize)]
    struct Item {
        id: String,
        title: String,
    }

    /// テストケース1: ApiOk のデシリアライズ
    #[test]
    fn deserializes_ok_envelope() {
        let json = r#"{"success":true,"data":{"id":"1","title":"作品A"}}"#;
        let envelope: ApiEnvelope<Item> = serde_json::from_str(json).unwrap();

        match envelope {
            ApiEnvelope::Ok { data, pagination } => {
                assert_eq!(data.id, "1");
                assert_eq!(data.title, "作品A");
                assert!(pagination.is_none());
            }
            ApiEnvelope::Err { .. } => panic!("expected Ok variant"),
        }
    }

    /// テストケース2: ApiError のデシリアライズ
    #[test]
    fn deserializes_err_envelope() {
        let json =
            r#"{"success":false,"error":{"code":"ITEM_NOT_FOUND","message":"見つかりません"}}"#;
        let envelope: ApiEnvelope<Item> = serde_json::from_str(json).unwrap();

        match envelope {
            ApiEnvelope::Err { error } => {
                assert_eq!(error.code, "ITEM_NOT_FOUND");
                assert_eq!(error.message, "見つかりません");
            }
            ApiEnvelope::Ok { .. } => panic!("expected Err variant"),
        }
    }

    /// テストケース3: pagination 付きレスポンス（keyset 形式）
    #[test]
    fn deserializes_ok_envelope_with_pagination() {
        let json = r#"{"success":true,"data":[{"id":"1","title":"作品A"}],"pagination":{"limit":20,"has_more":true,"next_after_created_at":"2026-07-01T12:00:00","next_after_id":"b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001","total":10}}"#;
        let envelope: ApiEnvelope<Vec<Item>> = serde_json::from_str(json).unwrap();

        match envelope {
            ApiEnvelope::Ok { data, pagination } => {
                assert_eq!(data.len(), 1);
                let pagination = pagination.unwrap();
                assert_eq!(pagination.total, Some(10));
                assert_eq!(pagination.limit, 20);
                assert!(pagination.has_more);
                assert!(pagination.next_after_id.is_some());
            }
            ApiEnvelope::Err { .. } => panic!("expected Ok variant"),
        }
    }

    /// テストケース4: untagged の誤判定がない（T = serde_json::Value でもエラーは Err と判定される）
    #[test]
    fn error_shape_is_not_misdetected_as_ok_for_loose_type() {
        let json =
            r#"{"success":false,"error":{"code":"ITEM_NOT_FOUND","message":"見つかりません"}}"#;
        let envelope: ApiEnvelope<serde_json::Value> = serde_json::from_str(json).unwrap();

        match envelope {
            ApiEnvelope::Err { error } => {
                assert_eq!(error.code, "ITEM_NOT_FOUND");
            }
            ApiEnvelope::Ok { .. } => panic!("expected Err variant, got Ok"),
        }
    }
}
