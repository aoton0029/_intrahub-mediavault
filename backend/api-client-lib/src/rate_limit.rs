use std::time::{Duration, Instant};

use crate::error::ApiError;

use tracing;

/// レート制限状態（シンプルカウンター）。
#[derive(Debug)]
pub struct RateLimitState {
    /// 現在ウィンドウのリクエスト数
    pub count: u32,
    /// 現在ウィンドウの開始時刻
    pub window_start: Instant,
    /// ウィンドウあたりの上限リクエスト数
    pub limit: u32,
    /// ウィンドウ長（秒）
    pub window_secs: u64,
}

impl RateLimitState {
    /// 新しい `RateLimitState` を作成する。
    pub fn new(limit: u32, window_secs: u64) -> Self {
        Self {
            count: 0,
            window_start: Instant::now(),
            limit,
            window_secs,
        }
    }

    /// レート制限チェックとカウンターインクリメント。
    ///
    /// ウィンドウ経過済みならリセット、制限超過なら `Err(ApiError::RateLimit)` を返す。
    pub fn check_and_increment(&mut self) -> Result<(), ApiError> {
        let elapsed = self.window_start.elapsed().as_secs();
        if elapsed >= self.window_secs {
            self.count = 1;
            self.window_start = Instant::now();
            tracing::debug!(
                count = 1,
                limit = self.limit,
                window_secs = self.window_secs,
                "rate limit window reset"
            );
            return Ok(());
        }
        if self.count >= self.limit {
            let retry_after = Duration::from_secs(self.window_secs - elapsed);
            tracing::warn!(
                count = self.count,
                limit = self.limit,
                retry_after_secs = retry_after.as_secs(),
                "rate limit exceeded"
            );
            return Err(ApiError::RateLimit {
                retry_after: Some(retry_after),
            });
        }
        self.count += 1;
        tracing::trace!(count = self.count, limit = self.limit, "rate limit ok");
        Ok(())
    }
}
