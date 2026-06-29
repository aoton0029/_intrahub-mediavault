//! staff（スタッフ管理）のモデル・リクエストDTO・バリデーション
//!
//! TASK-0020: スタッフ管理CRUD実装（models/item_relation.rsと対称な構造）
//!
//! 【実装状況】: Greenフェーズで全関数を実装済み。parse_*関数はitems CRUDの
//! parse_create_item_request等と同様、pure（DBアクセスなし）なバリデーション関数として
//! ハンドラ層から呼び出される。

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::response::{ApiError, ApiErrorCode};

/// staff本体（`POST /staff`のレスポンスで返す表現）
/// 🔵 信頼性レベル: database-schema.sqlのstaffテーブル定義に直接対応
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Staff {
    pub id: Uuid,
    pub external_id: Option<String>,
    pub name: String,
    pub image_url: Option<String>,
    pub created_at: NaiveDateTime,
}

/// item_staff本体（`POST /items/:id/staff`のレスポンスで返す表現）
/// 🔵 信頼性レベル: database-schema.sqlのitem_staffテーブル定義に直接対応
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemStaff {
    pub id: Uuid,
    pub item_id: Uuid,
    pub staff_id: Uuid,
    pub role: String,
    pub character_name: Option<String>,
}

/// `POST /staff` リクエストDTO
/// 🔵 信頼性レベル: staff-requirements.md 2.1「name必須・external_id/image_url optional」に直接対応
#[derive(Debug, Clone, Deserialize)]
pub struct CreateStaffRequest {
    pub name: String,
    pub external_id: Option<String>,
    pub image_url: Option<String>,
}

/// `POST /items/:id/staff` リクエストDTO
/// 🔵 信頼性レベル: staff-requirements.md 2.2「staff_id必須・role必須・character_name optional」に直接対応
#[derive(Debug, Clone, Deserialize)]
pub struct CreateItemStaffRequest {
    pub staff_id: Uuid,
    pub role: String,
    pub character_name: Option<String>,
}

/// role列の最大長（VARCHAR(100)）
/// 🔵 信頼性レベル: staff-requirements.md 3 入力検証制約「role上限100文字」に明記
pub const ROLE_MAX_LEN: usize = 100;

/// character_name列の最大長（VARCHAR(255)）
/// 🔵 信頼性レベル: staff-requirements.md 3 入力検証制約「character_name上限255文字」に明記
pub const CHARACTER_NAME_MAX_LEN: usize = 255;

/// 【機能概要】: CreateStaffRequestのバリデーション（name空文字拒否）を行う
/// 【実装方針】: items CRUDのparse_create_item_requestと同等のパターン（note.md）。
/// trim後に空文字ならVALIDATION_ERRORとし、INSERT前に早期リターンする。
/// 【テスト対応】: TC-N-06（正常パース）, TC-E-01（空文字拒否）を通すための実装
/// 🔵 信頼性レベル: staff-requirements.md 2.1 / TC-N-06, TC-E-01 に対応
pub fn parse_create_staff_request(
    request: CreateStaffRequest,
) -> Result<CreateStaffRequest, ApiError> {
    // 【入力値検証】: nameが空文字（trim後）の場合はDB不変のまま早期リターンする 🔵
    if request.name.trim().is_empty() {
        // 【エラー処理】: nameはNOT NULL・空文字不可（staff-requirements.md 2.1） 🔵
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "nameは必須です",
        ));
    }

    // 【入力値検証】: nameの長さ制限（VARCHAR(255)）も合わせて確認する 🟡
    if request.name.chars().count() > 255 {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "nameは255文字以内で指定してください",
        ));
    }

    // 【結果返却】: 検証済みのリクエストをそのまま返す（items CRUDと同等のパターン） 🔵
    Ok(request)
}

/// 【機能概要】: CreateItemStaffRequestのバリデーション（role空文字・長さ制限・character_name長さ制限）を行う
/// 【実装方針】: role上限100文字・character_name上限255文字をチェックする
/// 【テスト対応】: TC-E-06, TC-B-02, TC-B-03 を通すための実装
/// 🔵 信頼性レベル: staff-requirements.md 2.2・3 / TC-E-06, TC-B-02, TC-B-03 に対応
pub fn parse_create_item_staff_request(
    request: CreateItemStaffRequest,
) -> Result<CreateItemStaffRequest, ApiError> {
    // 【入力値検証】: roleが空文字（trim後）の場合は早期リターン 🔵
    if request.role.trim().is_empty() {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "roleは必須です",
        ));
    }

    // 【入力値検証】: roleの長さ制限（VARCHAR(100)、上限100文字） 🔵
    if request.role.chars().count() > ROLE_MAX_LEN {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            format!("roleは{ROLE_MAX_LEN}文字以内で指定してください"),
        ));
    }

    // 【入力値検証】: character_nameの長さ制限（VARCHAR(255)、上限255文字、optional） 🔵
    if let Some(character_name) = &request.character_name
        && character_name.chars().count() > CHARACTER_NAME_MAX_LEN
    {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            format!("character_nameは{CHARACTER_NAME_MAX_LEN}文字以内で指定してください"),
        ));
    }

    // 【結果返却】: 検証済みのリクエストをそのまま返す 🔵
    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::response::ApiErrorCode;

    /// TC-N-06: 有効なnameでparse_create_staff_requestが成功する
    /// 🔵 信頼性レベル: note.md セクション2「parse_*関数」パターン / item.rs参考実装に基づく
    #[test]
    fn parse_create_staff_request_accepts_valid_name() {
        // 【テスト目的】: 正常な入力（必須nameのみ指定）でバリデーションがOkを返すことを確認する
        // 【テスト内容】: CreateStaffRequest { name: "監督A", external_id: None, image_url: None } を渡す
        // 【期待される動作】: parse_create_staff_requestがOk(req)を返し、req.nameが保持される
        // 🔵 信頼性レベル: staff-requirements.md 2.1 / TASK-0020テストケース1に明記

        // 【テストデータ準備】: ハンドラ手前のpure関数を単体で検証（DB不要）
        // 【初期条件設定】: 必須のnameのみを設定し、optional項目はNoneとする
        let req = CreateStaffRequest {
            name: "監督A".to_string(),
            external_id: None,
            image_url: None,
        };

        // 【実際の処理実行】: バリデーション関数を直接呼び出す
        // 【処理内容】: name空文字チェックを通過させる
        let result = parse_create_staff_request(req);

        // 【結果検証】: Result::Ok であり、nameの値が保持されていること
        // 【期待値確認】: items CRUDの既存パターンと同様、検証済みリクエストがそのまま返る
        assert!(result.is_ok()); // 【確認内容】: 正常なnameはバリデーションを通過することを確認 🔵
        assert_eq!(result.unwrap().name, "監督A"); // 【確認内容】: name値が変更されずに保持されることを確認 🔵
    }

    /// TC-E-01: 空のnameはバリデーションエラーになる
    /// 🔵 信頼性レベル: staff-requirements.md 2.1 / 4.3「role空文字 / 不正UUID → 400」と同方針
    #[test]
    fn parse_create_staff_request_rejects_empty_name() {
        // 【テスト目的】: 空nameがVALIDATION_ERRORで弾かれることを確認する
        // 【テスト内容】: parse_create_staff_requestに空文字nameを渡す
        // 【期待される動作】: Err(ApiError { code: VALIDATION_ERROR }) が返る
        // 🔵 信頼性レベル: staff-requirements.md 2.1 に明記

        // 【テストデータ準備】: 空文字nameのリクエストDTOを構築（DB不要のpure関数テスト）
        // 【初期条件設定】: nameを空文字列に設定し、不正入力状態を再現する
        let req = CreateStaffRequest {
            name: "".into(),
            external_id: None,
            image_url: None,
        };

        // 【実際の処理実行】: バリデーション関数を直接呼び出す
        // 【処理内容】: name空文字チェックで早期リターンされることを期待する
        let result = parse_create_staff_request(req);

        // 【結果検証】: Errであり、エラーコードがVALIDATION_ERRORであること
        // 【期待値確認】: 不正データのDB混入を防ぐための早期リターンが機能していること
        assert!(result.is_err()); // 【検証項目】: 空nameは拒否される 🔵
        assert_eq!(
            result.unwrap_err().error.code,
            ApiErrorCode::ValidationError.as_code_str()
        ); // 【確認内容】: 正しいエラーコードが返ることを確認 🔵
    }

    /// TC-E-06: 空のroleはバリデーションエラーになる
    /// 🔵 信頼性レベル: staff-requirements.md 2.2・4.3 に明記
    #[test]
    fn parse_create_item_staff_request_rejects_empty_role() {
        // 【テスト目的】: roleが空文字の場合にVALIDATION_ERRORが返ることを確認する
        // 【テスト内容】: parse_create_item_staff_requestにrole=""のリクエストを渡す
        // 【期待される動作】: Err(ApiError { code: VALIDATION_ERROR }) が返る
        // 🔵 信頼性レベル: staff-requirements.md 2.2・4.3 に明記

        // 【テストデータ準備】: role空文字のCreateItemStaffRequestを構築
        // 【初期条件設定】: staff_idはダミーUUID、roleのみ不正値とする
        let req = CreateItemStaffRequest {
            staff_id: Uuid::new_v4(),
            role: "".into(),
            character_name: None,
        };

        // 【実際の処理実行】: バリデーション関数を直接呼び出す
        // 【処理内容】: role空文字チェックで早期リターンされることを期待する
        let result = parse_create_item_staff_request(req);

        // 【結果検証】: Errであり、エラーコードがVALIDATION_ERRORであること
        // 【期待値確認】: 役割なし紐付けの混入を防ぐための早期リターンが機能していること
        assert!(result.is_err()); // 【検証項目】: 空roleは拒否される 🔵
        assert_eq!(
            result.unwrap_err().error.code,
            ApiErrorCode::ValidationError.as_code_str()
        ); // 【確認内容】: 正しいエラーコードが返ることを確認 🔵
    }

    /// TC-B-02(a): role 100文字ちょうどで成功する
    /// 🔵 信頼性レベル: staff-requirements.md 3 入力検証制約「role上限100文字」に明記
    #[test]
    fn parse_create_item_staff_request_accepts_role_at_max_length() {
        // 【テスト目的】: roleがちょうど100文字（上限）の場合に検証を通過することを確認する
        // 【テスト内容】: role=100文字の文字列を渡す
        // 【期待される動作】: Result::Okが返り、roleが切り詰められずそのまま保持される
        // 🔵 信頼性レベル: staff-requirements.md 3「role上限100文字」を直接検証

        // 【テストデータ準備】: ROLE_MAX_LEN（100）文字のASCII文字列を生成
        // 【初期条件設定】: 境界値ちょうどでの動作を確認するためのデータ
        let role_100 = "a".repeat(ROLE_MAX_LEN);
        let req = CreateItemStaffRequest {
            staff_id: Uuid::new_v4(),
            role: role_100.clone(),
            character_name: None,
        };

        // 【実際の処理実行】: バリデーション関数を直接呼び出す
        // 【処理内容】: 上限ちょうどの長さチェックを通過させる
        let result = parse_create_item_staff_request(req);

        // 【結果検証】: Result::Okであり、roleが100文字のまま保持されること
        // 【期待値確認】: 境界の内外で判定が反転する仕様の「内側」を確認
        assert!(result.is_ok()); // 【確認内容】: 上限ちょうど（100文字）は許可されることを確認 🔵
        assert_eq!(result.unwrap().role.len(), ROLE_MAX_LEN); // 【確認内容】: roleが切り詰められず保持されることを確認 🔵
    }

    /// TC-B-02(b): role 101文字で400 VALIDATION_ERRORになる
    /// 🔵 信頼性レベル: staff-requirements.md 3 入力検証制約「role上限100文字」に明記
    #[test]
    fn parse_create_item_staff_request_rejects_role_exceeding_max_length() {
        // 【テスト目的】: roleが上限を1文字超過（101文字）した場合に拒否されることを確認する
        // 【テスト内容】: role=101文字の文字列を渡す
        // 【期待される動作】: Err(ApiError { code: VALIDATION_ERROR }) が返る
        // 🔵 信頼性レベル: staff-requirements.md 3「role上限100文字」を直接検証

        // 【テストデータ準備】: ROLE_MAX_LEN+1（101）文字のASCII文字列を生成
        // 【初期条件設定】: 境界値の「外側」を確認するためのデータ
        let role_101 = "a".repeat(ROLE_MAX_LEN + 1);
        let req = CreateItemStaffRequest {
            staff_id: Uuid::new_v4(),
            role: role_101,
            character_name: None,
        };

        // 【実際の処理実行】: バリデーション関数を直接呼び出す
        // 【処理内容】: 上限超過の長さチェックで早期リターンされることを期待する
        let result = parse_create_item_staff_request(req);

        // 【結果検証】: Errであり、エラーコードがVALIDATION_ERRORであること
        // 【期待値確認】: 境界の内外で判定が反転する仕様の「外側」を確認
        assert!(result.is_err()); // 【確認内容】: 上限超過（101文字）は拒否されることを確認 🔵
        assert_eq!(
            result.unwrap_err().error.code,
            ApiErrorCode::ValidationError.as_code_str()
        ); // 【確認内容】: 正しいエラーコードが返ることを確認 🔵
    }

    /// TC-B-03(a): character_name 255文字ちょうどで成功する
    /// 🔵 信頼性レベル: staff-requirements.md 3 入力検証制約「character_name上限255文字」に明記
    #[test]
    fn parse_create_item_staff_request_accepts_character_name_at_max_length() {
        // 【テスト目的】: character_nameがちょうど255文字（上限）の場合に検証を通過することを確認する
        // 【テスト内容】: character_name=255文字の文字列を渡す
        // 【期待される動作】: Result::Okが返り、character_nameがそのまま保持される
        // 🔵 信頼性レベル: staff-requirements.md 3「character_name上限255文字」を直接検証

        // 【テストデータ準備】: CHARACTER_NAME_MAX_LEN（255）文字のASCII文字列を生成
        // 【初期条件設定】: roleは正常値、character_nameのみ境界値とする
        let character_name_255 = "c".repeat(CHARACTER_NAME_MAX_LEN);
        let req = CreateItemStaffRequest {
            staff_id: Uuid::new_v4(),
            role: "声優".to_string(),
            character_name: Some(character_name_255.clone()),
        };

        // 【実際の処理実行】: バリデーション関数を直接呼び出す
        // 【処理内容】: character_nameの上限ちょうどの長さチェックを通過させる
        let result = parse_create_item_staff_request(req);

        // 【結果検証】: Result::Okであり、character_nameが255文字のまま保持されること
        assert!(result.is_ok()); // 【確認内容】: 上限ちょうど（255文字）は許可されることを確認 🔵
        assert_eq!(
            result.unwrap().character_name.unwrap().len(),
            CHARACTER_NAME_MAX_LEN
        ); // 【確認内容】: character_nameが切り詰められず保持されることを確認 🔵
    }

    /// TC-B-03(b): character_name 256文字で400 VALIDATION_ERRORになる
    /// 🔵 信頼性レベル: staff-requirements.md 3 入力検証制約「character_name上限255文字」に明記
    #[test]
    fn parse_create_item_staff_request_rejects_character_name_exceeding_max_length() {
        // 【テスト目的】: character_nameが上限を1文字超過（256文字）した場合に拒否されることを確認する
        // 【テスト内容】: character_name=256文字の文字列を渡す
        // 【期待される動作】: Err(ApiError { code: VALIDATION_ERROR }) が返る
        // 🔵 信頼性レベル: staff-requirements.md 3「character_name上限255文字」を直接検証

        // 【テストデータ準備】: CHARACTER_NAME_MAX_LEN+1（256）文字のASCII文字列を生成
        let character_name_256 = "c".repeat(CHARACTER_NAME_MAX_LEN + 1);
        let req = CreateItemStaffRequest {
            staff_id: Uuid::new_v4(),
            role: "声優".to_string(),
            character_name: Some(character_name_256),
        };

        // 【実際の処理実行】: バリデーション関数を直接呼び出す
        // 【処理内容】: character_nameの上限超過の長さチェックで早期リターンされることを期待する
        let result = parse_create_item_staff_request(req);

        // 【結果検証】: Errであり、エラーコードがVALIDATION_ERRORであること
        assert!(result.is_err()); // 【確認内容】: 上限超過（256文字）は拒否されることを確認 🔵
        assert_eq!(
            result.unwrap_err().error.code,
            ApiErrorCode::ValidationError.as_code_str()
        ); // 【確認内容】: 正しいエラーコードが返ることを確認 🔵
    }

    /// TC-N-02関連: CreateStaffRequestがexternal_id/image_url込みで正しくデシリアライズされる
    /// 🔵 信頼性レベル: staff-requirements.md 2.1 / note.md セクション4 入力仕様に基づく
    #[test]
    fn create_staff_request_deserializes_all_fields() {
        // 【テスト目的】: name/external_id/image_url全フィールド指定時に正しくデシリアライズされることを確認する
        // 【テスト内容】: serde_json::from_valueでJSONからCreateStaffRequestへ変換する
        // 【期待される動作】: 全フィールドの値がそのまま保持される
        // 🔵 信頼性レベル: staff-requirements.md 2.1 に明記

        // 【テストデータ準備】: TC-N-02相当の入力（全フィールド指定）
        let value = serde_json::json!({
            "name": "声優B",
            "external_id": "anilist-12345",
            "image_url": "https://example.com/b.png"
        });

        // 【実際の処理実行】: serde_jsonによるデシリアライズ
        let request: CreateStaffRequest = serde_json::from_value(value).unwrap();

        // 【結果検証】: 各フィールドが入力値と一致すること
        assert_eq!(request.name, "声優B"); // 【確認内容】: name値が保持されることを確認 🔵
        assert_eq!(request.external_id, Some("anilist-12345".to_string())); // 【確認内容】: external_id値が保持されることを確認 🔵
        assert_eq!(
            request.image_url,
            Some("https://example.com/b.png".to_string())
        ); // 【確認内容】: image_url値が保持されることを確認 🔵
    }

    /// TC-E-07: 不正なUUID形式のstaff_idはデシリアライズエラーになる
    /// 🟡 信頼性レベル: staff-requirements.md 2.2・4.3「不正UUID → 400」（🟡）
    #[test]
    fn create_item_staff_request_with_invalid_uuid_fails_deserialization() {
        // 【テスト目的】: staff_idがUUID形式でない文字列の場合、デシリアライズ自体が失敗することを確認する
        // 【テスト内容】: staff_id="not-a-uuid"を含むJSONをCreateItemStaffRequestへ変換する
        // 【期待される動作】: serde_json::from_valueがErrを返す
        // 🟡 信頼性レベル: staff-requirements.md 2.2・4.3「不正UUID → 400」（🟡）

        // 【テストデータ準備】: UUIDとして解釈不能な文字列を含むJSON
        let value = serde_json::json!({
            "staff_id": "not-a-uuid",
            "role": "監督",
            "character_name": null
        });

        // 【実際の処理実行】: serde_jsonによるデシリアライズ
        let result: Result<CreateItemStaffRequest, _> = serde_json::from_value(value);

        // 【結果検証】: デシリアライズが失敗すること（型不正の早期検出）
        assert!(result.is_err()); // 【確認内容】: 不正なUUID文字列がデシリアライズ段階で拒否されることを確認 🟡
    }
}
