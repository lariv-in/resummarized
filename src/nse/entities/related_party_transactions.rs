use chrono::NaiveDate;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "nse_related_party_transactions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub item_id: i64,
    pub period_end_date: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub nse_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub scrip_code: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub msei_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub company_name: Option<String>,
    pub fy_start: Option<NaiveDate>,
    pub fy_end: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub reporting_period: Option<String>,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub has_related_party: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub entered_transactions: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub transaction_count: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub counterparty: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub transaction_type: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub amount: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::item::Entity",
        from = "Column::ItemId",
        to = "super::item::Column::Id"
    )]
    Item,
}

impl Related<super::item::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Item.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
