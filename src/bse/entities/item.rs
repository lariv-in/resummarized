use chrono::{DateTime, NaiveDate, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "bse_rss_items")]
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
    pub scripcode: Option<String>,
    pub as_on_date: Option<NaiveDate>,
    pub meeting_date: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub meeting_type: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub purpose: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub segment: Option<String>,
    pub rd_date: Option<NaiveDate>,
    pub bc_start_date: Option<NaiveDate>,
    pub bc_end_date: Option<NaiveDate>,
    pub nd_start_date: Option<NaiveDate>,
    pub nd_end_date: Option<NaiveDate>,
    pub actual_payment_date: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub type_of_security: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub audited_unaudited: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub standalone_consolidated: Option<String>,
    pub period_start_date: Option<NaiveDate>,
    pub period_end_date: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub ind_as: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub promoter_and_group: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub public_val: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub emptr: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub status: Option<String>,
    pub submission_date: Option<NaiveDate>,
    pub revised_filing_date: Option<NaiveDate>,
    #[sea_orm(unique, column_type = "Text")]
    pub content_hash: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
