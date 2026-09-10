use chrono::NaiveDate;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "nse_insider_trading")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub item_id: i64,
    #[sea_orm(column_type = "Text", nullable)]
    pub nse_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub scrip_code: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub msei_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub isin: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub company_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub regulation: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub instrument: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub person: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub category: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub txn_type: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub qty: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub value: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub mode: Option<String>,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub prior_qty: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub prior_pct: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub post_qty: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub post_pct: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub signatory: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub designation: Option<String>,
    pub filing_date: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub exchange: Option<String>,
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
