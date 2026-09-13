use reqwest::Client;
use sea_orm::DatabaseConnection;

use super::config::EdgarConfig;

#[derive(Clone)]
pub struct EdgarState {
    pub db: DatabaseConnection,
    pub config: EdgarConfig,
    pub client: Client,
}

impl EdgarState {
    pub fn new(db: DatabaseConnection, config: EdgarConfig, client: Client) -> Self {
        Self { db, config, client }
    }
}
