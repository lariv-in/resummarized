use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "euronext_feed_status")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub feed_kind: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub last_build_date: Option<DateTime<Utc>>,
    #[sea_orm(column_type = "Text", nullable)]
    pub last_error: Option<String>,
    pub last_fetched_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
