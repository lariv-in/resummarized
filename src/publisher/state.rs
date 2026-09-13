use sea_orm::DatabaseConnection;

use super::rate_limit::SubscriberRateLimiter;

/// Runtime state for the Publisher plugin.
#[derive(Clone)]
pub struct PublisherState {
    pub db: DatabaseConnection,
    pub rate_limiter: SubscriberRateLimiter,
}

impl PublisherState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            rate_limiter: SubscriberRateLimiter::new(),
        }
    }
}
