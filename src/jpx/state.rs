use reqwest::Client;
use sea_orm::DatabaseConnection;

use super::config::JpxConfig;

#[derive(Clone)]
pub struct JpxState {
    pub db: DatabaseConnection,
    pub config: JpxConfig,
    pub client: Client,
}

impl JpxState {
    pub fn new(db: DatabaseConnection, config: JpxConfig, client: Client) -> Self {
        Self { db, config, client }
    }
}
