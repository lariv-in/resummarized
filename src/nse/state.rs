use reqwest::Client;
use sea_orm::DatabaseConnection;

use super::config::NseConfig;

#[derive(Clone)]
pub struct NseState {
    pub db: DatabaseConnection,
    pub config: NseConfig,
    pub client: Client,
}

impl NseState {
    pub fn new(db: DatabaseConnection, config: NseConfig, client: Client) -> Self {
        Self { db, config, client }
    }
}
