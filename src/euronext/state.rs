use reqwest::Client;
use sea_orm::DatabaseConnection;

use super::config::EuronextConfig;

#[derive(Clone)]
pub struct EuronextState {
    pub db: DatabaseConnection,
    pub config: EuronextConfig,
    pub client: Client,
}

impl EuronextState {
    pub fn new(db: DatabaseConnection, config: EuronextConfig, client: Client) -> Self {
        Self { db, config, client }
    }
}
