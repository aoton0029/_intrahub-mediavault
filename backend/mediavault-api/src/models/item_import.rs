//! POST /items/import 用リクエストDTO・バリデーション
//!
//! TASK-0025: POST /items/import（外部検索結果からのアイテムインポート）実装
//!
//! 【信頼性レベル】: 🔵 item-import-requirements.md 第2章・item-import-testcases.md TC-0025-N01より

use chrono::NaiveDate;

use crate::models::domain::MediaDetails;
use crate::models::item::{CreateItemRequest, validate_title};
use crate::models::response::{ApiError, ApiErrorCode};

/// `POST /items/import` の内部中間表現
///
/// 【機能概要】: リクエストボディ（[`MediaDetails`]）をリポジトリ層が扱う形へ落とした中間DTO。
/// ワイヤ形式はMediaDetailsに一本化されたため、本構造体はデシリアライズ対象ではない。
/// `external_id`はAPI起源アイテムの必須項目（DB CHECK制約 chk_items_source_external_id対象）。
#[derive(Debug, Clone)]
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
    /// ノーマライズ済みMediaDetailsのJSON表現。メディア別詳細テーブル用に保持する。
    pub details: Option<serde_json::Value>,
}

/// MediaCoreの`release_date`（精度がプロバイダごとに異なる文字列）を`NaiveDate`へ変換する。
///
/// "YYYY-MM-DD" を優先し、年のみ（"2003"等）は1月1日へフォールバック。
/// 解釈できない形式はNoneとし、インポート自体は拒否しない。
fn parse_release_date(raw: &str) -> Option<NaiveDate> {
    let trimmed = raw.trim();
    NaiveDate::parse_from_str(trimmed, "%Y-%m-%d")
        .ok()
        .or_else(|| {
            trimmed
                .parse::<i32>()
                .ok()
                .and_then(|year| NaiveDate::from_ymd_opt(year, 1, 1))
        })
}

impl From<MediaDetails> for ImportItemRequest {
    fn from(details: MediaDetails) -> Self {
        // 詳細テーブル用にノーマライズ済みJSONを丸ごと保持する（シリアライズは自前型のため失敗しない）
        let details_json = serde_json::to_value(&details).ok();
        let core = details.core().clone();
        ImportItemRequest {
            media_type: core.media_type,
            external_id: core.external_id,
            title: core.title,
            original_title: core.original_title,
            description: core.description,
            cover_image_url: core.image_url,
            release_date: core.release_date.as_deref().and_then(parse_release_date),
            homepage_url: core.url,
            details: details_json,
        }
    }
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

/// JSON値を`MediaDetails`としてデシリアライズし、バリデーション後に中間DTOへ変換する。
///
/// 【機能概要】: リクエストボディはGET /items/searchが返す`MediaDetails`と同形。
/// media_type不正・external_id/title欠落や空文字は400 VALIDATION_ERRORへ変換する。
/// 🔵 信頼性レベル: item-import-requirements.md 2.3データフロー、
/// item-import-testcases.md TC-0025-N01・E01〜E05より
pub fn parse_import_item_request(value: serde_json::Value) -> Result<ImportItemRequest, ApiError> {
    let details = parse_media_details_for_import(value)?;
    Ok(ImportItemRequest::from(details))
}

/// リクエストボディを`MediaDetails`へデシリアライズし、バリデーションのみ行う。
///
/// アニメ（Annict経由）のようにDB保存前にサーバー側で詳細情報を再取得・マージする
/// 必要があるケースのため、`ImportItemRequest`への変換前の`MediaDetails`を返す。
pub fn parse_media_details_for_import(value: serde_json::Value) -> Result<MediaDetails, ApiError> {
    // 【デシリアライズ】: media_typeを判別子としてMediaDetailsへ変換する。
    // デシリアライズ失敗（media_type不正値・external_id欠落等）はVALIDATION_ERRORへ変換される 🔵
    let details: MediaDetails = crate::models::item::deserialize_request(value)?;

    // 【external_idバリデーション】: 空文字・空白のみを400 VALIDATION_ERRORとして拒否する 🔵
    validate_external_id(&details.core().external_id)?;
    // 【titleバリデーション】: 既存CreateItemRequest用validate_titleを再利用し、
    // 空文字・空白のみのtitleを拒否する 🟡
    validate_title(&details.core().title)?;

    Ok(details)
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

    /// MediaDetailsのコア項目がImportItemRequestの各カラムへマッピングされる
    /// 【テスト目的】: image_url→cover_image_url、url→homepage_url、release_date文字列→NaiveDate、
    /// ノーマライズ済みJSONのdetails保持を確認する
    #[test]
    fn import_item_request_maps_media_details_core_fields() {
        // 【テストデータ準備】: 検索結果（MediaDetails）と同形の代表的リクエストボディ
        let value = serde_json::json!({
            "media_type": "movie",
            "provider": "tmdb",
            "external_id": "603",
            "title": "マトリックス",
            "original_title": "The Matrix",
            "description": "あらすじ",
            "release_date": "1999-03-31",
            "image_url": "https://image.tmdb.org/t/p/w342/poster.jpg",
            "url": "https://example.com",
            "rating": 8.2
        });

        let request = parse_import_item_request(value).unwrap();

        assert_eq!(request.media_type, MediaType::Movie); // 【確認内容】: media_typeのマッピング 🔵
        assert_eq!(request.original_title.as_deref(), Some("The Matrix"));
        assert_eq!(request.description.as_deref(), Some("あらすじ"));
        assert_eq!(
            request.cover_image_url.as_deref(),
            Some("https://image.tmdb.org/t/p/w342/poster.jpg")
        ); // 【確認内容】: image_url→cover_image_url 🔵
        assert_eq!(request.release_date, NaiveDate::from_ymd_opt(1999, 3, 31)); // 【確認内容】: "YYYY-MM-DD"文字列→NaiveDate 🔵
        assert_eq!(request.homepage_url.as_deref(), Some("https://example.com")); // 【確認内容】: url→homepage_url 🔵
        let details = request
            .details
            .expect("ノーマライズ済みJSONが保持されるはず");
        assert_eq!(details["media_type"], "movie"); // 【確認内容】: detailsにMediaDetails全体が保持される 🟡
    }

    /// release_dateが年のみ・不正形式の場合のフォールバックを確認する
    #[test]
    fn parse_release_date_handles_year_only_and_invalid_values() {
        assert_eq!(
            parse_release_date("2003"),
            NaiveDate::from_ymd_opt(2003, 1, 1)
        ); // 【確認内容】: 年のみは1月1日へフォールバック 🟡
        assert_eq!(parse_release_date("June 2, 2005"), None); // 【確認内容】: 解釈不能な形式はNone（拒否しない） 🟡
        assert_eq!(parse_release_date(""), None);
    }
}
