//! POST /items/import 用リクエストDTO・バリデーション
//!
//! TASK-0025: POST /items/import（外部検索結果からのアイテムインポート）実装
//!
//! 【信頼性レベル】: 🔵 item-import-requirements.md 第2章・item-import-testcases.md TC-0025-N01より

use chrono::NaiveDate;
use serde::Deserialize;

use crate::models::item::{validate_title, CreateItemRequest};
use crate::models::response::{ApiError, ApiErrorCode};

/// `POST /items/import` リクエストDTO
///
/// 【機能概要】: 外部API検索結果（GET /items/search）から選択した1件をインポートするためのDTO。
/// `external_id`はAPI起源アイテムの必須項目（DB CHECK制約 chk_items_source_external_id対象）。
/// 🔵 信頼性レベル: item-import-requirements.md 2.1 入力仕様表に直接対応
#[derive(Debug, Clone, Deserialize)]
pub struct ImportItemRequest {
    pub media_type: crate::models::item::MediaType,
    /// 外部API上の一意ID。空文字・欠落は400 VALIDATION_ERROR対象（必須・非Option）
    pub external_id: String,
    pub title: String,
    pub original_title: Option<String>,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    pub release_date: Option<NaiveDate>,
    pub homepage_url: Option<String>,
    /// メディア別詳細テーブル用データ。現状はitem_idのみINSERTするため保持のみ。
    /// 🟡 信頼性レベル: 要件2.1「Option化を推奨」に基づく
    #[serde(default)]
    pub details: Option<serde_json::Value>,
}

/// `ImportItemRequest`を`create_item_with_source`が受け取る`CreateItemRequest`へ変換する。
///
/// 【機能概要】: インポート専用DTOと手動作成DTOの差分（`external_id`の有無、`rating`/`is_favorite`の
/// 入力対象外）を1箇所に集約し、`item_repository::import_item`側のフィールド列挙を不要にする。
/// 【設計方針】: `external_id`は`create_item_with_source`側の専用引数として渡すためここでは含めない。
/// `rating`・`is_favorite`はインポート時には指定対象外のため、常に`None`（DB側デフォルトfalse）とする。
/// 【保守性】: `CreateItemRequest`へフィールドが追加された場合、ここでコンパイルエラーとして
/// 検知できるよう個々のフィールドを明示的に列挙する（`..Default::default()`等は使わない）。
/// 🟡 信頼性レベル: item-import-requirements.md 2.3データフロー・既存`import_item`実装からの妥当な推測
impl From<ImportItemRequest> for CreateItemRequest {
    fn from(request: ImportItemRequest) -> Self {
        CreateItemRequest {
            media_type: request.media_type,
            title: request.title,
            original_title: request.original_title,
            description: request.description,
            cover_image_url: request.cover_image_url,
            release_date: request.release_date,
            homepage_url: request.homepage_url,
            // 【インポート時は対象外】: ratingはユーザーの評価入力、is_favoriteはユーザー操作のため
            // 外部APIインポート時点では設定しない（Noneでis_favoriteはDB側デフォルトfalseになる） 🟡
            rating: None,
            is_favorite: None,
            details: request.details,
            // 【TASK-0030拡張】: 外部APIインポート経路ではconsumed_date入力対象外のためNone固定 🟡
            consumed_date: None,
        }
    }
}

/// `external_id`が空文字・空白のみでないことを検証する。
///
/// 【機能概要】: ImportItemRequest.external_idの必須バリデーション。
/// trim().is_empty()基準で既存`validate_title`と判定基準を揃える。
/// 【実装方針】: `validate_title`（models/item.rs）と同様にtrim().is_empty()で空文字・空白のみを
/// 400 VALIDATION_ERRORとして拒否する。
/// 🔵 信頼性レベル: item-import-requirements.md 2.1「空文字・欠落は400」、
/// item-import-testcases.md TC-0025-E01〜E03より
pub fn validate_external_id(external_id: &str) -> Result<(), ApiError> {
    // 【空文字・空白のみ判定】: 既存validate_title（item.rs）と同基準のtrim().is_empty()で判定する 🔵
    if external_id.trim().is_empty() {
        // 【エラー処理】: external_idは必須項目のため、空文字・空白のみは400 VALIDATION_ERRORとする 🔵
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "external_idは空にできません",
        ));
    }
    Ok(())
}

/// JSON値を`ImportItemRequest`へデシリアライズし、バリデーションを行う。
///
/// 【機能概要】: `parse_create_item_request`相当の役割。media_type/external_id/titleの
/// デシリアライズ失敗・空文字を400 VALIDATION_ERRORへ変換する。
/// 【実装方針】: `deserialize_request` → `validate_external_id` → `validate_title`の順に
/// 検証する。デシリアライズ失敗は`?`演算子経由でVALIDATION_ERRORへ変換される。
/// 🔵 信頼性レベル: item-import-requirements.md 2.3データフロー、
/// item-import-testcases.md TC-0025-N01・E01〜E05より
pub fn parse_import_item_request(value: serde_json::Value) -> Result<ImportItemRequest, ApiError> {
    // 【デシリアライズ】: media_type/external_id/title等をImportItemRequestへ変換する。
    // デシリアライズ失敗（media_type不正値・external_id欠落等）はVALIDATION_ERRORへ変換される 🔵
    let request: ImportItemRequest = crate::models::item::deserialize_request(value)?;

    // 【external_idバリデーション】: 空文字・空白のみを400 VALIDATION_ERRORとして拒否する 🔵
    validate_external_id(&request.external_id)?;
    // 【titleバリデーション】: 既存CreateItemRequest用validate_titleを再利用し、
    // 空文字・空白のみのtitleを拒否する 🟡
    validate_title(&request.title)?;

    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::item::MediaType;
    use axum::http::StatusCode;

    /// TC-0025-N01: ImportItemRequestが必須項目（media_type/external_id/title）のみで正常にデシリアライズされる
    /// 【テスト目的】: 新規DTO ImportItemRequestが、最小構成のJSONから正しくパースされることを確認する
    /// 【テスト内容】: media_type/external_id/titleのみを含むJSONをparse_import_item_requestへ渡す
    /// 【期待される動作】: Ok(ImportItemRequest)が返り、各フィールドが入力値と一致する
    /// 🔵 信頼性レベル: 要件2.1入力仕様表、既存create_item_request_deserializes_successfullyのパリティより
    #[test]
    fn import_item_request_deserializes_minimal_fields() {
        // 【テストデータ準備】: 必須3フィールドのみの最小構成リクエスト
        // 【初期条件設定】: 任意項目（original_title等）は省略する
        let value = serde_json::json!({
            "media_type": "anime",
            "external_id": "12345",
            "title": "鬼滅の刃"
        });

        // 【実際の処理実行】: まだバリデーションが未完成のparse_import_item_requestを呼び出す
        // 【処理内容】: JSON→ImportItemRequestへのデシリアライズ・検証
        let result = parse_import_item_request(value);

        // 【結果検証】: デシリアライズが成功し、各フィールドが入力値と一致することを確認
        let request = result.unwrap();
        assert_eq!(request.media_type, MediaType::Anime); // 【確認内容】: media_typeがAnimeへ変換されることを確認 🔵
        assert_eq!(request.external_id, "12345"); // 【確認内容】: external_idがStringとして保持されることを確認 🔵
        assert_eq!(request.title, "鬼滅の刃"); // 【確認内容】: titleが正しく保持されることを確認 🔵
    }

    /// TC-0025-E01: external_id欠落 → 400 VALIDATION_ERROR
    /// 【テスト目的】: 必須external_idのバリデーションを確認する
    /// 【テスト内容】: external_idキーを含まないJSONをparse_import_item_requestへ渡す
    /// 【期待される動作】: Err(ApiError)が返り、error.code=="VALIDATION_ERROR"、status==400
    /// 🔵 信頼性レベル: 要件2.2エラー表・4.2、TASK-0025.mdテストケース2に直接対応
    /// 【補足】: external_idはString型（非Option）のためserdeデシリアライズ自体が失敗し
    /// VALIDATION_ERRORになる（validate_external_idの呼び出し有無に関わらず成立する経路）
    #[test]
    fn import_item_request_missing_external_id_returns_validation_error() {
        // 【テストデータ準備】: external_idキーを含まないリクエストボディ
        // 【初期条件設定】: フロントエンドのバグ・外部検索結果のexternal_id未取得時を再現
        let value = serde_json::json!({
            "media_type": "anime",
            "title": "鬼滅の刃"
        });

        // 【実際の処理実行】: parse_import_item_requestを呼び出す
        let err = parse_import_item_request(value).unwrap_err();

        // 【結果検証】: VALIDATION_ERROR・400であることを確認
        assert_eq!(err.error.code, "VALIDATION_ERROR"); // 【確認内容】: external_id欠落でVALIDATION_ERRORが返ることを確認 🔵
        assert_eq!(err.status, StatusCode::BAD_REQUEST); // 【確認内容】: HTTPステータスが400であることを確認 🔵
    }

    /// TC-0025-E02: external_id空文字 → 400 VALIDATION_ERROR
    /// 【テスト目的】: 空文字のexternal_idを欠落と同等に拒否することを確認する
    /// 【テスト内容】: external_id=""のJSONをparse_import_item_requestへ渡す
    /// 【期待される動作】: Err(ApiError)が返り、error.code=="VALIDATION_ERROR"
    /// 🔵 信頼性レベル: 要件2.1制約欄「空文字・欠落は400」、既存validate_titleのtrim規約より
    #[test]
    fn import_item_request_empty_external_id_returns_validation_error() {
        // 【テストデータ準備】: external_idが空文字のリクエストボディ
        // 【初期条件設定】: 外部APIが空のIDを返した場合を再現
        let value = serde_json::json!({
            "media_type": "anime",
            "external_id": "",
            "title": "鬼滅の刃"
        });

        // 【実際の処理実行】: parse_import_item_requestを呼び出す
        let err = parse_import_item_request(value).unwrap_err();

        // 【結果検証】: VALIDATION_ERRORであることを確認
        assert_eq!(err.error.code, "VALIDATION_ERROR"); // 【確認内容】: 空文字external_idがVALIDATION_ERRORになることを確認 🔵
        assert_eq!(err.status, StatusCode::BAD_REQUEST); // 【確認内容】: HTTPステータスが400であることを確認 🔵
    }

    /// TC-0025-E03: external_id空白のみ（"   "） → 400 VALIDATION_ERROR
    /// 【テスト目的】: 空白のみのexternal_idを拒否し、titleバリデーションと基準を揃えることを確認する
    /// 【テスト内容】: external_id="   "のJSONをparse_import_item_requestへ渡す
    /// 【期待される動作】: Err(ApiError)が返り、error.code=="VALIDATION_ERROR"
    /// 🟡 信頼性レベル: 既存validate_titleのtrim().is_empty()方式からの妥当な推測
    #[test]
    fn import_item_request_blank_external_id_returns_validation_error() {
        // 【テストデータ準備】: 半角スペースのみのexternal_id
        // 【初期条件設定】: 入力フォームの空白混入を再現
        let value = serde_json::json!({
            "media_type": "anime",
            "external_id": "   ",
            "title": "鬼滅の刃"
        });

        // 【実際の処理実行】: parse_import_item_requestを呼び出す
        let err = parse_import_item_request(value).unwrap_err();

        // 【結果検証】: VALIDATION_ERRORであることを確認
        assert_eq!(err.error.code, "VALIDATION_ERROR"); // 【確認内容】: 空白のみのexternal_idがVALIDATION_ERRORになることを確認 🟡
        assert_eq!(err.status, StatusCode::BAD_REQUEST); // 【確認内容】: HTTPステータスが400であることを確認 🟡
    }

    /// TC-0025-E04: media_type不正値 → 400 VALIDATION_ERROR
    /// 【テスト目的】: 不正media_typeのデシリアライズ失敗→400変換を確認する
    /// 【テスト内容】: media_type="invalid_type"のJSONをparse_import_item_requestへ渡す
    /// 【期待される動作】: Err(ApiError)が返り、error.code=="VALIDATION_ERROR"
    /// 🟡 信頼性レベル: 既存invalid_media_type_returns_validation_error（item.rs）とのパリティ
    #[test]
    fn import_item_request_invalid_media_type_returns_validation_error() {
        // 【テストデータ準備】: enum 8種に存在しないmedia_type値
        let value = serde_json::json!({
            "media_type": "invalid_type",
            "external_id": "12345",
            "title": "X"
        });

        // 【実際の処理実行】: parse_import_item_requestを呼び出す
        let err = parse_import_item_request(value).unwrap_err();

        // 【結果検証】: VALIDATION_ERRORであることを確認
        assert_eq!(err.error.code, "VALIDATION_ERROR"); // 【確認内容】: 不正media_typeでVALIDATION_ERRORが返ることを確認 🟡
        assert_eq!(err.status, StatusCode::BAD_REQUEST); // 【確認内容】: HTTPステータスが400であることを確認 🟡
    }

    /// TC-0025-E05: title空文字 → 400 VALIDATION_ERROR
    /// 【テスト目的】: titleバリデーションのパリティを確認する
    /// 【テスト内容】: title=""のJSONをparse_import_item_requestへ渡す
    /// 【期待される動作】: Err(ApiError)が返り、error.code=="VALIDATION_ERROR"
    /// 🟡 信頼性レベル: 既存empty_title_returns_validation_error（item.rs）との一貫性
    #[test]
    fn import_item_request_empty_title_returns_validation_error() {
        // 【テストデータ準備】: titleが空文字のリクエストボディ
        let value = serde_json::json!({
            "media_type": "anime",
            "external_id": "12345",
            "title": ""
        });

        // 【実際の処理実行】: parse_import_item_requestを呼び出す
        let err = parse_import_item_request(value).unwrap_err();

        // 【結果検証】: VALIDATION_ERRORであることを確認
        assert_eq!(err.error.code, "VALIDATION_ERROR"); // 【確認内容】: title空文字でVALIDATION_ERRORが返ることを確認 🟡
        assert_eq!(err.status, StatusCode::BAD_REQUEST); // 【確認内容】: HTTPステータスが400であることを確認 🟡
    }

    /// TC-0025-B02: detailsフィールド省略時に#[serde(default)]でNone扱いとなりデシリアライズが成功する
    /// 【テスト目的】: detailsのOption化・default挙動を確認する
    /// 【テスト内容】: detailsキーを含まないJSONをparse_import_item_requestへ渡す
    /// 【期待される動作】: Ok(request)が返り、request.details == None
    /// 🟡 信頼性レベル: 要件2.1 L49-51（detailsのOption化推奨・範囲外明記）、
    /// 既存CreateItemRequest.detailsより
    #[test]
    fn import_item_request_omitted_details_defaults_to_none() {
        // 【テストデータ準備】: detailsキーを含まない最小構成（必須3項目のみ）
        // 【初期条件設定】: 詳細データを送らないシンプルなインポートを再現
        let value = serde_json::json!({
            "media_type": "anime",
            "external_id": "1",
            "title": "A"
        });

        // 【実際の処理実行】: parse_import_item_requestを呼び出す
        let result = parse_import_item_request(value);

        // 【結果検証】: デシリアライズが成功し、detailsがNoneであることを確認
        let request = result.unwrap();
        assert_eq!(request.details, None); // 【確認内容】: details省略時に#[serde(default)]でNoneになることを確認 🟡
    }
}
