use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Rate limiter for generating one-time edit subscription links per subscriber.
/// Default: 5 link generations per 5 minutes per subscriber.
#[derive(Clone, Default)]
pub struct SubscriberRateLimiter {
    generations: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

impl SubscriberRateLimiter {
    pub fn new() -> Self {
        Self {
            generations: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check and record an attempt. Returns `true` if within limit, `false` if exceeded.
    pub async fn check_and_record(&self, email: &str, limit: usize, window: Duration) -> bool {
        let key = email.trim().to_ascii_lowercase();
        let mut map = self.generations.lock().await;
        let now = Instant::now();
        let list = map.entry(key).or_default();
        list.retain(|&t| now.duration_since(t) <= window);
        if list.len() >= limit {
            false
        } else {
            list.push(now);
            true
        }
    }

    /// Default check: 5 generations per 5 minutes (300 seconds).
    pub async fn check_link_generation(&self, email: &str) -> bool {
        self.check_and_record(email, 5, Duration::from_secs(300)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_up_to_limit() {
        let limiter = SubscriberRateLimiter::new();
        let email = "subscriber@example.com";
        for _ in 0..5 {
            assert!(limiter.check_and_record(email, 5, Duration::from_secs(10)).await);
        }
        // 6th attempt should fail
        assert!(!limiter.check_and_record(email, 5, Duration::from_secs(10)).await);

        // Different email should succeed
        assert!(limiter.check_and_record("other@example.com", 5, Duration::from_secs(10)).await);
    }
}
