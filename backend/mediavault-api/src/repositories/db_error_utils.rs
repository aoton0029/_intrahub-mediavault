//! PostgreSQL SQLSTATEコード判定の共通ヘルパー
//!
//! TASK-0015リファクタリング: tag_repository.rs / category_repository.rs で
//! 重複していたSQLSTATE判定ロジック（一意制約違反・外部キー制約違反の検出）を
//! 1箇所に集約し、保守性を高める。
//! 🟡 信頼性レベル: PostgreSQL標準のSQLSTATEコード仕様に基づく（既存実装からの抽出）

/// 【ヘルパー関数】: PostgreSQLの一意制約違反（SQLSTATE 23505）かどうかを判定する
/// 【再利用性】: tag_repository::create_tag/attach_tag_to_item、
/// category_repository::create_category/attach_category_to_item など、
/// UNIQUE制約・複合PRIMARY KEY制約を持つ全テーブルのINSERT処理で再利用できる
/// 【単一責任】: SQLSTATEコードの判定のみに責任を限定し、エラーメッセージの構築は呼び出し側に委ねる
/// 🟡 信頼性レベル: PostgreSQL標準のSQLSTATEコード仕様に基づく
pub fn is_unique_violation(err: &sqlx::Error) -> bool {
    matches!(err, sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some("23505"))
}

/// 【ヘルパー関数】: PostgreSQLの外部キー制約違反（SQLSTATE 23503）かどうかを判定する
/// 【再利用性】: item_tags/item_categoriesなど、他テーブルを参照するFK制約を持つ
/// 中間テーブルへのINSERT処理で再利用できる
/// 【単一責任】: SQLSTATEコードの判定のみに責任を限定する
/// 🟡 信頼性レベル: PostgreSQL標準のSQLSTATEコード仕様に基づく
pub fn is_foreign_key_violation(err: &sqlx::Error) -> bool {
    matches!(err, sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some("23503"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 【テスト目的】: 一意制約違反以外のエラー（接続エラー等）でfalseを返すことを確認する
    /// 【テスト内容】: sqlx::Error::PoolClosedのような非Databaseエラーを渡す
    /// 【期待される動作】: is_unique_violationがfalseを返す
    /// 🟡 信頼性レベル: 既存db_error変換テストパターンからの妥当な推測
    #[test]
    fn is_unique_violation_returns_false_for_non_database_error() {
        let err = sqlx::Error::PoolClosed;
        assert!(!is_unique_violation(&err)); // 【確認内容】: DB制約違反以外のエラーが誤判定されないことを確認 🟡
    }

    /// 【テスト目的】: 外部キー制約違反以外のエラーでfalseを返すことを確認する
    /// 🟡 信頼性レベル: is_unique_violationのテストと対称
    #[test]
    fn is_foreign_key_violation_returns_false_for_non_database_error() {
        let err = sqlx::Error::PoolClosed;
        assert!(!is_foreign_key_violation(&err)); // 【確認内容】: DB制約違反以外のエラーが誤判定されないことを確認 🟡
    }
}
