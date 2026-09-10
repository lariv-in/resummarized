use chrono::NaiveDate;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "nse_unitholding_patterns")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub item_id: i64,
    pub as_on_date: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub scrip_code: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub nse_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub msei_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub sebi_registration: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub company_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub type_of_report: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub number_of_securities: Option<String>,
    pub reporting_period_start: Option<NaiveDate>,
    pub date_of_report: Option<NaiveDate>,
    pub fy_start: Option<NaiveDate>,
    pub fy_end: Option<NaiveDate>,
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
