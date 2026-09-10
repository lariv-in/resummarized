use chrono::NaiveDate;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "nse_investor_complaints")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub item_id: i64,
    pub for_quarter_ending: Option<NaiveDate>,
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
    pub class: Option<String>,
    pub period_end: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub submission_type: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub pending_start: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub received: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub disposed: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub pending_end: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub scores_id: Option<String>,
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
