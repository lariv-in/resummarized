use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "nse_rss_items")]
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
    #[sea_orm(has_one = "super::daily_buyback::Entity")]
    DailyBuyback,
    #[sea_orm(has_one = "super::annual_reports::Entity")]
    AnnualReports,
    #[sea_orm(has_one = "super::corporate_actions::Entity")]
    CorporateActions,
    #[sea_orm(has_one = "super::reason_for_encumbrance::Entity")]
    ReasonForEncumbrance,
    #[sea_orm(has_one = "super::regulation_29::Entity")]
    Regulation29,
    #[sea_orm(has_one = "super::regulation_31::Entity")]
    Regulation31,
    #[sea_orm(has_one = "super::share_transfers::Entity")]
    ShareTransfers,
    #[sea_orm(has_one = "super::brsr::Entity")]
    Brsr,
    #[sea_orm(has_one = "super::voting_results::Entity")]
    VotingResults,
    #[sea_orm(has_one = "super::unitholding_patterns::Entity")]
    UnitholdingPatterns,
    #[sea_orm(has_one = "super::statement_of_deviation::Entity")]
    StatementOfDeviation,
    #[sea_orm(has_one = "super::shareholding_pattern::Entity")]
    ShareholdingPattern,
    #[sea_orm(has_one = "super::secretarial_compliance::Entity")]
    SecretarialCompliance,
    #[sea_orm(has_one = "super::related_party_transactions::Entity")]
    RelatedPartyTransactions,
    #[sea_orm(has_one = "super::investor_complaints::Entity")]
    InvestorComplaints,
    #[sea_orm(has_one = "super::insider_trading::Entity")]
    InsiderTrading,
    #[sea_orm(has_one = "super::integrated_filing_financials::Entity")]
    IntegratedFilingFinancials,
    #[sea_orm(has_one = "super::financial_results::Entity")]
    FinancialResults,
}

impl Related<super::announcements::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Announcements.def()
    }
}
impl Related<super::daily_buyback::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DailyBuyback.def()
    }
}
impl Related<super::annual_reports::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AnnualReports.def()
    }
}
impl Related<super::corporate_actions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CorporateActions.def()
    }
}
impl Related<super::reason_for_encumbrance::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ReasonForEncumbrance.def()
    }
}
impl Related<super::regulation_29::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Regulation29.def()
    }
}
impl Related<super::regulation_31::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Regulation31.def()
    }
}
impl Related<super::share_transfers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ShareTransfers.def()
    }
}
impl Related<super::brsr::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Brsr.def()
    }
}
impl Related<super::voting_results::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::VotingResults.def()
    }
}
impl Related<super::unitholding_patterns::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UnitholdingPatterns.def()
    }
}
impl Related<super::statement_of_deviation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::StatementOfDeviation.def()
    }
}
impl Related<super::shareholding_pattern::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ShareholdingPattern.def()
    }
}
impl Related<super::secretarial_compliance::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SecretarialCompliance.def()
    }
}
impl Related<super::related_party_transactions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RelatedPartyTransactions.def()
    }
}
impl Related<super::investor_complaints::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::InvestorComplaints.def()
    }
}
impl Related<super::insider_trading::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::InsiderTrading.def()
    }
}
impl Related<super::integrated_filing_financials::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::IntegratedFilingFinancials.def()
    }
}
impl Related<super::financial_results::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FinancialResults.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
