use chrono::NaiveDate;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "nse_integrated_filing_financials")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub item_id: i64,
    #[sea_orm(column_type = "Text", nullable)]
    pub submission_type: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub remarks: Option<String>,
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
    pub type_of_company: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub class_of_security: Option<String>,
    pub fy_start: Option<NaiveDate>,
    pub fy_end: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub reporting_period: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub reporting_quarter: Option<String>,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub audited: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub nature: Option<String>,
    pub board_meeting: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub revenue: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub profit: Option<String>,
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
