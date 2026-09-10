use chrono::{DateTime, NaiveDate, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "nse_brsr")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub item_id: i64,
    pub original_submission_date: Option<DateTime<Utc>>,
    #[sea_orm(column_type = "Text", nullable)]
    pub nse_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub scrip_code: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub msei_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub isin: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub cin: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub company_name: Option<String>,
    pub date_of_incorporation: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub registered_office: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub corporate_office: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub email: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub telephone: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub website: Option<String>,
    pub fy_start: Option<NaiveDate>,
    pub fy_end: Option<NaiveDate>,
    pub py_start: Option<NaiveDate>,
    pub py_end: Option<NaiveDate>,
    pub ppy_start: Option<NaiveDate>,
    pub ppy_end: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub paid_up_capital: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub contact_person: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub contact_phone: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub contact_email: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub reporting_boundary: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub core_assurance: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub turnover: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub net_worth: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub states_served: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub countries_served: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub board_size: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub female_directors: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub kmp: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub female_kmp: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub csr_applicable: Option<String>,
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
