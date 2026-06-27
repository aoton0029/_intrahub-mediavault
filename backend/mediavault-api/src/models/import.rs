//! インポート結果サマリ型・失敗詳細型
//!
//! TASK-0030: ブクログCSVインポート実装
//!
//! 【機能概要】: `POST /import/booklog` のレスポンスとして返す `ImportSummary`・`ImportFailure` 型を定義する
//! 【実装方針】: api-endpoints.md「POST /import/booklog」のレスポンス例
//!   `{"success_count": 10, "failure_count": 2, "failures": [{"row_number": 3, "reason": "title is empty"}]}`
//!   に対応するシリアライズ可能な構造体を定義する
//! 【テスト対応】: TC-N-01・TC-E-05・TC-B-01・TC-B-02・TC-B-04
//! 🔵 信頼性レベル: 要件定義書2章「出力: ImportSummary」・api-endpoints.md「POST /import/booklog」より

use serde::Serialize;

/// インポートでスキップされた1行の詳細（行番号・不正理由）
///
/// 【機能概要】: 形式不正でスキップされた行の `row_number`（1始まり、ヘッダー行を除いたデータ行基準）
/// と `reason`（不正理由の文字列、例: "title is empty"）を保持する
/// 🔵 信頼性レベル: 要件定義書2章「ImportFailure.row_number」「ImportFailure.reason」の定義に直接対応
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ImportFailure {
    pub row_number: u32,
    pub reason: String,
}

impl ImportFailure {
    /// 【機能概要】: row_numberとreasonからImportFailureを構築するコンストラクタ
    /// 🔵 信頼性レベル: 要件定義書2章ImportFailure仕様より
    pub fn new(row_number: u32, reason: impl Into<String>) -> Self {
        Self {
            row_number,
            reason: reason.into(),
        }
    }
}

/// `POST /import/booklog` の処理結果サマリ
///
/// 【機能概要】: インポート全体の成功件数・失敗件数・失敗詳細一覧を保持する。
/// 全行が不正でも例外を起こさずこの型を200で返却する（TC-016-E01）
/// 🔵 信頼性レベル: api-endpoints.md「レスポンス（成功, 200）: ImportSummary」より
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ImportSummary {
    pub success_count: u32,
    pub failure_count: u32,
    pub failures: Vec<ImportFailure>,
}

impl ImportSummary {
    /// 【機能概要】: 成功件数0・失敗0件の空サマリを構築する（TC-B-01: ヘッダーのみCSV用）
    /// 🟡 信頼性レベル: 要件定義の「全行処理後にサマリ返却」からの妥当な推測
    pub fn empty() -> Self {
        Self::default()
    }

    /// 【機能概要】: 成功行1件を加算する
    /// 🔵 信頼性レベル: success_countの定義（登録に成功した行数）に対応
    pub fn record_success(&mut self) {
        self.success_count += 1;
    }

    /// 【機能概要】: 失敗行を1件記録する（failure_countはfailures.len()と常に一致させる）
    /// 🔵 信頼性レベル: 要件定義書2章「failure_count = failures.len()」に対応
    pub fn record_failure(&mut self, failure: ImportFailure) {
        self.failures.push(failure);
        self.failure_count = self.failures.len() as u32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// TC-N-01関連: ImportSummaryの初期状態（empty）はsuccess_count=0/failure_count=0/failures=[]
    /// 【テスト目的】: ヘッダーのみ・データ0行CSV（TC-B-01）で返す空サマリの構造を確認する
    /// 【テスト内容】: ImportSummary::empty()を呼び出し、各フィールドの初期値を検証する
    /// 【期待される動作】: success_count==0, failure_count==0, failures==Vec::new()
    /// 🟡 信頼性レベル: 要件定義の「全行処理後にサマリ返却」からの妥当な推測（TC-B-01対応）
    #[test]
    fn import_summary_empty_has_zero_counts() {
        // 【テストデータ準備】: データ0行のCSV処理結果を想定した空サマリ
        // 【初期条件設定】: まだImportSummary::empty()は未実装のためコンパイルエラーとなる想定
        let summary = ImportSummary::empty();

        // 【結果検証】: 全フィールドが初期値（0件・空配列）であることを確認
        assert_eq!(summary.success_count, 0); // 【確認内容】: success_countの初期値が0であることを確認 🟡
        assert_eq!(summary.failure_count, 0); // 【確認内容】: failure_countの初期値が0であることを確認 🟡
        assert_eq!(summary.failures.len(), 0); // 【確認内容】: failuresが空配列であることを確認 🟡
    }

    /// TC-N-01: ImportSummaryのJSONシリアライズ形式がapi-endpoints.mdの例に一致する
    /// 【テスト目的】: success_count/failure_count/failuresが期待するキー名・型でシリアライズされることを確認する
    /// 【テスト内容】: record_success/record_failureで構築したImportSummaryをserde_json::to_valueでシリアライズする
    /// 【期待される動作】: {"success_count":1,"failure_count":1,"failures":[{"row_number":3,"reason":"title is empty"}]}
    /// 🔵 信頼性レベル: api-endpoints.md「POST /import/booklog」レスポンス例に直接対応
    #[test]
    fn import_summary_serializes_to_expected_json_shape() {
        // 【テストデータ準備】: 成功1件・失敗1件（row_number=3, reason="title is empty"）のサマリを構築する
        // 【初期条件設定】: TC-N-01のレスポンス例（要件定義書2章）に対応する最小ケース
        let mut summary = ImportSummary::empty();
        summary.record_success();
        summary.record_failure(ImportFailure::new(3, "title is empty"));

        // 【実際の処理実行】: serde_json::to_valueでシリアライズする
        // 【処理内容】: ImportSummary/ImportFailureのSerialize実装によるJSON変換
        let json = serde_json::to_value(&summary).unwrap();

        // 【結果検証】: トップレベルキーとfailures配列要素の形状が要件通りであることを確認
        assert_eq!(
            json,
            serde_json::json!({
                "success_count": 1,
                "failure_count": 1,
                "failures": [{ "row_number": 3, "reason": "title is empty" }]
            })
        ); // 【確認内容】: success_count/failure_count/failures各キー名・値が要件のレスポンス例に一致することを確認 🔵
    }

    /// TC-E-05: record_failureを複数回呼ぶとfailure_countがfailures.len()と一致し続ける
    /// 【テスト目的】: 複数の不正行を記録した際にfailure_countがfailures配列長と常に同期することを確認する
    /// 【テスト内容】: row_number=2(title is empty)とrow_number=4(invalid date format)を記録する
    /// 【期待される動作】: failures.len()==2、failure_count==2、各要素のrow_number/reasonが一致
    /// 🔵 信頼性レベル: 要件定義2章ImportFailure仕様・EDGE-002に直接対応
    #[test]
    fn import_summary_record_failure_keeps_failure_count_in_sync() {
        // 【テストデータ準備】: 複数種の不正理由を持つ2件の失敗行を用意する
        // 【初期条件設定】: TC-E-05「行2=title空、行4=日付不正」のシナリオを再現する
        let mut summary = ImportSummary::empty();
        summary.record_failure(ImportFailure::new(2, "title is empty"));
        summary.record_failure(ImportFailure::new(4, "invalid date format"));

        // 【実際の処理実行】: record_failureを2回呼び出した後の状態を取得する
        // 【処理内容】: 複数件の失敗記録の蓄積処理を確認する

        // 【結果検証】: failure_countとfailures.len()が一致し、各要素の内容が正確であることを確認
        assert_eq!(summary.failure_count, 2); // 【確認内容】: failure_countがfailures.len()と一致（2件）することを確認 🔵
        assert_eq!(summary.failures.len(), 2); // 【確認内容】: failures配列に2件格納されていることを確認 🔵
        assert_eq!(summary.failures[0], ImportFailure::new(2, "title is empty")); // 【確認内容】: 1件目のrow_number/reasonが正確であることを確認 🔵
        assert_eq!(
            summary.failures[1],
            ImportFailure::new(4, "invalid date format")
        ); // 【確認内容】: 2件目のrow_number/reasonが正確であることを確認 🔵
    }
}
