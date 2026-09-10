//! Per-feed satellite load, upsert, sort, display, and search.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, NaiveDate, Utc};
use lariv_rs::db::trigram;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, JoinType,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait, Select,
};
use serde::Serialize;
use serde_json::Value as JsonValue;

use super::brsr::{self, BrsrField};
use super::description::{DescriptionField, NseDescriptionFields};
use super::entities::{
    announcements, annual_reports, brsr as brsr_ent, corporate_actions, daily_buyback,
    financial_results, insider_trading, integrated_filing_financials, investor_complaints, item,
    reason_for_encumbrance, regulation_29, regulation_31, related_party_transactions,
    secretarial_compliance, share_transfers, shareholding_pattern, statement_of_deviation,
    unitholding_patterns, voting_results,
};
use super::feeds::NseFeedKind;
use super::fr::{self, FrField};
use super::ic::{self, IcField};
use super::iff::{self, IffField};
use super::it::{self, ItField};
use super::rpt::{self, RptField};
use super::scr::{self, ScrField};
use super::shp::{self, ShpField};
use super::sod::{self, SodField, SodObjectRow};
use super::uhp::{self, UhpField};
use super::voting::{self, VoteField};
use super::xbrl::{set_opt, set_opt_date};

pub enum LinkedFacts {
    Brsr(brsr::BrsrFacts),
    Vote(voting::VoteFacts),
    Uhp(uhp::UhpFacts),
    Sod(sod::SodFacts),
    Shp(shp::ShpFacts),
    Scr(scr::ScrFacts),
    Rpt(rpt::RptFacts),
    Ic(ic::IcFacts),
    It(it::ItFacts),
    Iff(iff::IffFacts),
    Fr(fr::FrFacts),
}

#[derive(Clone, Debug)]
pub enum ExtraRow {
    Announcements(announcements::Model),
    DailyBuyback(daily_buyback::Model),
    AnnualReports(annual_reports::Model),
    CorporateActions(corporate_actions::Model),
    ReasonForEncumbrance(reason_for_encumbrance::Model),
    Regulation29(regulation_29::Model),
    Regulation31(regulation_31::Model),
    ShareTransfers(share_transfers::Model),
    Brsr(brsr_ent::Model),
    VotingResults(voting_results::Model),
    UnitholdingPatterns(unitholding_patterns::Model),
    StatementOfDeviation(statement_of_deviation::Model),
    ShareholdingPattern(shareholding_pattern::Model),
    SecretarialCompliance(secretarial_compliance::Model),
    RelatedPartyTransactions(related_party_transactions::Model),
    InvestorComplaints(investor_complaints::Model),
    InsiderTrading(insider_trading::Model),
    IntegratedFilingFinancials(integrated_filing_financials::Model),
    FinancialResults(financial_results::Model),
}

macro_rules! persist {
    ($entity:ident, $db:expr, $item_id:expr, $am:ident, $body:block) => {{
        let existing = $entity::Entity::find_by_id($item_id).one($db).await?;
        let is_new = existing.is_none();
        let mut $am: $entity::ActiveModel = match existing {
            Some(row) => row.into(),
            None => $entity::ActiveModel {
                item_id: Set($item_id),
                ..Default::default()
            },
        };
        $body
        if is_new {
            $am.insert($db).await?;
        } else {
            $am.update($db).await?;
        }
    }};
}

macro_rules! load_kind {
    ($map:expr, $entity:ident, $variant:ident, $db:expr, $ids:expr) => {{
        for row in $entity::Entity::find()
            .filter($entity::Column::ItemId.is_in($ids.to_vec()))
            .all($db)
            .await?
        {
            $map.insert(row.item_id, ExtraRow::$variant(row));
        }
    }};
}

fn set_dt(slot: &mut sea_orm::ActiveValue<Option<DateTime<Utc>>>, value: Option<DateTime<Utc>>) {
    if let Some(v) = value {
        *slot = Set(Some(v));
    }
}

fn set_date(slot: &mut sea_orm::ActiveValue<Option<NaiveDate>>, value: Option<NaiveDate>) {
    set_opt_date(slot, value);
}

pub fn sort_direction(sort: &str, key: &str) -> Option<bool> {
    let sort = sort.trim();
    let desc = format!("{key} DESC");
    let asc = format!("{key} ASC");
    if sort.eq_ignore_ascii_case(&desc) {
        Some(true)
    } else if sort.eq_ignore_ascii_case(&asc) || sort.eq_ignore_ascii_case(key) {
        Some(false)
    } else {
        None
    }
}

pub async fn load_map(
    db: &DatabaseConnection,
    kind: NseFeedKind,
    ids: &[i64],
) -> Result<HashMap<i64, ExtraRow>, sea_orm::DbErr> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    let mut map = HashMap::new();
    match kind {
        NseFeedKind::Announcements => load_kind!(map, announcements, Announcements, db, ids),
        NseFeedKind::DailyBuyback => load_kind!(map, daily_buyback, DailyBuyback, db, ids),
        NseFeedKind::AnnualReports => load_kind!(map, annual_reports, AnnualReports, db, ids),
        NseFeedKind::CorporateActions => {
            load_kind!(map, corporate_actions, CorporateActions, db, ids)
        }
        NseFeedKind::ReasonForEncumbrance => {
            load_kind!(map, reason_for_encumbrance, ReasonForEncumbrance, db, ids)
        }
        NseFeedKind::Regulation29 => load_kind!(map, regulation_29, Regulation29, db, ids),
        NseFeedKind::Regulation31 => load_kind!(map, regulation_31, Regulation31, db, ids),
        NseFeedKind::ShareTransfers => load_kind!(map, share_transfers, ShareTransfers, db, ids),
        NseFeedKind::Brsr => load_kind!(map, brsr_ent, Brsr, db, ids),
        NseFeedKind::VotingResults => load_kind!(map, voting_results, VotingResults, db, ids),
        NseFeedKind::UnitholdingPatterns => {
            load_kind!(map, unitholding_patterns, UnitholdingPatterns, db, ids)
        }
        NseFeedKind::StatementOfDeviation => {
            load_kind!(map, statement_of_deviation, StatementOfDeviation, db, ids)
        }
        NseFeedKind::ShareholdingPattern => {
            load_kind!(map, shareholding_pattern, ShareholdingPattern, db, ids)
        }
        NseFeedKind::SecretarialCompliance => {
            load_kind!(map, secretarial_compliance, SecretarialCompliance, db, ids)
        }
        NseFeedKind::RelatedPartyTransactions => {
            load_kind!(
                map,
                related_party_transactions,
                RelatedPartyTransactions,
                db,
                ids
            )
        }
        NseFeedKind::InvestorComplaints => {
            load_kind!(map, investor_complaints, InvestorComplaints, db, ids)
        }
        NseFeedKind::InsiderTrading => load_kind!(map, insider_trading, InsiderTrading, db, ids),
        NseFeedKind::IntegratedFilingFinancials => {
            load_kind!(
                map,
                integrated_filing_financials,
                IntegratedFilingFinancials,
                db,
                ids
            )
        }
        NseFeedKind::FinancialResults => {
            load_kind!(map, financial_results, FinancialResults, db, ids)
        }
        NseFeedKind::BoardMeetings
        | NseFeedKind::CorporateGovernance
        | NseFeedKind::OfferDocuments
        | NseFeedKind::Circulars => {}
    }
    Ok(map)
}

pub fn needs_linked_facts(kind: NseFeedKind, extra: Option<&ExtraRow>) -> bool {
    match kind {
        NseFeedKind::Brsr => match extra {
            Some(ExtraRow::Brsr(m)) => m.nse_symbol.is_none(),
            _ => true,
        },
        NseFeedKind::VotingResults => match extra {
            Some(ExtraRow::VotingResults(m)) => m.symbol.is_none(),
            _ => true,
        },
        NseFeedKind::UnitholdingPatterns => match extra {
            Some(ExtraRow::UnitholdingPatterns(m)) => m.nse_symbol.is_none(),
            _ => true,
        },
        NseFeedKind::StatementOfDeviation => match extra {
            Some(ExtraRow::StatementOfDeviation(m)) => {
                m.nse_symbol.is_none() || m.objects.is_none()
            }
            _ => true,
        },
        NseFeedKind::ShareholdingPattern => match extra {
            Some(ExtraRow::ShareholdingPattern(m)) => m.nse_symbol.is_none(),
            _ => true,
        },
        NseFeedKind::SecretarialCompliance => match extra {
            Some(ExtraRow::SecretarialCompliance(m)) => m.nse_symbol.is_none(),
            _ => true,
        },
        NseFeedKind::RelatedPartyTransactions => match extra {
            Some(ExtraRow::RelatedPartyTransactions(m)) => m.nse_symbol.is_none(),
            _ => true,
        },
        NseFeedKind::InvestorComplaints => match extra {
            Some(ExtraRow::InvestorComplaints(m)) => m.nse_symbol.is_none(),
            _ => true,
        },
        NseFeedKind::InsiderTrading => match extra {
            Some(ExtraRow::InsiderTrading(m)) => m.nse_symbol.is_none(),
            _ => true,
        },
        NseFeedKind::IntegratedFilingFinancials => match extra {
            Some(ExtraRow::IntegratedFilingFinancials(m)) => m.nse_symbol.is_none(),
            _ => true,
        },
        NseFeedKind::FinancialResults => match extra {
            Some(ExtraRow::FinancialResults(m)) => m.nse_symbol.is_none(),
            _ => true,
        },
        _ => false,
    }
}

pub async fn upsert(
    db: &DatabaseConnection,
    kind: NseFeedKind,
    item_id: i64,
    parsed: &NseDescriptionFields,
    facts: Option<&LinkedFacts>,
) -> Result<(), sea_orm::DbErr> {
    match kind {
        NseFeedKind::Announcements => persist!(announcements, db, item_id, am, {
            set_opt(&mut am.subject, &parsed.subject);
        }),
        NseFeedKind::DailyBuyback => persist!(daily_buyback, db, item_id, am, {
            set_opt(&mut am.subject, &parsed.subject);
        }),
        NseFeedKind::AnnualReports => persist!(annual_reports, db, item_id, am, {
            set_date(&mut am.as_on_date, parsed.as_on_date);
        }),
        NseFeedKind::CorporateActions => persist!(corporate_actions, db, item_id, am, {
            set_opt(&mut am.series, &parsed.series);
            set_opt(&mut am.purpose, &parsed.purpose);
            set_opt(&mut am.face_value, &parsed.face_value);
            set_date(&mut am.record_date, parsed.record_date);
            set_date(
                &mut am.book_closure_start_date,
                parsed.book_closure_start_date,
            );
            set_date(&mut am.book_closure_end_date, parsed.book_closure_end_date);
        }),
        NseFeedKind::ReasonForEncumbrance => persist!(reason_for_encumbrance, db, item_id, am, {
            set_opt(
                &mut am.encumbered_promoter_names,
                &parsed.encumbered_promoter_names,
            );
        }),
        NseFeedKind::Regulation29 => persist!(regulation_29, db, item_id, am, {
            set_opt(&mut am.acquirer_names, &parsed.acquirer_names);
        }),
        NseFeedKind::Regulation31 => persist!(regulation_31, db, item_id, am, {
            set_opt(&mut am.promoter_names, &parsed.promoter_names);
        }),
        NseFeedKind::ShareTransfers => persist!(share_transfers, db, item_id, am, {
            set_date(&mut am.period_ended, parsed.period_ended);
        }),
        NseFeedKind::Brsr => persist!(brsr_ent, db, item_id, am, {
            set_dt(
                &mut am.original_submission_date,
                parsed.original_submission_date,
            );
            if let Some(LinkedFacts::Brsr(f)) = facts {
                f.apply(&mut am);
            }
        }),
        NseFeedKind::VotingResults => persist!(voting_results, db, item_id, am, {
            set_date(&mut am.meeting_date, parsed.meeting_date);
            if let Some(LinkedFacts::Vote(f)) = facts {
                f.apply(&mut am);
            }
        }),
        NseFeedKind::UnitholdingPatterns => persist!(unitholding_patterns, db, item_id, am, {
            set_date(&mut am.as_on_date, parsed.as_on_date);
            if let Some(LinkedFacts::Uhp(f)) = facts {
                f.apply(&mut am);
            }
        }),
        NseFeedKind::StatementOfDeviation => persist!(statement_of_deviation, db, item_id, am, {
            set_date(&mut am.period_end_date, parsed.period_end_date);
            if let Some(LinkedFacts::Sod(f)) = facts {
                f.apply(&mut am);
            }
        }),
        NseFeedKind::ShareholdingPattern => persist!(shareholding_pattern, db, item_id, am, {
            if let Some(LinkedFacts::Shp(f)) = facts {
                f.apply(&mut am);
            }
        }),
        NseFeedKind::SecretarialCompliance => persist!(secretarial_compliance, db, item_id, am, {
            set_opt(&mut am.financial_year, &parsed.financial_year);
            set_opt(&mut am.submission_type, &parsed.submission_type);
            if let Some(LinkedFacts::Scr(f)) = facts {
                f.apply(&mut am);
            }
        }),
        NseFeedKind::RelatedPartyTransactions => {
            persist!(related_party_transactions, db, item_id, am, {
                set_date(&mut am.period_end_date, parsed.period_end_date);
                if let Some(LinkedFacts::Rpt(f)) = facts {
                    f.apply(&mut am);
                }
            })
        }
        NseFeedKind::InvestorComplaints => persist!(investor_complaints, db, item_id, am, {
            set_date(&mut am.for_quarter_ending, parsed.for_quarter_ending);
            if let Some(LinkedFacts::Ic(f)) = facts {
                f.apply(&mut am);
            }
        }),
        NseFeedKind::InsiderTrading => persist!(insider_trading, db, item_id, am, {
            if let Some(LinkedFacts::It(f)) = facts {
                f.apply(&mut am);
            }
        }),
        NseFeedKind::IntegratedFilingFinancials => {
            persist!(integrated_filing_financials, db, item_id, am, {
                set_opt(&mut am.submission_type, &parsed.submission_type);
                set_opt(&mut am.remarks, &parsed.remarks);
                if let Some(LinkedFacts::Iff(f)) = facts {
                    f.apply(&mut am);
                }
            })
        }
        NseFeedKind::FinancialResults => persist!(financial_results, db, item_id, am, {
            set_opt(&mut am.relating_to, &parsed.relating_to);
            set_opt(&mut am.audited_unaudited, &parsed.audited_unaudited);
            set_opt(&mut am.cumulative, &parsed.cumulative);
            set_opt(&mut am.consolidated, &parsed.consolidated);
            set_opt(&mut am.ind_as, &parsed.ind_as);
            set_opt(&mut am.period, &parsed.period);
            set_date(&mut am.period_ended, parsed.period_ended);
            if let Some(LinkedFacts::Fr(f)) = facts {
                f.apply(&mut am);
            }
        }),
        NseFeedKind::BoardMeetings
        | NseFeedKind::CorporateGovernance
        | NseFeedKind::OfferDocuments
        | NseFeedKind::Circulars => {}
    }
    Ok(())
}

fn order<C: ColumnTrait>(query: Select<item::Entity>, col: C, desc: bool) -> Select<item::Entity> {
    if desc {
        query.order_by_desc(col)
    } else {
        query.order_by_asc(col)
    }
}

fn join_order<C: ColumnTrait>(
    query: Select<item::Entity>,
    rel: sea_orm::RelationDef,
    col: C,
    desc: bool,
) -> Select<item::Entity> {
    order(query.join(JoinType::LeftJoin, rel), col, desc)
}

pub fn apply_sort(
    query: Select<item::Entity>,
    kind: NseFeedKind,
    sort: &str,
) -> Select<item::Entity> {
    if let Some(desc) = sort_direction(sort, "Title") {
        return order(query, item::Column::Title, desc);
    }
    if let Some(desc) = sort_direction(sort, "PubDate") {
        return order(query, item::Column::PubDate, desc);
    }
    if let Some((desc, field)) = kind
        .extra_list_fields()
        .iter()
        .find_map(|f| sort_direction(sort, f.sort_key()).map(|d| (d, *f)))
        && let Some(sorted) = desc_sort(query.clone(), kind, field, desc)
    {
        return sorted;
    }
    match kind {
        NseFeedKind::Brsr => xbrl_sort(query, sort, BrsrField::LIST, item::Relation::Brsr.def()),
        NseFeedKind::VotingResults => xbrl_sort(
            query,
            sort,
            VoteField::LIST,
            item::Relation::VotingResults.def(),
        ),
        NseFeedKind::UnitholdingPatterns => xbrl_sort(
            query,
            sort,
            UhpField::LIST,
            item::Relation::UnitholdingPatterns.def(),
        ),
        NseFeedKind::StatementOfDeviation => xbrl_sort(
            query,
            sort,
            SodField::LIST,
            item::Relation::StatementOfDeviation.def(),
        ),
        NseFeedKind::ShareholdingPattern => xbrl_sort(
            query,
            sort,
            ShpField::LIST,
            item::Relation::ShareholdingPattern.def(),
        ),
        NseFeedKind::SecretarialCompliance => xbrl_sort(
            query,
            sort,
            ScrField::LIST,
            item::Relation::SecretarialCompliance.def(),
        ),
        NseFeedKind::RelatedPartyTransactions => xbrl_sort(
            query,
            sort,
            RptField::LIST,
            item::Relation::RelatedPartyTransactions.def(),
        ),
        NseFeedKind::InvestorComplaints => xbrl_sort(
            query,
            sort,
            IcField::LIST,
            item::Relation::InvestorComplaints.def(),
        ),
        NseFeedKind::InsiderTrading => xbrl_sort(
            query,
            sort,
            ItField::LIST,
            item::Relation::InsiderTrading.def(),
        ),
        NseFeedKind::IntegratedFilingFinancials => xbrl_sort(
            query,
            sort,
            IffField::LIST,
            item::Relation::IntegratedFilingFinancials.def(),
        ),
        NseFeedKind::FinancialResults => xbrl_sort(
            query,
            sort,
            FrField::LIST,
            item::Relation::FinancialResults.def(),
        ),
        _ => query.order_by_desc(item::Column::Id),
    }
}

fn xbrl_sort<F, C>(
    query: Select<item::Entity>,
    sort: &str,
    fields: &[F],
    rel: sea_orm::RelationDef,
) -> Select<item::Entity>
where
    F: Copy,
    F: FnIsField<C>,
    C: ColumnTrait,
{
    if let Some((desc, field)) = fields
        .iter()
        .find_map(|f| sort_direction(sort, f.sort_key()).map(|d| (d, *f)))
    {
        join_order(query, rel, field.column(), desc)
    } else {
        query.order_by_desc(item::Column::Id)
    }
}

trait FnIsField<C: ColumnTrait>: Copy {
    fn sort_key(self) -> &'static str;
    fn column(self) -> C;
}

macro_rules! impl_field {
    ($ty:ty, $col:ty) => {
        impl FnIsField<$col> for $ty {
            fn sort_key(self) -> &'static str {
                Self::sort_key(self)
            }
            fn column(self) -> $col {
                Self::column(self)
            }
        }
    };
}

impl_field!(BrsrField, brsr_ent::Column);
impl_field!(VoteField, voting_results::Column);
impl_field!(UhpField, unitholding_patterns::Column);
impl_field!(SodField, statement_of_deviation::Column);
impl_field!(ShpField, shareholding_pattern::Column);
impl_field!(ScrField, secretarial_compliance::Column);
impl_field!(RptField, related_party_transactions::Column);
impl_field!(IcField, investor_complaints::Column);
impl_field!(ItField, insider_trading::Column);
impl_field!(IffField, integrated_filing_financials::Column);
impl_field!(FrField, financial_results::Column);

fn desc_sort(
    query: Select<item::Entity>,
    kind: NseFeedKind,
    field: DescriptionField,
    desc: bool,
) -> Option<Select<item::Entity>> {
    Some(match (kind, field) {
        (NseFeedKind::Announcements, DescriptionField::Subject) => join_order(
            query,
            item::Relation::Announcements.def(),
            announcements::Column::Subject,
            desc,
        ),
        (NseFeedKind::DailyBuyback, DescriptionField::Subject) => join_order(
            query,
            item::Relation::DailyBuyback.def(),
            daily_buyback::Column::Subject,
            desc,
        ),
        (NseFeedKind::AnnualReports, DescriptionField::AsOnDate) => join_order(
            query,
            item::Relation::AnnualReports.def(),
            annual_reports::Column::AsOnDate,
            desc,
        ),
        (NseFeedKind::UnitholdingPatterns, DescriptionField::AsOnDate) => join_order(
            query,
            item::Relation::UnitholdingPatterns.def(),
            unitholding_patterns::Column::AsOnDate,
            desc,
        ),
        (NseFeedKind::Brsr, DescriptionField::OriginalSubmissionDate) => join_order(
            query,
            item::Relation::Brsr.def(),
            brsr_ent::Column::OriginalSubmissionDate,
            desc,
        ),
        (NseFeedKind::CorporateActions, DescriptionField::Series) => join_order(
            query,
            item::Relation::CorporateActions.def(),
            corporate_actions::Column::Series,
            desc,
        ),
        (NseFeedKind::CorporateActions, DescriptionField::Purpose) => join_order(
            query,
            item::Relation::CorporateActions.def(),
            corporate_actions::Column::Purpose,
            desc,
        ),
        (NseFeedKind::CorporateActions, DescriptionField::FaceValue) => join_order(
            query,
            item::Relation::CorporateActions.def(),
            corporate_actions::Column::FaceValue,
            desc,
        ),
        (NseFeedKind::CorporateActions, DescriptionField::RecordDate) => join_order(
            query,
            item::Relation::CorporateActions.def(),
            corporate_actions::Column::RecordDate,
            desc,
        ),
        (NseFeedKind::CorporateActions, DescriptionField::BookClosureStartDate) => join_order(
            query,
            item::Relation::CorporateActions.def(),
            corporate_actions::Column::BookClosureStartDate,
            desc,
        ),
        (NseFeedKind::CorporateActions, DescriptionField::BookClosureEndDate) => join_order(
            query,
            item::Relation::CorporateActions.def(),
            corporate_actions::Column::BookClosureEndDate,
            desc,
        ),
        (NseFeedKind::FinancialResults, DescriptionField::RelatingTo) => join_order(
            query,
            item::Relation::FinancialResults.def(),
            financial_results::Column::RelatingTo,
            desc,
        ),
        (NseFeedKind::FinancialResults, DescriptionField::AuditedUnaudited) => join_order(
            query,
            item::Relation::FinancialResults.def(),
            financial_results::Column::AuditedUnaudited,
            desc,
        ),
        (NseFeedKind::FinancialResults, DescriptionField::Cumulative) => join_order(
            query,
            item::Relation::FinancialResults.def(),
            financial_results::Column::Cumulative,
            desc,
        ),
        (NseFeedKind::FinancialResults, DescriptionField::Consolidated) => join_order(
            query,
            item::Relation::FinancialResults.def(),
            financial_results::Column::Consolidated,
            desc,
        ),
        (NseFeedKind::FinancialResults, DescriptionField::IndAs) => join_order(
            query,
            item::Relation::FinancialResults.def(),
            financial_results::Column::IndAs,
            desc,
        ),
        (NseFeedKind::FinancialResults, DescriptionField::Period) => join_order(
            query,
            item::Relation::FinancialResults.def(),
            financial_results::Column::Period,
            desc,
        ),
        (NseFeedKind::FinancialResults, DescriptionField::PeriodEnded) => join_order(
            query,
            item::Relation::FinancialResults.def(),
            financial_results::Column::PeriodEnded,
            desc,
        ),
        (NseFeedKind::ShareTransfers, DescriptionField::PeriodEnded) => join_order(
            query,
            item::Relation::ShareTransfers.def(),
            share_transfers::Column::PeriodEnded,
            desc,
        ),
        (NseFeedKind::IntegratedFilingFinancials, DescriptionField::SubmissionType) => join_order(
            query,
            item::Relation::IntegratedFilingFinancials.def(),
            integrated_filing_financials::Column::SubmissionType,
            desc,
        ),
        (NseFeedKind::IntegratedFilingFinancials, DescriptionField::Remarks) => join_order(
            query,
            item::Relation::IntegratedFilingFinancials.def(),
            integrated_filing_financials::Column::Remarks,
            desc,
        ),
        (NseFeedKind::InvestorComplaints, DescriptionField::ForQuarterEnding) => join_order(
            query,
            item::Relation::InvestorComplaints.def(),
            investor_complaints::Column::ForQuarterEnding,
            desc,
        ),
        (NseFeedKind::ReasonForEncumbrance, DescriptionField::EncumberedPromoterNames) => {
            join_order(
                query,
                item::Relation::ReasonForEncumbrance.def(),
                reason_for_encumbrance::Column::EncumberedPromoterNames,
                desc,
            )
        }
        (NseFeedKind::Regulation29, DescriptionField::AcquirerNames) => join_order(
            query,
            item::Relation::Regulation29.def(),
            regulation_29::Column::AcquirerNames,
            desc,
        ),
        (NseFeedKind::Regulation31, DescriptionField::PromoterNames) => join_order(
            query,
            item::Relation::Regulation31.def(),
            regulation_31::Column::PromoterNames,
            desc,
        ),
        (NseFeedKind::RelatedPartyTransactions, DescriptionField::PeriodEndDate) => join_order(
            query,
            item::Relation::RelatedPartyTransactions.def(),
            related_party_transactions::Column::PeriodEndDate,
            desc,
        ),
        (NseFeedKind::StatementOfDeviation, DescriptionField::PeriodEndDate) => join_order(
            query,
            item::Relation::StatementOfDeviation.def(),
            statement_of_deviation::Column::PeriodEndDate,
            desc,
        ),
        (NseFeedKind::SecretarialCompliance, DescriptionField::FinancialYear) => join_order(
            query,
            item::Relation::SecretarialCompliance.def(),
            secretarial_compliance::Column::FinancialYear,
            desc,
        ),
        (NseFeedKind::SecretarialCompliance, DescriptionField::SubmissionType) => join_order(
            query,
            item::Relation::SecretarialCompliance.def(),
            secretarial_compliance::Column::SubmissionType,
            desc,
        ),
        (NseFeedKind::VotingResults, DescriptionField::MeetingDate) => join_order(
            query,
            item::Relation::VotingResults.def(),
            voting_results::Column::MeetingDate,
            desc,
        ),
        _ => return None,
    })
}

pub fn extra_columns(kind: NseFeedKind) -> Vec<(&'static str, &'static str)> {
    let mut cols: Vec<_> = kind
        .extra_list_fields()
        .iter()
        .map(|f| (f.sort_key(), f.label()))
        .collect();
    cols.extend(xbrl_list_columns(kind));
    cols
}

fn xbrl_list_columns(kind: NseFeedKind) -> Vec<(&'static str, &'static str)> {
    match kind {
        NseFeedKind::Brsr => BrsrField::LIST
            .iter()
            .map(|f| (f.sort_key(), f.label()))
            .collect(),
        NseFeedKind::VotingResults => VoteField::LIST
            .iter()
            .map(|f| (f.sort_key(), f.label()))
            .collect(),
        NseFeedKind::UnitholdingPatterns => UhpField::LIST
            .iter()
            .map(|f| (f.sort_key(), f.label()))
            .collect(),
        NseFeedKind::StatementOfDeviation => SodField::LIST
            .iter()
            .map(|f| (f.sort_key(), f.label()))
            .collect(),
        NseFeedKind::ShareholdingPattern => ShpField::LIST
            .iter()
            .map(|f| (f.sort_key(), f.label()))
            .collect(),
        NseFeedKind::SecretarialCompliance => ScrField::LIST
            .iter()
            .map(|f| (f.sort_key(), f.label()))
            .collect(),
        NseFeedKind::RelatedPartyTransactions => RptField::LIST
            .iter()
            .map(|f| (f.sort_key(), f.label()))
            .collect(),
        NseFeedKind::InvestorComplaints => IcField::LIST
            .iter()
            .map(|f| (f.sort_key(), f.label()))
            .collect(),
        NseFeedKind::InsiderTrading => ItField::LIST
            .iter()
            .map(|f| (f.sort_key(), f.label()))
            .collect(),
        NseFeedKind::IntegratedFilingFinancials => IffField::LIST
            .iter()
            .map(|f| (f.sort_key(), f.label()))
            .collect(),
        NseFeedKind::FinancialResults => FrField::LIST
            .iter()
            .map(|f| (f.sort_key(), f.label()))
            .collect(),
        _ => Vec::new(),
    }
}

pub fn extra_cells(kind: NseFeedKind, extra: Option<&ExtraRow>, tz: &str) -> Vec<String> {
    let desc = extra.map(ExtraRow::description_fields).unwrap_or_default();
    let mut cells: Vec<_> = kind
        .extra_list_fields()
        .iter()
        .map(|f| f.display(&desc, tz))
        .collect();
    cells.extend(xbrl_list_cells(kind, extra));
    cells
}

fn xbrl_list_cells(kind: NseFeedKind, extra: Option<&ExtraRow>) -> Vec<String> {
    match (kind, extra) {
        (NseFeedKind::Brsr, Some(ExtraRow::Brsr(m))) => {
            BrsrField::LIST.iter().map(|f| f.display(m)).collect()
        }
        (NseFeedKind::VotingResults, Some(ExtraRow::VotingResults(m))) => {
            VoteField::LIST.iter().map(|f| f.display(m)).collect()
        }
        (NseFeedKind::UnitholdingPatterns, Some(ExtraRow::UnitholdingPatterns(m))) => {
            UhpField::LIST.iter().map(|f| f.display(m)).collect()
        }
        (NseFeedKind::StatementOfDeviation, Some(ExtraRow::StatementOfDeviation(m))) => {
            SodField::LIST.iter().map(|f| f.display(m)).collect()
        }
        (NseFeedKind::ShareholdingPattern, Some(ExtraRow::ShareholdingPattern(m))) => {
            ShpField::LIST.iter().map(|f| f.display(m)).collect()
        }
        (NseFeedKind::SecretarialCompliance, Some(ExtraRow::SecretarialCompliance(m))) => {
            ScrField::LIST.iter().map(|f| f.display(m)).collect()
        }
        (NseFeedKind::RelatedPartyTransactions, Some(ExtraRow::RelatedPartyTransactions(m))) => {
            RptField::LIST.iter().map(|f| f.display(m)).collect()
        }
        (NseFeedKind::InvestorComplaints, Some(ExtraRow::InvestorComplaints(m))) => {
            IcField::LIST.iter().map(|f| f.display(m)).collect()
        }
        (NseFeedKind::InsiderTrading, Some(ExtraRow::InsiderTrading(m))) => {
            ItField::LIST.iter().map(|f| f.display(m)).collect()
        }
        (
            NseFeedKind::IntegratedFilingFinancials,
            Some(ExtraRow::IntegratedFilingFinancials(m)),
        ) => IffField::LIST.iter().map(|f| f.display(m)).collect(),
        (NseFeedKind::FinancialResults, Some(ExtraRow::FinancialResults(m))) => {
            FrField::LIST.iter().map(|f| f.display(m)).collect()
        }
        (NseFeedKind::Brsr, _) => BrsrField::LIST.iter().map(|_| String::new()).collect(),
        (NseFeedKind::VotingResults, _) => VoteField::LIST.iter().map(|_| String::new()).collect(),
        (NseFeedKind::UnitholdingPatterns, _) => {
            UhpField::LIST.iter().map(|_| String::new()).collect()
        }
        (NseFeedKind::StatementOfDeviation, _) => {
            SodField::LIST.iter().map(|_| String::new()).collect()
        }
        (NseFeedKind::ShareholdingPattern, _) => {
            ShpField::LIST.iter().map(|_| String::new()).collect()
        }
        (NseFeedKind::SecretarialCompliance, _) => {
            ScrField::LIST.iter().map(|_| String::new()).collect()
        }
        (NseFeedKind::RelatedPartyTransactions, _) => {
            RptField::LIST.iter().map(|_| String::new()).collect()
        }
        (NseFeedKind::InvestorComplaints, _) => {
            IcField::LIST.iter().map(|_| String::new()).collect()
        }
        (NseFeedKind::InsiderTrading, _) => ItField::LIST.iter().map(|_| String::new()).collect(),
        (NseFeedKind::IntegratedFilingFinancials, _) => {
            IffField::LIST.iter().map(|_| String::new()).collect()
        }
        (NseFeedKind::FinancialResults, _) => FrField::LIST.iter().map(|_| String::new()).collect(),
        _ => Vec::new(),
    }
}

pub fn detail_fields(
    kind: NseFeedKind,
    extra: Option<&ExtraRow>,
    tz: &str,
) -> (
    Vec<(String, String)>,
    Vec<SodObjectRow>,
    Vec<(String, String)>,
) {
    let desc = extra.map(ExtraRow::description_fields).unwrap_or_default();
    let mut head: Vec<_> = kind
        .extra_list_fields()
        .iter()
        .filter_map(|f| {
            let value = f.display(&desc, tz);
            (!value.is_empty()).then(|| (f.label().to_string(), value))
        })
        .collect();
    let mut objects = Vec::new();
    let mut tail = Vec::new();
    match (kind, extra) {
        (NseFeedKind::Brsr, Some(ExtraRow::Brsr(m))) => {
            head.extend(nonempty(BrsrField::DETAIL, |f| f.display(m)));
        }
        (NseFeedKind::VotingResults, Some(ExtraRow::VotingResults(m))) => {
            head.extend(nonempty(VoteField::DETAIL, |f| f.display(m)));
        }
        (NseFeedKind::UnitholdingPatterns, Some(ExtraRow::UnitholdingPatterns(m))) => {
            head.extend(nonempty(UhpField::DETAIL, |f| f.display(m)));
        }
        (NseFeedKind::ShareholdingPattern, Some(ExtraRow::ShareholdingPattern(m))) => {
            head.extend(nonempty(ShpField::DETAIL, |f| f.display(m)));
        }
        (NseFeedKind::SecretarialCompliance, Some(ExtraRow::SecretarialCompliance(m))) => {
            head.extend(nonempty(ScrField::DETAIL, |f| f.display(m)));
        }
        (NseFeedKind::RelatedPartyTransactions, Some(ExtraRow::RelatedPartyTransactions(m))) => {
            head.extend(nonempty(RptField::DETAIL, |f| f.display(m)));
        }
        (NseFeedKind::InvestorComplaints, Some(ExtraRow::InvestorComplaints(m))) => {
            head.extend(nonempty(IcField::DETAIL, |f| f.display(m)));
        }
        (NseFeedKind::InsiderTrading, Some(ExtraRow::InsiderTrading(m))) => {
            head.extend(nonempty(ItField::DETAIL, |f| f.display(m)));
        }
        (
            NseFeedKind::IntegratedFilingFinancials,
            Some(ExtraRow::IntegratedFilingFinancials(m)),
        ) => {
            head.extend(nonempty(IffField::DETAIL, |f| f.display(m)));
        }
        (NseFeedKind::FinancialResults, Some(ExtraRow::FinancialResults(m))) => {
            head.extend(nonempty(FrField::DETAIL, |f| f.display(m)));
        }
        (NseFeedKind::StatementOfDeviation, Some(ExtraRow::StatementOfDeviation(m))) => {
            head.extend(nonempty(SodField::DETAIL_HEAD, |f| f.display(m)));
            objects = sod::parse_object_rows(&m.objects);
            tail.extend(nonempty(SodField::DETAIL_TAIL, |f| f.display(m)));
        }
        _ => {}
    }
    (head, objects, tail)
}

fn nonempty<F: Copy + HasLabel>(
    fields: &[F],
    display: impl Fn(F) -> String,
) -> Vec<(String, String)> {
    fields
        .iter()
        .filter_map(|f| {
            let value = display(*f);
            (!value.is_empty()).then(|| (field_label(*f), value))
        })
        .collect()
}

fn field_label<F: Copy>(field: F) -> String
where
    F: HasLabel,
{
    field.label().to_string()
}

trait HasLabel {
    fn label(self) -> &'static str;
}

macro_rules! impl_label {
    ($($ty:ty),* $(,)?) => {
        $(
            impl HasLabel for $ty {
                fn label(self) -> &'static str {
                    Self::label(self)
                }
            }
        )*
    };
}

impl_label!(
    BrsrField, VoteField, UhpField, SodField, ShpField, ScrField, RptField, IcField, ItField,
    IffField, FrField
);

impl ExtraRow {
    pub fn json_name(&self) -> &'static str {
        match self {
            Self::Announcements(_) => "announcements",
            Self::DailyBuyback(_) => "daily_buyback",
            Self::AnnualReports(_) => "annual_reports",
            Self::CorporateActions(_) => "corporate_actions",
            Self::ReasonForEncumbrance(_) => "reason_for_encumbrance",
            Self::Regulation29(_) => "regulation_29",
            Self::Regulation31(_) => "regulation_31",
            Self::ShareTransfers(_) => "share_transfers",
            Self::Brsr(_) => "brsr",
            Self::VotingResults(_) => "voting_results",
            Self::UnitholdingPatterns(_) => "unitholding_patterns",
            Self::StatementOfDeviation(_) => "statement_of_deviation",
            Self::ShareholdingPattern(_) => "shareholding_pattern",
            Self::SecretarialCompliance(_) => "secretarial_compliance",
            Self::RelatedPartyTransactions(_) => "related_party_transactions",
            Self::InvestorComplaints(_) => "investor_complaints",
            Self::InsiderTrading(_) => "insider_trading",
            Self::IntegratedFilingFinancials(_) => "integrated_filing_financials",
            Self::FinancialResults(_) => "financial_results",
        }
    }

    pub fn to_json(&self) -> JsonValue {
        match self {
            Self::Announcements(m) => json_of(m),
            Self::DailyBuyback(m) => json_of(m),
            Self::AnnualReports(m) => json_of(m),
            Self::CorporateActions(m) => json_of(m),
            Self::ReasonForEncumbrance(m) => json_of(m),
            Self::Regulation29(m) => json_of(m),
            Self::Regulation31(m) => json_of(m),
            Self::ShareTransfers(m) => json_of(m),
            Self::Brsr(m) => json_of(m),
            Self::VotingResults(m) => json_of(m),
            Self::UnitholdingPatterns(m) => json_of(m),
            Self::StatementOfDeviation(m) => json_of(m),
            Self::ShareholdingPattern(m) => json_of(m),
            Self::SecretarialCompliance(m) => json_of(m),
            Self::RelatedPartyTransactions(m) => json_of(m),
            Self::InvestorComplaints(m) => json_of(m),
            Self::InsiderTrading(m) => json_of(m),
            Self::IntegratedFilingFinancials(m) => json_of(m),
            Self::FinancialResults(m) => json_of(m),
        }
    }

    fn description_fields(&self) -> NseDescriptionFields {
        let mut fields = NseDescriptionFields::default();
        match self {
            Self::Announcements(m) => fields.subject = m.subject.clone(),
            Self::DailyBuyback(m) => fields.subject = m.subject.clone(),
            Self::AnnualReports(m) => fields.as_on_date = m.as_on_date,
            Self::CorporateActions(m) => {
                fields.series = m.series.clone();
                fields.purpose = m.purpose.clone();
                fields.face_value = m.face_value.clone();
                fields.record_date = m.record_date;
                fields.book_closure_start_date = m.book_closure_start_date;
                fields.book_closure_end_date = m.book_closure_end_date;
            }
            Self::ReasonForEncumbrance(m) => {
                fields.encumbered_promoter_names = m.encumbered_promoter_names.clone();
            }
            Self::Regulation29(m) => fields.acquirer_names = m.acquirer_names.clone(),
            Self::Regulation31(m) => fields.promoter_names = m.promoter_names.clone(),
            Self::ShareTransfers(m) => fields.period_ended = m.period_ended,
            Self::Brsr(m) => fields.original_submission_date = m.original_submission_date,
            Self::VotingResults(m) => fields.meeting_date = m.meeting_date,
            Self::UnitholdingPatterns(m) => fields.as_on_date = m.as_on_date,
            Self::StatementOfDeviation(m) => fields.period_end_date = m.period_end_date,
            Self::SecretarialCompliance(m) => {
                fields.financial_year = m.financial_year.clone();
                fields.submission_type = m.submission_type.clone();
            }
            Self::RelatedPartyTransactions(m) => fields.period_end_date = m.period_end_date,
            Self::InvestorComplaints(m) => fields.for_quarter_ending = m.for_quarter_ending,
            Self::IntegratedFilingFinancials(m) => {
                fields.submission_type = m.submission_type.clone();
                fields.remarks = m.remarks.clone();
            }
            Self::FinancialResults(m) => {
                fields.relating_to = m.relating_to.clone();
                fields.audited_unaudited = m.audited_unaudited.clone();
                fields.cumulative = m.cumulative.clone();
                fields.consolidated = m.consolidated.clone();
                fields.ind_as = m.ind_as.clone();
                fields.period = m.period.clone();
                fields.period_ended = m.period_ended;
            }
            Self::ShareholdingPattern(_) | Self::InsiderTrading(_) => {}
        }
        fields
    }
}

fn json_of<T: Serialize>(value: &T) -> JsonValue {
    serde_json::to_value(value).unwrap_or(JsonValue::Null)
}

macro_rules! search_sat {
    ($ids:expr, $entity:ident, $cols:expr, $db:expr, $backend:expr, $query:expr, $limit:expr) => {{
        let mut select = $entity::Entity::find();
        select = trigram::apply_text_search(select, $backend, $cols, $query);
        for row in select.limit($limit).all($db).await? {
            $ids.insert(row.item_id);
        }
    }};
}

pub async fn search_satellite_item_ids(
    db: &DatabaseConnection,
    query: &str,
    limit: u64,
) -> Result<HashSet<i64>, sea_orm::DbErr> {
    if query.is_empty() {
        return Ok(HashSet::new());
    }
    let backend = db.get_database_backend();
    let mut ids = HashSet::new();
    search_sat!(
        ids,
        announcements,
        &[announcements::Column::Subject],
        db,
        backend,
        query,
        limit
    );
    search_sat!(
        ids,
        daily_buyback,
        &[daily_buyback::Column::Subject],
        db,
        backend,
        query,
        limit
    );
    search_sat!(
        ids,
        corporate_actions,
        &[
            corporate_actions::Column::Series,
            corporate_actions::Column::Purpose,
            corporate_actions::Column::FaceValue,
        ],
        db,
        backend,
        query,
        limit
    );
    search_sat!(
        ids,
        reason_for_encumbrance,
        &[reason_for_encumbrance::Column::EncumberedPromoterNames],
        db,
        backend,
        query,
        limit
    );
    search_sat!(
        ids,
        regulation_29,
        &[regulation_29::Column::AcquirerNames],
        db,
        backend,
        query,
        limit
    );
    search_sat!(
        ids,
        regulation_31,
        &[regulation_31::Column::PromoterNames],
        db,
        backend,
        query,
        limit
    );
    search_sat!(
        ids,
        brsr_ent,
        &[
            brsr_ent::Column::NseSymbol,
            brsr_ent::Column::ScripCode,
            brsr_ent::Column::MseiSymbol,
            brsr_ent::Column::Isin,
            brsr_ent::Column::Cin,
            brsr_ent::Column::CompanyName,
            brsr_ent::Column::RegisteredOffice,
            brsr_ent::Column::CorporateOffice,
            brsr_ent::Column::Email,
            brsr_ent::Column::Telephone,
            brsr_ent::Column::Website,
            brsr_ent::Column::PaidUpCapital,
            brsr_ent::Column::ContactPerson,
            brsr_ent::Column::ContactPhone,
            brsr_ent::Column::ContactEmail,
            brsr_ent::Column::ReportingBoundary,
            brsr_ent::Column::CoreAssurance,
            brsr_ent::Column::Turnover,
            brsr_ent::Column::NetWorth,
            brsr_ent::Column::StatesServed,
            brsr_ent::Column::CountriesServed,
            brsr_ent::Column::BoardSize,
            brsr_ent::Column::FemaleDirectors,
            brsr_ent::Column::Kmp,
            brsr_ent::Column::FemaleKmp,
            brsr_ent::Column::CsrApplicable,
        ],
        db,
        backend,
        query,
        limit
    );
    search_sat!(
        ids,
        voting_results,
        &[
            voting_results::Column::ScripCode,
            voting_results::Column::Symbol,
            voting_results::Column::MseiSymbol,
            voting_results::Column::Isin,
            voting_results::Column::CompanyName,
            voting_results::Column::TypeOfMeeting,
            voting_results::Column::StartTime,
            voting_results::Column::EndTime,
            voting_results::Column::Scrutinizer,
            voting_results::Column::ScrutinizerFirm,
            voting_results::Column::ScrutinizerQualification,
            voting_results::Column::ScrutinizerMembership,
            voting_results::Column::ShareholdersOnRecord,
            voting_results::Column::PromotersInPerson,
            voting_results::Column::PublicInPerson,
            voting_results::Column::PromotersVc,
            voting_results::Column::PublicVc,
            voting_results::Column::ResolutionsPassed,
        ],
        db,
        backend,
        query,
        limit
    );
    search_sat!(
        ids,
        unitholding_patterns,
        &[
            unitholding_patterns::Column::ScripCode,
            unitholding_patterns::Column::NseSymbol,
            unitholding_patterns::Column::MseiSymbol,
            unitholding_patterns::Column::SebiRegistration,
            unitholding_patterns::Column::CompanyName,
            unitholding_patterns::Column::TypeOfReport,
            unitholding_patterns::Column::NumberOfSecurities,
        ],
        db,
        backend,
        query,
        limit
    );
    search_sat!(
        ids,
        statement_of_deviation,
        &[
            statement_of_deviation::Column::NseSymbol,
            statement_of_deviation::Column::ScripCode,
            statement_of_deviation::Column::MseiSymbol,
            statement_of_deviation::Column::Isin,
            statement_of_deviation::Column::CompanyName,
            statement_of_deviation::Column::StatementCount,
            statement_of_deviation::Column::ModeOfFundRaising,
            statement_of_deviation::Column::AmountRaised,
            statement_of_deviation::Column::MonitoringAgency,
            statement_of_deviation::Column::MonitoringAgencyName,
            statement_of_deviation::Column::HasDeviation,
            statement_of_deviation::Column::DeviationExplanation,
            statement_of_deviation::Column::ShareholderApproved,
            statement_of_deviation::Column::AuditCommitteeComments,
            statement_of_deviation::Column::AuditorComments,
            statement_of_deviation::Column::Signatory,
            statement_of_deviation::Column::Designation,
            statement_of_deviation::Column::Place,
        ],
        db,
        backend,
        query,
        limit
    );
    search_sat!(
        ids,
        shareholding_pattern,
        &[
            shareholding_pattern::Column::NseSymbol,
            shareholding_pattern::Column::ScripCode,
            shareholding_pattern::Column::MseiSymbol,
            shareholding_pattern::Column::Isin,
            shareholding_pattern::Column::CompanyName,
            shareholding_pattern::Column::ClassOfSecurity,
            shareholding_pattern::Column::TypeOfReport,
            shareholding_pattern::Column::FiledUnder,
            shareholding_pattern::Column::PromoterPct,
            shareholding_pattern::Column::PublicPct,
            shareholding_pattern::Column::PromoterShares,
            shareholding_pattern::Column::PublicShares,
        ],
        db,
        backend,
        query,
        limit
    );
    search_sat!(
        ids,
        secretarial_compliance,
        &[
            secretarial_compliance::Column::FinancialYear,
            secretarial_compliance::Column::SubmissionType,
            secretarial_compliance::Column::NseSymbol,
            secretarial_compliance::Column::ScripCode,
            secretarial_compliance::Column::MseiSymbol,
            secretarial_compliance::Column::Isin,
            secretarial_compliance::Column::CompanyName,
            secretarial_compliance::Column::ObservationsReported,
            secretarial_compliance::Column::PreviousObservations,
            secretarial_compliance::Column::ActionsTaken,
            secretarial_compliance::Column::CertifyingFirm,
            secretarial_compliance::Column::PcsName,
            secretarial_compliance::Column::MembershipType,
            secretarial_compliance::Column::MembershipNumber,
            secretarial_compliance::Column::Udin,
            secretarial_compliance::Column::CpNumber,
            secretarial_compliance::Column::Place,
        ],
        db,
        backend,
        query,
        limit
    );
    search_sat!(
        ids,
        related_party_transactions,
        &[
            related_party_transactions::Column::NseSymbol,
            related_party_transactions::Column::ScripCode,
            related_party_transactions::Column::MseiSymbol,
            related_party_transactions::Column::CompanyName,
            related_party_transactions::Column::ReportingPeriod,
            related_party_transactions::Column::HasRelatedParty,
            related_party_transactions::Column::EnteredTransactions,
            related_party_transactions::Column::TransactionCount,
            related_party_transactions::Column::Counterparty,
            related_party_transactions::Column::TransactionType,
            related_party_transactions::Column::Amount,
        ],
        db,
        backend,
        query,
        limit
    );
    search_sat!(
        ids,
        investor_complaints,
        &[
            investor_complaints::Column::NseSymbol,
            investor_complaints::Column::ScripCode,
            investor_complaints::Column::MseiSymbol,
            investor_complaints::Column::Isin,
            investor_complaints::Column::CompanyName,
            investor_complaints::Column::Class,
            investor_complaints::Column::SubmissionType,
            investor_complaints::Column::PendingStart,
            investor_complaints::Column::Received,
            investor_complaints::Column::Disposed,
            investor_complaints::Column::PendingEnd,
            investor_complaints::Column::ScoresId,
        ],
        db,
        backend,
        query,
        limit
    );
    search_sat!(
        ids,
        insider_trading,
        &[
            insider_trading::Column::NseSymbol,
            insider_trading::Column::ScripCode,
            insider_trading::Column::MseiSymbol,
            insider_trading::Column::Isin,
            insider_trading::Column::CompanyName,
            insider_trading::Column::Regulation,
            insider_trading::Column::Instrument,
            insider_trading::Column::Person,
            insider_trading::Column::Category,
            insider_trading::Column::TxnType,
            insider_trading::Column::Qty,
            insider_trading::Column::Value,
            insider_trading::Column::Mode,
            insider_trading::Column::PriorQty,
            insider_trading::Column::PriorPct,
            insider_trading::Column::PostQty,
            insider_trading::Column::PostPct,
            insider_trading::Column::Signatory,
            insider_trading::Column::Designation,
            insider_trading::Column::Exchange,
        ],
        db,
        backend,
        query,
        limit
    );
    search_sat!(
        ids,
        integrated_filing_financials,
        &[
            integrated_filing_financials::Column::SubmissionType,
            integrated_filing_financials::Column::Remarks,
            integrated_filing_financials::Column::NseSymbol,
            integrated_filing_financials::Column::ScripCode,
            integrated_filing_financials::Column::MseiSymbol,
            integrated_filing_financials::Column::Isin,
            integrated_filing_financials::Column::CompanyName,
            integrated_filing_financials::Column::TypeOfCompany,
            integrated_filing_financials::Column::ClassOfSecurity,
            integrated_filing_financials::Column::ReportingPeriod,
            integrated_filing_financials::Column::ReportingQuarter,
            integrated_filing_financials::Column::Audited,
            integrated_filing_financials::Column::Nature,
            integrated_filing_financials::Column::Revenue,
            integrated_filing_financials::Column::Profit,
        ],
        db,
        backend,
        query,
        limit
    );
    search_sat!(
        ids,
        financial_results,
        &[
            financial_results::Column::RelatingTo,
            financial_results::Column::AuditedUnaudited,
            financial_results::Column::Cumulative,
            financial_results::Column::Consolidated,
            financial_results::Column::IndAs,
            financial_results::Column::Period,
            financial_results::Column::NseSymbol,
            financial_results::Column::ScripCode,
            financial_results::Column::MseiSymbol,
            financial_results::Column::CompanyName,
            financial_results::Column::ClassOfSecurity,
            financial_results::Column::ReportingQuarter,
            financial_results::Column::Audited,
            financial_results::Column::Nature,
            financial_results::Column::Revenue,
            financial_results::Column::Profit,
        ],
        db,
        backend,
        query,
        limit
    );
    Ok(ids)
}
