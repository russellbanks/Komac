use chrono::TimeDelta;
use indicatif::ProgressBar;
use std::time::{Duration, Instant};
use tokio::{sync::Mutex, time::sleep};

use crate::commands::utils::SPINNER_SLOW_TICK_RATE;

pub struct RateLimit {
    last_pr_time: Mutex<Instant>,
    rate_limit_delay: Duration,
}

/// GitHub has an undocumented limit of 150 pull requests per hour
///
/// <https://github.com/cli/cli/issues/4801#issuecomment-1430651377>
const MAX_PULL_REQUESTS_PER_HOUR: u8 = 150;

/// Minimum delay to not go above 150 pull requests per hour
const HOURLY_RATE_LIMIT_DELAY: Duration = Duration::from_secs(
    TimeDelta::hours(1).num_seconds().unsigned_abs() / MAX_PULL_REQUESTS_PER_HOUR as u64,
);

/// GitHub has an undocumented limit of 20 pull requests per minute
///
/// <https://github.com/cli/cli/issues/4801#issuecomment-1430651377>
const MAX_PULL_REQUESTS_PER_MINUTE: u8 = 20;

/// Minimum delay to not go above 20 pull requests per minute
const PER_MINUTE_RATE_LIMIT_DELAY: Duration = Duration::from_secs(
    TimeDelta::minutes(1).num_seconds().unsigned_abs() / MAX_PULL_REQUESTS_PER_MINUTE as u64,
);

impl RateLimit {
    pub fn new(fast: bool) -> RateLimit {
        let rate_limit_delay = if fast {
            PER_MINUTE_RATE_LIMIT_DELAY
        } else {
            HOURLY_RATE_LIMIT_DELAY
        };

        RateLimit {
            last_pr_time: Mutex::new(Instant::now().checked_sub(rate_limit_delay).unwrap()),
            rate_limit_delay,
        }
    }

    pub async fn wait(&self) {
        let last_pr_time = self.last_pr_time.lock().await;
        let time_since_last_pr = Instant::now().duration_since(*last_pr_time);

        if time_since_last_pr < self.rate_limit_delay {
            let wait_time = self.rate_limit_delay - time_since_last_pr;
            let wait_pb = ProgressBar::new_spinner()
                .with_message(format!(
                    "Last pull request was created {time_since_last_pr:?} ago. Waiting for {wait_time:?}",
                ));
            wait_pb.enable_steady_tick(SPINNER_SLOW_TICK_RATE);
            sleep(wait_time).await;
            wait_pb.finish_and_clear();
        }
    }

    pub async fn record(&self) {
        let mut last_pr_time = self.last_pr_time.lock().await;
        *last_pr_time = Instant::now();
    }
}
