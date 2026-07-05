//! タグのモデル・リクエストDTO・バリデーション
//!
//! TASK-0015: タグ・カテゴリCRUD実装

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::response::{ApiError, ApiErrorCode};

/// タグ本体（`POST /tags`等のレスポンスで返す表現）
/// 🔵 信頼性レベル: database-schema.sqlのtagsテーブル定義（id, name）に直接対応
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
}

/// タグ + 付与アイテム件数（`GET /tags`のレスポンスで返す表現）
///
/// フロントエンドのフィルタチップ・サイドバー表示用に、タグ本体へ
/// 付与済みアイテム数（item_tags経由のCOUNT）を付加した表現。
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TagWithCount {
    pub id: Uuid,
    pub name: String,
    pub item_count: i64,
}

/// `POST /tags` リクエストDTO
/// 🔵 信頼性レベル: タスク仕様「タグ名（name）を受け取り」に直接対応
#[derive(Debug, Clone, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
}

/// 【機能概要】: タグ名が空白のみでないことを検証する
/// 【実装方針】: 既存`validate_title`（item.rs）と同一のtrim().is_empty()基準を採用する。
/// 最大長（VARCHAR(100)）の検証はDB制約に委ね、アプリ側では明示的にチェックしない
/// 【テスト対応】: create_tag_with_empty_name_returns_validation_error,
/// create_tag_with_max_length_name_succeeds を通すための実装
/// 🟡 信頼性レベル: 既存validate_titleパターンからの妥当な推測
pub fn validate_tag_name(name: &str) -> Result<(), ApiError> {
    // 【入力値検証】: 空文字・空白のみのタグ名を拒否する 🟡
    if name.trim().is_empty() {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "タグ名は空にできません",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// TC-1: タグ作成リクエストの正常デシリアライズ
    /// 【テスト目的】: `{ "name": "お気に入り" }` がCreateTagRequestへ正しくデシリアライズされることを確認する
    /// 【テスト内容】: serde_json::from_valueでCreateTagRequestを構築し、nameフィールドを検証する
    /// 【期待される動作】: name="お気に入り"のCreateTagRequestが得られる
    /// 🔵 信頼性レベル: タスク仕様テストケース1（TASK-0015.md L59-62）と完全一致
    #[test]
    fn create_tag_request_deserializes_valid_name() {
        // 【テストデータ準備】: タスク仕様の代表例をそのまま利用する
        // 【初期条件設定】: CreateTagRequestはまだmodels/tag.rsに存在しないため、この呼び出し自体がコンパイルエラーとなる想定
        let value = serde_json::json!({ "name": "お気に入り" });

        // 【実際の処理実行】: deserialize_requestと同様の形式でCreateTagRequestへ変換する
        // 【処理内容】: serdeによるJSON→構造体変換
        let request: CreateTagRequest = serde_json::from_value(value).unwrap();

        // 【結果検証】: nameフィールドが入力値と一致することを確認
        assert_eq!(request.name, "お気に入り"); // 【確認内容】: デシリアライズ結果のnameが入力JSONの値と一致することを確認 🔵
    }

    /// TC-11: タグ名が空文字でVALIDATION_ERRORになる
    /// 【テスト目的】: 空文字または空白のみのタグ名がバリデーションで拒否されることを確認する
    /// 【テスト内容】: validate_tag_nameに空文字・空白文字列を渡し、エラーが返ることを検証する
    /// 【期待される動作】: ApiErrorCode::ValidationErrorが返る
    /// 🟡 信頼性レベル: 既存validate_title（item.rs）のtrim().is_empty()パターンからの妥当な推測
    #[test]
    fn create_tag_with_empty_name_returns_validation_error() {
        // 【テストデータ準備】: 空文字と空白のみの2パターンを用意する
        // 【初期条件設定】: validate_tag_nameはまだ未実装のため、この呼び出し自体がコンパイルエラーとなる想定
        let empty = "";
        let blank = "   ";

        // 【実際の処理実行】: validate_tag_nameを呼び出す
        // 【処理内容】: 既存validate_titleと同様のtrim().is_empty()判定を期待する
        let empty_result = validate_tag_name(empty);
        let blank_result = validate_tag_name(blank);

        // 【結果検証】: 両方ともErr(ValidationError)であることを確認
        assert!(empty_result.is_err()); // 【確認内容】: 空文字がバリデーションエラーになることを確認 🟡
        assert!(blank_result.is_err()); // 【確認内容】: 空白のみの文字列もバリデーションエラーになることを確認 🟡
    }

    /// TC-15: タグ名の最大長（100文字）境界
    /// 【テスト目的】: ちょうど100文字のタグ名はバリデーションを通過することを確認する
    /// 【テスト内容】: 100文字の文字列をvalidate_tag_nameに渡し、成功することを検証する
    /// 【期待される動作】: Ok(())が返る
    /// 🟡 信頼性レベル: DBスキーマ`VARCHAR(100)`制約からの推測（タスク仕様にバリデーション要否の明記なし）
    #[test]
    fn create_tag_with_max_length_name_succeeds() {
        // 【テストデータ準備】: tags.name VARCHAR(100)の上限値である100文字の文字列を用意する
        // 【初期条件設定】: 境界値テストのため、ちょうど100文字であることが重要
        let name_100 = "a".repeat(100);

        // 【実際の処理実行】: validate_tag_nameを呼び出す
        // 【処理内容】: 文字数チェックを含むバリデーション処理を想定
        let result = validate_tag_name(&name_100);

        // 【結果検証】: 100文字は正常範囲内であることを確認
        assert!(result.is_ok()); // 【確認内容】: ちょうど100文字の境界値がエラーにならないことを確認 🟡
    }

    /// TC-16: name未指定（フィールド欠落）でのデシリアライズエラー
    /// 【テスト目的】: 必須フィールドnameが存在しないJSONがデシリアライズエラーになることを確認する
    /// 【テスト内容】: `{}`をCreateTagRequestへデシリアライズしようとし、失敗することを検証する
    /// 【期待される動作】: serde_json::from_valueがErrを返す
    /// 🔵 信頼性レベル: 既存deserialize_requestパターン（item.rs）と完全一致するため高信頼
    #[test]
    fn create_tag_request_missing_name_field_fails_deserialization() {
        // 【テストデータ準備】: nameフィールドを持たない空オブジェクトを用意する
        // 【初期条件設定】: serdeの必須フィールド欠落時の標準動作を確認する
        let value = serde_json::json!({});

        // 【実際の処理実行】: CreateTagRequestへのデシリアライズを試みる
        // 【処理内容】: serde_json::from_valueの戻り値（Result）を検証する
        let result: Result<CreateTagRequest, _> = serde_json::from_value(value);

        // 【結果検証】: デシリアライズが失敗することを確認
        assert!(result.is_err()); // 【確認内容】: 必須フィールドnameの欠落でデシリアライズが失敗することを確認 🔵
    }
}
