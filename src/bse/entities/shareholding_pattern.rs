use chrono::NaiveDate;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "bse_shareholding_pattern")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub item_id: i64,
    #[sea_orm(column_type = "Text", nullable)]
    pub scripcode: Option<String>,
    pub as_on_date: Option<NaiveDate>,
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
