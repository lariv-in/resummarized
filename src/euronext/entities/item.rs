use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "euronext_rss_items")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub feed_kind: String,
    #[sea_orm(column_type = "Text")]
    pub title: String,
    #[sea_orm(column_type = "Text")]
    pub link: String,
    #[sea_orm(column_type = "Text")]
    pub description: String,
    pub pub_date: Option<DateTime<Utc>>,
    #[sea_orm(column_type = "Text", nullable)]
    pub guid: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub company_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub company_ticker: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub attachment: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub attachment_title: Option<String>,
    #[sea_orm(unique, column_type = "Text")]
    pub content_hash: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
