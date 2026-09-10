use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "bse_rss_items")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub feed_kind: String,
    #[sea_orm(column_type = "Text")]
    pub title: String,
    #[sea_orm(column_type = "Text")]
    pub link: String,
    #[sea_orm(column_type = "Text")]
    pub description: String,
    pub pub_date: Option<DateTime<Utc>>,
    #[sea_orm(unique, column_type = "Text")]
    pub content_hash: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::announcements::Entity")]
    Announcements,
    #[sea_orm(has_one = "super::annual_reports::Entity")]
    AnnualReports,
    #[sea_orm(has_one = "super::board_meetings::Entity")]
    BoardMeetings,
    #[sea_orm(has_one = "super::corporate_actions::Entity")]
    CorporateActions,
    #[sea_orm(has_one = "super::financial_results::Entity")]
    FinancialResults,
    #[sea_orm(has_one = "super::insider_trading::Entity")]
    InsiderTrading,
    #[sea_orm(has_one = "super::shareholding_pattern::Entity")]
    ShareholdingPattern,
    #[sea_orm(has_one = "super::voting_results::Entity")]
    VotingResults,
}

impl Related<super::announcements::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Announcements.def()
    }
}
impl Related<super::annual_reports::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AnnualReports.def()
    }
}
impl Related<super::board_meetings::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BoardMeetings.def()
    }
}
impl Related<super::corporate_actions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CorporateActions.def()
    }
}
impl Related<super::financial_results::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FinancialResults.def()
    }
}
impl Related<super::insider_trading::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::InsiderTrading.def()
    }
}
impl Related<super::shareholding_pattern::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ShareholdingPattern.def()
    }
}
impl Related<super::voting_results::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::VotingResults.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
