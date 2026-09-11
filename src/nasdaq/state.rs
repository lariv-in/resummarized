use reqwest::Client;
use sea_orm::DatabaseConnection;

use super::config::NasdaqConfig;

#[derive(Clone)]
pub struct NasdaqState {
    pub db: DatabaseConnection,
    pub config: NasdaqConfig,
    pub client: Client,
}

impl NasdaqState {
    pub fn new(db: DatabaseConnection, config: NasdaqConfig, client: Client) -> Self {
        Self { db, config, client }
    }
}
