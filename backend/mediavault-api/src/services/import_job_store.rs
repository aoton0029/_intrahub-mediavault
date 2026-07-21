//! ブクログインポートジョブの進捗を保持するインメモリストア
//!
//! 楽天ブックスAPIでのエンリッチメントは1リクエスト/ISBN・約1秒間隔でしか行えず、
//! 実際のブクログエクスポート規模（数千ISBN）では処理完了までに数十分かかることがある。
//! そのため`POST /import/booklog`はジョブを起動して即座に`job_id`を返し、クライアントは
//! `GET /import/booklog/jobs/:job_id`をポーリングして進捗（processed/total）と、
//! 完了後の最終`ImportSummary`を取得する。
//!
//! プロセス内限定（アプリ再起動で消える）の単純なレジストリで十分なため、
//! 永続化やジョブキューは導入しない。

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use uuid::Uuid;

use crate::models::import::{ImportJobState, ImportJobStatus, ImportSummary};

static JOBS: LazyLock<Mutex<HashMap<Uuid, ImportJobStatus>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// 新規ジョブを`Running`状態で登録し、採番した`job_id`を返す。
pub fn create_job(total: u32) -> Uuid {
    let job_id = Uuid::new_v4();
    let status = ImportJobStatus {
        status: ImportJobState::Running,
        total,
        processed: 0,
        summary: None,
    };
    JOBS.lock()
        .expect("import job store mutex poisoned")
        .insert(job_id, status);
    job_id
}

/// 1件処理が終わるたびに`processed`を1加算する。ジョブが存在しない場合は何もしない。
pub fn advance(job_id: Uuid) {
    if let Some(status) = JOBS
        .lock()
        .expect("import job store mutex poisoned")
        .get_mut(&job_id)
    {
        status.processed += 1;
    }
}

/// ジョブを`Completed`状態にし、最終`ImportSummary`を格納する。
pub fn complete(job_id: Uuid, summary: ImportSummary) {
    if let Some(status) = JOBS
        .lock()
        .expect("import job store mutex poisoned")
        .get_mut(&job_id)
    {
        status.status = ImportJobState::Completed;
        status.processed = status.total;
        status.summary = Some(summary);
    }
}

/// 中止をリクエストする。`Running`のジョブのみ`Cancelling`へ遷移させる
/// （既に`Completed`/`Cancelled`のジョブへの中止リクエストは無視する）。
pub fn request_cancel(job_id: Uuid) {
    if let Some(status) = JOBS
        .lock()
        .expect("import job store mutex poisoned")
        .get_mut(&job_id)
        && status.status == ImportJobState::Running
    {
        status.status = ImportJobState::Cancelling;
    }
}

/// バックグラウンドループが中止リクエストを検知するために使う。
pub fn is_cancelling(job_id: Uuid) -> bool {
    JOBS.lock()
        .expect("import job store mutex poisoned")
        .get(&job_id)
        .is_some_and(|status| status.status == ImportJobState::Cancelling)
}

/// ジョブを`Cancelled`状態にし、中止時点までの`ImportSummary`を格納する。
/// `processed`は`total`まで進めず、中止時点の値のまま維持する。
pub fn cancel(job_id: Uuid, summary: ImportSummary) {
    if let Some(status) = JOBS
        .lock()
        .expect("import job store mutex poisoned")
        .get_mut(&job_id)
    {
        status.status = ImportJobState::Cancelled;
        status.summary = Some(summary);
    }
}

/// 現在のジョブ状態を取得する（存在しなければNone）。
pub fn get(job_id: Uuid) -> Option<ImportJobStatus> {
    JOBS.lock()
        .expect("import job store mutex poisoned")
        .get(&job_id)
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::import::ImportFailure;

    #[test]
    fn create_job_starts_running_with_zero_processed() {
        let job_id = create_job(5);
        let status = get(job_id).unwrap();
        assert_eq!(status.status, ImportJobState::Running);
        assert_eq!(status.total, 5);
        assert_eq!(status.processed, 0);
        assert!(status.summary.is_none());
    }

    #[test]
    fn advance_increments_processed_count() {
        let job_id = create_job(3);
        advance(job_id);
        advance(job_id);
        let status = get(job_id).unwrap();
        assert_eq!(status.processed, 2);
        assert_eq!(status.status, ImportJobState::Running);
    }

    #[test]
    fn complete_marks_job_completed_with_summary() {
        let job_id = create_job(2);
        let mut summary = ImportSummary::empty();
        summary.record_success();
        summary.record_failure(ImportFailure::new(1, "title is empty"));
        complete(job_id, summary.clone());

        let status = get(job_id).unwrap();
        assert_eq!(status.status, ImportJobState::Completed);
        assert_eq!(status.processed, status.total);
        assert_eq!(status.summary, Some(summary));
    }

    #[test]
    fn get_returns_none_for_unknown_job_id() {
        assert!(get(Uuid::new_v4()).is_none());
    }

    #[test]
    fn advance_on_unknown_job_id_does_not_panic() {
        advance(Uuid::new_v4());
    }

    #[test]
    fn request_cancel_transitions_running_job_to_cancelling() {
        let job_id = create_job(5);
        request_cancel(job_id);
        let status = get(job_id).unwrap();
        assert_eq!(status.status, ImportJobState::Cancelling);
        assert!(is_cancelling(job_id));
    }

    #[test]
    fn request_cancel_on_completed_job_is_ignored() {
        let job_id = create_job(1);
        complete(job_id, ImportSummary::empty());
        request_cancel(job_id);
        let status = get(job_id).unwrap();
        assert_eq!(status.status, ImportJobState::Completed);
    }

    #[test]
    fn cancel_marks_job_cancelled_with_partial_summary_and_keeps_processed() {
        let job_id = create_job(10);
        advance(job_id);
        advance(job_id);
        request_cancel(job_id);

        let mut summary = ImportSummary::empty();
        summary.record_success();
        cancel(job_id, summary.clone());

        let status = get(job_id).unwrap();
        assert_eq!(status.status, ImportJobState::Cancelled);
        assert_eq!(status.processed, 2);
        assert_eq!(status.total, 10);
        assert_eq!(status.summary, Some(summary));
    }

    #[test]
    fn request_cancel_on_unknown_job_id_does_not_panic() {
        request_cancel(Uuid::new_v4());
    }

    #[test]
    fn is_cancelling_returns_false_for_unknown_job_id() {
        assert!(!is_cancelling(Uuid::new_v4()));
    }
}
