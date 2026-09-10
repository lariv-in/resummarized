use chrono::NaiveDate;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "nse_voting_results")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub item_id: i64,
    pub meeting_date: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub scrip_code: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub msei_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub isin: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub company_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub type_of_meeting: Option<String>,
    pub date_of_meeting: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub start_time: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub end_time: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub scrutinizer: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub scrutinizer_firm: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub scrutinizer_qualification: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub scrutinizer_membership: Option<String>,
    pub board_meeting_date: Option<NaiveDate>,
    pub report_issuance_date: Option<NaiveDate>,
    pub record_date: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub shareholders_on_record: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub promoters_in_person: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub public_in_person: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub promoters_vc: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub public_vc: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub resolutions_passed: Option<String>,
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
