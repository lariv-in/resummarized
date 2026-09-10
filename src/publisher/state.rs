use sea_orm::DatabaseConnection;

/// Runtime state for the Publisher plugin.
#[derive(Clone)]
pub struct PublisherState {
    pub db: DatabaseConnection,
}

impl PublisherState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
