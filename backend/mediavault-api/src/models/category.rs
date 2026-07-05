//! カテゴリのモデル・リクエストDTO・バリデーション
//!
//! TASK-0015: タグ・カテゴリCRUD実装（models/tag.rsと完全に対称な構造）

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::response::{ApiError, ApiErrorCode};

/// カテゴリ本体（`POST /categories`等のレスポンスで返す表現）
/// 🔵 信頼性レベル: database-schema.sqlのcategoriesテーブル定義（id, name）に直接対応
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
}

/// カテゴリ + 付与アイテム件数（`GET /categories`のレスポンスで返す表現）
/// 🟡 信頼性レベル: models::tag::TagWithCountと完全に対称
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CategoryWithCount {
    pub id: Uuid,
    pub name: String,
    pub item_count: i64,
}

/// `POST /categories` リクエストDTO
/// 🔵 信頼性レベル: タグと対称、タスク仕様「カテゴリのitem付与・削除」に対応
#[derive(Debug, Clone, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
}

/// 【機能概要】: カテゴリ名が空白のみでないことを検証する
/// 【実装方針】: validate_tag_name（models/tag.rs）と完全に対称な実装
/// 🟡 信頼性レベル: validate_tag_nameと同様の妥当な推測
pub fn validate_category_name(name: &str) -> Result<(), ApiError> {
    // 【入力値検証】: 空文字・空白のみのカテゴリ名を拒否する 🟡
    if name.trim().is_empty() {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "カテゴリ名は空にできません",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// TC-2: カテゴリ作成リクエストの正常デシリアライズ
    /// 【テスト目的】: `{ "name": "本棚A" }` がCreateCategoryRequestへ正しくデシリアライズされることを確認する
    /// 【テスト内容】: serde_json::from_valueでCreateCategoryRequestを構築し、nameフィールドを検証する
    /// 【期待される動作】: name="本棚A"のCreateCategoryRequestが得られる
    /// 🔵 信頼性レベル: タスク仕様テストケース4（TASK-0015.md L74-77）と完全一致
    #[test]
    fn create_category_request_deserializes_valid_name() {
        // 【テストデータ準備】: タスク仕様の代表例をそのまま利用する
        // 【初期条件設定】: CreateCategoryRequestはまだmodels/category.rsに存在しないため、この呼び出し自体がコンパイルエラーとなる想定
        let value = serde_json::json!({ "name": "本棚A" });

        // 【実際の処理実行】: CreateCategoryRequestへ変換する
        // 【処理内容】: serdeによるJSON→構造体変換
        let request: CreateCategoryRequest = serde_json::from_value(value).unwrap();

        // 【結果検証】: nameフィールドが入力値と一致することを確認
        assert_eq!(request.name, "本棚A"); // 【確認内容】: デシリアライズ結果のnameが入力JSONの値と一致することを確認 🔵
    }

    /// TC-12: カテゴリ名が空文字でVALIDATION_ERRORになる
    /// 【テスト目的】: 空文字または空白のみのカテゴリ名がバリデーションで拒否されることを確認する
    /// 【テスト内容】: validate_category_nameに空文字・空白文字列を渡し、エラーが返ることを検証する
    /// 【期待される動作】: ApiErrorCode::ValidationErrorが返る
    /// 🟡 信頼性レベル: TC-11（タグ）と対称な推測
    #[test]
    fn create_category_with_empty_name_returns_validation_error() {
        // 【テストデータ準備】: 空文字と空白のみの2パターンを用意する
        // 【初期条件設定】: validate_category_nameはまだ未実装のため、この呼び出し自体がコンパイルエラーとなる想定
        let empty = "";
        let blank = "   ";

        // 【実際の処理実行】: validate_category_nameを呼び出す
        // 【処理内容】: タグ用バリデーションと対称の実装を期待する
        let empty_result = validate_category_name(empty);
        let blank_result = validate_category_name(blank);

        // 【結果検証】: 両方ともErr(ValidationError)であることを確認
        assert!(empty_result.is_err()); // 【確認内容】: 空文字がバリデーションエラーになることを確認 🟡
        assert!(blank_result.is_err()); // 【確認内容】: 空白のみの文字列もバリデーションエラーになることを確認 🟡
    }

    /// TC-16相当（カテゴリ版）: name未指定でのデシリアライズエラー
    /// 【テスト目的】: 必須フィールドnameが存在しないJSONがデシリアライズエラーになることを確認する
    /// 【テスト内容】: `{}`をCreateCategoryRequestへデシリアライズしようとし、失敗することを検証する
    /// 【期待される動作】: serde_json::from_valueがErrを返す
    /// 🔵 信頼性レベル: 既存deserialize_requestパターン（item.rs）と完全一致するため高信頼
    #[test]
    fn create_category_request_missing_name_field_fails_deserialization() {
        // 【テストデータ準備】: nameフィールドを持たない空オブジェクトを用意する
        let value = serde_json::json!({});

        // 【実際の処理実行】: CreateCategoryRequestへのデシリアライズを試みる
        let result: Result<CreateCategoryRequest, _> = serde_json::from_value(value);

        // 【結果検証】: デシリアライズが失敗することを確認
        assert!(result.is_err()); // 【確認内容】: 必須フィールドnameの欠落でデシリアライズが失敗することを確認 🔵
    }
}
