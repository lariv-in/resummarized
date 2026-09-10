use chrono::NaiveDate;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "nse_statement_of_deviation")]
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
    pub isin: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub company_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub statement_count: Option<String>,
    pub quarter_ended: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub mode_of_fund_raising: Option<String>,
    pub date_of_funds_raising: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub amount_raised: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub monitoring_agency: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub monitoring_agency_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub has_deviation: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub deviation_explanation: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub shareholder_approved: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub audit_committee_comments: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub auditor_comments: Option<String>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub objects: Option<Json>,
    #[sea_orm(column_type = "Text", nullable)]
    pub signatory: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub designation: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub place: Option<String>,
    pub date_of_signing: Option<NaiveDate>,
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
