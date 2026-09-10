use chrono::NaiveDate;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "bse_corporate_actions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub item_id: i64,
    #[sea_orm(column_type = "Text", nullable)]
    pub scripcode: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub segment: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub purpose: Option<String>,
    pub rd_date: Option<NaiveDate>,
    pub bc_start_date: Option<NaiveDate>,
    pub bc_end_date: Option<NaiveDate>,
    pub nd_start_date: Option<NaiveDate>,
    pub nd_end_date: Option<NaiveDate>,
    pub actual_payment_date: Option<NaiveDate>,
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
