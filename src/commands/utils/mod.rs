pub mod environment;
mod rate_limit;
mod submit_option;

use std::time::Duration;

pub use rate_limit::RateLimit;
pub use submit_option::SubmitOption;

pub const SPINNER_TICK_RATE: Duration = Duration::from_millis(50);

pub const SPINNER_SLOW_TICK_RATE: Duration = Duration::from_millis(100);
