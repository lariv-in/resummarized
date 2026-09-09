use reqwest::Client;
use sea_orm::DatabaseConnection;

use super::config::BseConfig;

#[derive(Clone)]
pub struct BseState {
    pub db: DatabaseConnection,
    pub config: BseConfig,
    pub client: Client,
}

impl BseState {
    pub fn new(db: DatabaseConnection, config: BseConfig, client: Client) -> Self {
        Self { db, config, client }
    }
}
