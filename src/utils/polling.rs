use tokio::time::{Duration, Interval};

/// Create a Tokio interval for polling tasks
///
/// # Arguments
///
/// * `secs` - interval duration in seconds
pub fn create_interval(secs: u64) -> Interval {
    tokio::time::interval(Duration::from_secs(secs))
}
