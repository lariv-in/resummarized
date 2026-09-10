use chrono::NaiveDate;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "bse_financial_results")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub item_id: i64,
    #[sea_orm(column_type = "Text", nullable)]
    pub scripcode: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub audited_unaudited: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub standalone_consolidated: Option<String>,
    pub period_start_date: Option<NaiveDate>,
    pub period_end_date: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub ind_as: Option<String>,
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
