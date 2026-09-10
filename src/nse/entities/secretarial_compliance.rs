use chrono::NaiveDate;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "nse_secretarial_compliance")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub item_id: i64,
    #[sea_orm(column_type = "Text", nullable)]
    pub financial_year: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub submission_type: Option<String>,
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
    pub fy_start: Option<NaiveDate>,
    pub fy_end: Option<NaiveDate>,
    pub date_of_report: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub observations_reported: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub previous_observations: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub actions_taken: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub certifying_firm: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub pcs_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub membership_type: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub membership_number: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub udin: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub cp_number: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub place: Option<String>,
    pub pcs_report_date: Option<NaiveDate>,
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
