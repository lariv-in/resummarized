//! Per-feed satellite load, upsert, sort, display, and search.

use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;
use lariv_rs::db::trigram;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, JoinType,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait, Select,
};
use serde::Serialize;
use serde_json::Value as JsonValue;

use super::description::{BseDescriptionFields, DescriptionField};
use super::entities::{
    announcements, annual_reports, board_meetings, corporate_actions, financial_results,
    insider_trading, item, shareholding_pattern, voting_results,
};
use super::feeds::BseFeedKind;
use crate::list_filters::{
    FilterField, FilterKind, apply_datetime_day, apply_text_contains, apply_value_filter,
    filter_value,
};

#[derive(Clone, Debug)]
pub enum ExtraRow {
    Announcements(announcements::Model),
    AnnualReports(annual_reports::Model),
    BoardMeetings(board_meetings::Model),
    CorporateActions(corporate_actions::Model),
    FinancialResults(financial_results::Model),
    InsiderTrading(insider_trading::Model),
    ShareholdingPattern(shareholding_pattern::Model),
    VotingResults(voting_results::Model),
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

fn set_opt(slot: &mut sea_orm::ActiveValue<Option<String>>, value: &Option<String>) {
    if let Some(v) = value {
        *slot = Set(Some(v.clone()));
    }
}

fn set_date(slot: &mut sea_orm::ActiveValue<Option<NaiveDate>>, value: Option<NaiveDate>) {
    if let Some(v) = value {
        *slot = Set(Some(v));
    }
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
    kind: BseFeedKind,
    ids: &[i64],
) -> Result<HashMap<i64, ExtraRow>, sea_orm::DbErr> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    let mut map = HashMap::new();
    match kind {
        BseFeedKind::Announcements => load_kind!(map, announcements, Announcements, db, ids),
        BseFeedKind::AnnualReports => load_kind!(map, annual_reports, AnnualReports, db, ids),
        BseFeedKind::BoardMeetings => load_kind!(map, board_meetings, BoardMeetings, db, ids),
        BseFeedKind::CorporateActions => {
            load_kind!(map, corporate_actions, CorporateActions, db, ids)
        }
        BseFeedKind::FinancialResults => {
            load_kind!(map, financial_results, FinancialResults, db, ids)
        }
        BseFeedKind::InsiderTrading => load_kind!(map, insider_trading, InsiderTrading, db, ids),
        BseFeedKind::ShareholdingPattern => {
            load_kind!(map, shareholding_pattern, ShareholdingPattern, db, ids)
        }
        BseFeedKind::VotingResults => load_kind!(map, voting_results, VotingResults, db, ids),
        BseFeedKind::Sensex
        | BseFeedKind::Notices
        | BseFeedKind::MediaRelease
        | BseFeedKind::IndexMediaRelease => {}
    }
    Ok(map)
}

pub async fn upsert(
    db: &DatabaseConnection,
    kind: BseFeedKind,
    item_id: i64,
    parsed: &BseDescriptionFields,
    scripcode: Option<&str>,
) -> Result<(), sea_orm::DbErr> {
    let code = scripcode
        .map(str::to_string)
        .or_else(|| parsed.scripcode.clone());
    match kind {
        BseFeedKind::Announcements => persist!(announcements, db, item_id, am, {
            set_opt(&mut am.scripcode, &code);
        }),
        BseFeedKind::AnnualReports => persist!(annual_reports, db, item_id, am, {
            set_opt(&mut am.scripcode, &code);
            set_date(&mut am.as_on_date, parsed.as_on_date);
        }),
        BseFeedKind::BoardMeetings => persist!(board_meetings, db, item_id, am, {
            set_opt(&mut am.scripcode, &code);
            set_date(&mut am.meeting_date, parsed.meeting_date);
            set_opt(&mut am.purpose, &parsed.purpose);
        }),
        BseFeedKind::CorporateActions => persist!(corporate_actions, db, item_id, am, {
            set_opt(&mut am.scripcode, &code);
            set_opt(&mut am.segment, &parsed.segment);
            set_opt(&mut am.purpose, &parsed.purpose);
            set_date(&mut am.rd_date, parsed.rd_date);
            set_date(&mut am.bc_start_date, parsed.bc_start_date);
            set_date(&mut am.bc_end_date, parsed.bc_end_date);
            set_date(&mut am.nd_start_date, parsed.nd_start_date);
            set_date(&mut am.nd_end_date, parsed.nd_end_date);
            set_date(&mut am.actual_payment_date, parsed.actual_payment_date);
        }),
        BseFeedKind::FinancialResults => persist!(financial_results, db, item_id, am, {
            set_opt(&mut am.scripcode, &code);
            set_opt(&mut am.audited_unaudited, &parsed.audited_unaudited);
            set_opt(
                &mut am.standalone_consolidated,
                &parsed.standalone_consolidated,
            );
            set_date(&mut am.period_start_date, parsed.period_start_date);
            set_date(&mut am.period_end_date, parsed.period_end_date);
            set_opt(&mut am.ind_as, &parsed.ind_as);
        }),
        BseFeedKind::InsiderTrading => persist!(insider_trading, db, item_id, am, {
            set_opt(&mut am.scripcode, &code);
            set_opt(&mut am.type_of_security, &parsed.type_of_security);
        }),
        BseFeedKind::ShareholdingPattern => persist!(shareholding_pattern, db, item_id, am, {
            set_opt(&mut am.scripcode, &code);
            set_date(&mut am.as_on_date, parsed.as_on_date);
            set_opt(&mut am.promoter_and_group, &parsed.promoter_and_group);
            set_opt(&mut am.public_val, &parsed.public_val);
            set_opt(&mut am.emptr, &parsed.emptr);
            set_opt(&mut am.status, &parsed.status);
            set_date(&mut am.submission_date, parsed.submission_date);
            set_date(&mut am.revised_filing_date, parsed.revised_filing_date);
        }),
        BseFeedKind::VotingResults => persist!(voting_results, db, item_id, am, {
            set_opt(&mut am.scripcode, &code);
            set_date(&mut am.meeting_date, parsed.meeting_date);
            set_opt(&mut am.meeting_type, &parsed.meeting_type);
        }),
        BseFeedKind::Sensex
        | BseFeedKind::Notices
        | BseFeedKind::MediaRelease
        | BseFeedKind::IndexMediaRelease => {}
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
    _rel: sea_orm::RelationDef,
    col: C,
    desc: bool,
) -> Select<item::Entity> {
    order(query, col, desc)
}

pub fn satellite_relation(kind: BseFeedKind) -> Option<sea_orm::RelationDef> {
    Some(match kind {
        BseFeedKind::Announcements => item::Relation::Announcements.def(),
        BseFeedKind::AnnualReports => item::Relation::AnnualReports.def(),
        BseFeedKind::BoardMeetings => item::Relation::BoardMeetings.def(),
        BseFeedKind::CorporateActions => item::Relation::CorporateActions.def(),
        BseFeedKind::FinancialResults => item::Relation::FinancialResults.def(),
        BseFeedKind::InsiderTrading => item::Relation::InsiderTrading.def(),
        BseFeedKind::ShareholdingPattern => item::Relation::ShareholdingPattern.def(),
        BseFeedKind::VotingResults => item::Relation::VotingResults.def(),
        BseFeedKind::Sensex
        | BseFeedKind::Notices
        | BseFeedKind::MediaRelease
        | BseFeedKind::IndexMediaRelease => return None,
    })
}

pub fn list_select(kind: BseFeedKind) -> Select<item::Entity> {
    let query = item::Entity::find().filter(item::Column::FeedKind.eq(kind.slug()));
    match satellite_relation(kind) {
        Some(rel) => query.join(JoinType::LeftJoin, rel),
        None => query,
    }
}

pub fn apply_sort(
    query: Select<item::Entity>,
    kind: BseFeedKind,
    sort: &str,
) -> Select<item::Entity> {
    if let Some(desc) = sort_direction(sort, "Title") {
        return order(query, item::Column::Title, desc);
    }
    if let Some(desc) = sort_direction(sort, "PubDate") {
        return order(query, item::Column::PubDate, desc);
    }
    if let Some(desc) = sort_direction(sort, "Description") {
        return order(query, item::Column::Description, desc);
    }
    if let Some((desc, field)) = kind
        .extra_list_fields()
        .iter()
        .find_map(|f| sort_direction(sort, f.sort_key()).map(|d| (d, *f)))
        && let Some(sorted) = desc_sort(query.clone(), kind, field, desc)
    {
        return sorted;
    }
    query.order_by_desc(item::Column::Id)
}

pub fn apply_filters(
    mut query: Select<item::Entity>,
    kind: BseFeedKind,
    filters: &HashMap<String, String>,
) -> Select<item::Entity> {
    if let Some(v) = filter_value(filters, "Title") {
        query = apply_text_contains(query, item::Column::Title, v);
    }
    if let Some(v) = filter_value(filters, "PubDate") {
        query = apply_datetime_day(query, item::Column::PubDate, v);
    }
    if let Some(v) = filter_value(filters, "Description") {
        query = apply_text_contains(query, item::Column::Description, v);
    }
    for field in kind.extra_list_fields() {
        if let Some(v) = filter_value(filters, field.sort_key()) {
            query = desc_filter(query, kind, *field, v);
        }
    }
    query
}

fn desc_sort(
    query: Select<item::Entity>,
    kind: BseFeedKind,
    field: DescriptionField,
    desc: bool,
) -> Option<Select<item::Entity>> {
    Some(match (kind, field) {
        (BseFeedKind::Announcements, DescriptionField::Scripcode) => join_order(
            query,
            item::Relation::Announcements.def(),
            announcements::Column::Scripcode,
            desc,
        ),
        (BseFeedKind::AnnualReports, DescriptionField::Scripcode) => join_order(
            query,
            item::Relation::AnnualReports.def(),
            annual_reports::Column::Scripcode,
            desc,
        ),
        (BseFeedKind::AnnualReports, DescriptionField::AsOnDate) => join_order(
            query,
            item::Relation::AnnualReports.def(),
            annual_reports::Column::AsOnDate,
            desc,
        ),
        (BseFeedKind::BoardMeetings, DescriptionField::Scripcode) => join_order(
            query,
            item::Relation::BoardMeetings.def(),
            board_meetings::Column::Scripcode,
            desc,
        ),
        (BseFeedKind::BoardMeetings, DescriptionField::MeetingDate) => join_order(
            query,
            item::Relation::BoardMeetings.def(),
            board_meetings::Column::MeetingDate,
            desc,
        ),
        (BseFeedKind::BoardMeetings, DescriptionField::Purpose) => join_order(
            query,
            item::Relation::BoardMeetings.def(),
            board_meetings::Column::Purpose,
            desc,
        ),
        (BseFeedKind::CorporateActions, DescriptionField::Scripcode) => join_order(
            query,
            item::Relation::CorporateActions.def(),
            corporate_actions::Column::Scripcode,
            desc,
        ),
        (BseFeedKind::CorporateActions, DescriptionField::Segment) => join_order(
            query,
            item::Relation::CorporateActions.def(),
            corporate_actions::Column::Segment,
            desc,
        ),
        (BseFeedKind::CorporateActions, DescriptionField::Purpose) => join_order(
            query,
            item::Relation::CorporateActions.def(),
            corporate_actions::Column::Purpose,
            desc,
        ),
        (BseFeedKind::CorporateActions, DescriptionField::RdDate) => join_order(
            query,
            item::Relation::CorporateActions.def(),
            corporate_actions::Column::RdDate,
            desc,
        ),
        (BseFeedKind::CorporateActions, DescriptionField::BcStartDate) => join_order(
            query,
            item::Relation::CorporateActions.def(),
            corporate_actions::Column::BcStartDate,
            desc,
        ),
        (BseFeedKind::CorporateActions, DescriptionField::BcEndDate) => join_order(
            query,
            item::Relation::CorporateActions.def(),
            corporate_actions::Column::BcEndDate,
            desc,
        ),
        (BseFeedKind::CorporateActions, DescriptionField::NdStartDate) => join_order(
            query,
            item::Relation::CorporateActions.def(),
            corporate_actions::Column::NdStartDate,
            desc,
        ),
        (BseFeedKind::CorporateActions, DescriptionField::NdEndDate) => join_order(
            query,
            item::Relation::CorporateActions.def(),
            corporate_actions::Column::NdEndDate,
            desc,
        ),
        (BseFeedKind::CorporateActions, DescriptionField::ActualPaymentDate) => join_order(
            query,
            item::Relation::CorporateActions.def(),
            corporate_actions::Column::ActualPaymentDate,
            desc,
        ),
        (BseFeedKind::FinancialResults, DescriptionField::Scripcode) => join_order(
            query,
            item::Relation::FinancialResults.def(),
            financial_results::Column::Scripcode,
            desc,
        ),
        (BseFeedKind::FinancialResults, DescriptionField::AuditedUnaudited) => join_order(
            query,
            item::Relation::FinancialResults.def(),
            financial_results::Column::AuditedUnaudited,
            desc,
        ),
        (BseFeedKind::FinancialResults, DescriptionField::StandaloneConsolidated) => join_order(
            query,
            item::Relation::FinancialResults.def(),
            financial_results::Column::StandaloneConsolidated,
            desc,
        ),
        (BseFeedKind::FinancialResults, DescriptionField::PeriodStartDate) => join_order(
            query,
            item::Relation::FinancialResults.def(),
            financial_results::Column::PeriodStartDate,
            desc,
        ),
        (BseFeedKind::FinancialResults, DescriptionField::PeriodEndDate) => join_order(
            query,
            item::Relation::FinancialResults.def(),
            financial_results::Column::PeriodEndDate,
            desc,
        ),
        (BseFeedKind::FinancialResults, DescriptionField::IndAs) => join_order(
            query,
            item::Relation::FinancialResults.def(),
            financial_results::Column::IndAs,
            desc,
        ),
        (BseFeedKind::InsiderTrading, DescriptionField::Scripcode) => join_order(
            query,
            item::Relation::InsiderTrading.def(),
            insider_trading::Column::Scripcode,
            desc,
        ),
        (BseFeedKind::InsiderTrading, DescriptionField::TypeOfSecurity) => join_order(
            query,
            item::Relation::InsiderTrading.def(),
            insider_trading::Column::TypeOfSecurity,
            desc,
        ),
        (BseFeedKind::ShareholdingPattern, DescriptionField::Scripcode) => join_order(
            query,
            item::Relation::ShareholdingPattern.def(),
            shareholding_pattern::Column::Scripcode,
            desc,
        ),
        (BseFeedKind::ShareholdingPattern, DescriptionField::AsOnDate) => join_order(
            query,
            item::Relation::ShareholdingPattern.def(),
            shareholding_pattern::Column::AsOnDate,
            desc,
        ),
        (BseFeedKind::ShareholdingPattern, DescriptionField::PromoterAndGroup) => join_order(
            query,
            item::Relation::ShareholdingPattern.def(),
            shareholding_pattern::Column::PromoterAndGroup,
            desc,
        ),
        (BseFeedKind::ShareholdingPattern, DescriptionField::PublicVal) => join_order(
            query,
            item::Relation::ShareholdingPattern.def(),
            shareholding_pattern::Column::PublicVal,
            desc,
        ),
        (BseFeedKind::ShareholdingPattern, DescriptionField::Status) => join_order(
            query,
            item::Relation::ShareholdingPattern.def(),
            shareholding_pattern::Column::Status,
            desc,
        ),
        (BseFeedKind::ShareholdingPattern, DescriptionField::SubmissionDate) => join_order(
            query,
            item::Relation::ShareholdingPattern.def(),
            shareholding_pattern::Column::SubmissionDate,
            desc,
        ),
        (BseFeedKind::ShareholdingPattern, DescriptionField::RevisedFilingDate) => join_order(
            query,
            item::Relation::ShareholdingPattern.def(),
            shareholding_pattern::Column::RevisedFilingDate,
            desc,
        ),
        (BseFeedKind::VotingResults, DescriptionField::Scripcode) => join_order(
            query,
            item::Relation::VotingResults.def(),
            voting_results::Column::Scripcode,
            desc,
        ),
        (BseFeedKind::VotingResults, DescriptionField::MeetingDate) => join_order(
            query,
            item::Relation::VotingResults.def(),
            voting_results::Column::MeetingDate,
            desc,
        ),
        (BseFeedKind::VotingResults, DescriptionField::MeetingType) => join_order(
            query,
            item::Relation::VotingResults.def(),
            voting_results::Column::MeetingType,
            desc,
        ),
        _ => return None,
    })
}

fn desc_filter(
    query: Select<item::Entity>,
    kind: BseFeedKind,
    field: DescriptionField,
    value: &str,
) -> Select<item::Entity> {
    let kind_v = field.value_kind();
    match (kind, field) {
        (BseFeedKind::Announcements, DescriptionField::Scripcode) => {
            apply_value_filter(query, announcements::Column::Scripcode, kind_v, value)
        }
        (BseFeedKind::AnnualReports, DescriptionField::Scripcode) => {
            apply_value_filter(query, annual_reports::Column::Scripcode, kind_v, value)
        }
        (BseFeedKind::AnnualReports, DescriptionField::AsOnDate) => {
            apply_value_filter(query, annual_reports::Column::AsOnDate, kind_v, value)
        }
        (BseFeedKind::BoardMeetings, DescriptionField::Scripcode) => {
            apply_value_filter(query, board_meetings::Column::Scripcode, kind_v, value)
        }
        (BseFeedKind::BoardMeetings, DescriptionField::MeetingDate) => {
            apply_value_filter(query, board_meetings::Column::MeetingDate, kind_v, value)
        }
        (BseFeedKind::BoardMeetings, DescriptionField::Purpose) => {
            apply_value_filter(query, board_meetings::Column::Purpose, kind_v, value)
        }
        (BseFeedKind::CorporateActions, DescriptionField::Scripcode) => {
            apply_value_filter(query, corporate_actions::Column::Scripcode, kind_v, value)
        }
        (BseFeedKind::CorporateActions, DescriptionField::Segment) => {
            apply_value_filter(query, corporate_actions::Column::Segment, kind_v, value)
        }
        (BseFeedKind::CorporateActions, DescriptionField::Purpose) => {
            apply_value_filter(query, corporate_actions::Column::Purpose, kind_v, value)
        }
        (BseFeedKind::CorporateActions, DescriptionField::RdDate) => {
            apply_value_filter(query, corporate_actions::Column::RdDate, kind_v, value)
        }
        (BseFeedKind::CorporateActions, DescriptionField::BcStartDate) => {
            apply_value_filter(query, corporate_actions::Column::BcStartDate, kind_v, value)
        }
        (BseFeedKind::CorporateActions, DescriptionField::BcEndDate) => {
            apply_value_filter(query, corporate_actions::Column::BcEndDate, kind_v, value)
        }
        (BseFeedKind::CorporateActions, DescriptionField::NdStartDate) => {
            apply_value_filter(query, corporate_actions::Column::NdStartDate, kind_v, value)
        }
        (BseFeedKind::CorporateActions, DescriptionField::NdEndDate) => {
            apply_value_filter(query, corporate_actions::Column::NdEndDate, kind_v, value)
        }
        (BseFeedKind::CorporateActions, DescriptionField::ActualPaymentDate) => apply_value_filter(
            query,
            corporate_actions::Column::ActualPaymentDate,
            kind_v,
            value,
        ),
        (BseFeedKind::FinancialResults, DescriptionField::Scripcode) => {
            apply_value_filter(query, financial_results::Column::Scripcode, kind_v, value)
        }
        (BseFeedKind::FinancialResults, DescriptionField::AuditedUnaudited) => apply_value_filter(
            query,
            financial_results::Column::AuditedUnaudited,
            kind_v,
            value,
        ),
        (BseFeedKind::FinancialResults, DescriptionField::StandaloneConsolidated) => {
            apply_value_filter(
                query,
                financial_results::Column::StandaloneConsolidated,
                kind_v,
                value,
            )
        }
        (BseFeedKind::FinancialResults, DescriptionField::PeriodStartDate) => apply_value_filter(
            query,
            financial_results::Column::PeriodStartDate,
            kind_v,
            value,
        ),
        (BseFeedKind::FinancialResults, DescriptionField::PeriodEndDate) => apply_value_filter(
            query,
            financial_results::Column::PeriodEndDate,
            kind_v,
            value,
        ),
        (BseFeedKind::FinancialResults, DescriptionField::IndAs) => {
            apply_value_filter(query, financial_results::Column::IndAs, kind_v, value)
        }
        (BseFeedKind::InsiderTrading, DescriptionField::Scripcode) => {
            apply_value_filter(query, insider_trading::Column::Scripcode, kind_v, value)
        }
        (BseFeedKind::InsiderTrading, DescriptionField::TypeOfSecurity) => apply_value_filter(
            query,
            insider_trading::Column::TypeOfSecurity,
            kind_v,
            value,
        ),
        (BseFeedKind::ShareholdingPattern, DescriptionField::Scripcode) => apply_value_filter(
            query,
            shareholding_pattern::Column::Scripcode,
            kind_v,
            value,
        ),
        (BseFeedKind::ShareholdingPattern, DescriptionField::AsOnDate) => {
            apply_value_filter(query, shareholding_pattern::Column::AsOnDate, kind_v, value)
        }
        (BseFeedKind::ShareholdingPattern, DescriptionField::PromoterAndGroup) => {
            apply_value_filter(
                query,
                shareholding_pattern::Column::PromoterAndGroup,
                kind_v,
                value,
            )
        }
        (BseFeedKind::ShareholdingPattern, DescriptionField::PublicVal) => apply_value_filter(
            query,
            shareholding_pattern::Column::PublicVal,
            kind_v,
            value,
        ),
        (BseFeedKind::ShareholdingPattern, DescriptionField::Status) => {
            apply_value_filter(query, shareholding_pattern::Column::Status, kind_v, value)
        }
        (BseFeedKind::ShareholdingPattern, DescriptionField::SubmissionDate) => apply_value_filter(
            query,
            shareholding_pattern::Column::SubmissionDate,
            kind_v,
            value,
        ),
        (BseFeedKind::ShareholdingPattern, DescriptionField::RevisedFilingDate) => {
            apply_value_filter(
                query,
                shareholding_pattern::Column::RevisedFilingDate,
                kind_v,
                value,
            )
        }
        (BseFeedKind::VotingResults, DescriptionField::Scripcode) => {
            apply_value_filter(query, voting_results::Column::Scripcode, kind_v, value)
        }
        (BseFeedKind::VotingResults, DescriptionField::MeetingDate) => {
            apply_value_filter(query, voting_results::Column::MeetingDate, kind_v, value)
        }
        (BseFeedKind::VotingResults, DescriptionField::MeetingType) => {
            apply_value_filter(query, voting_results::Column::MeetingType, kind_v, value)
        }
        _ => query,
    }
}

pub fn extra_columns(kind: BseFeedKind) -> Vec<FilterField> {
    kind.extra_list_fields()
        .iter()
        .map(|f| FilterField {
            key: f.sort_key(),
            label: f.label(),
            kind: f.value_kind().filter_kind(),
        })
        .collect()
}

pub fn filter_fields(kind: BseFeedKind) -> Vec<FilterField> {
    let mut fields = vec![
        FilterField {
            key: "Title",
            label: "Title",
            kind: FilterKind::Text,
        },
        FilterField {
            key: "PubDate",
            label: "PubDate",
            kind: FilterKind::Date,
        },
    ];
    fields.extend(extra_columns(kind));
    if kind.shows_description_column() {
        fields.push(FilterField {
            key: "Description",
            label: "Description",
            kind: FilterKind::Text,
        });
    }
    fields
}

pub fn extra_cells(kind: BseFeedKind, extra: Option<&ExtraRow>, tz: &str) -> Vec<String> {
    let desc = extra.map(ExtraRow::description_fields).unwrap_or_default();
    kind.extra_list_fields()
        .iter()
        .map(|f| f.display(&desc, tz))
        .collect()
}

pub fn detail_fields(extra: Option<&ExtraRow>, tz: &str) -> Vec<(String, String)> {
    let desc = extra.map(ExtraRow::description_fields).unwrap_or_default();
    DescriptionField::ALL
        .iter()
        .filter_map(|f| {
            let value = f.display(&desc, tz);
            (!value.is_empty()).then(|| (f.label().to_string(), value))
        })
        .collect()
}

impl ExtraRow {
    pub fn json_name(&self) -> &'static str {
        match self {
            Self::Announcements(_) => "announcements",
            Self::AnnualReports(_) => "annual_reports",
            Self::BoardMeetings(_) => "board_meetings",
            Self::CorporateActions(_) => "corporate_actions",
            Self::FinancialResults(_) => "financial_results",
            Self::InsiderTrading(_) => "insider_trading",
            Self::ShareholdingPattern(_) => "shareholding_pattern",
            Self::VotingResults(_) => "voting_results",
        }
    }

    pub fn to_json(&self) -> JsonValue {
        match self {
            Self::Announcements(m) => json_of(m),
            Self::AnnualReports(m) => json_of(m),
            Self::BoardMeetings(m) => json_of(m),
            Self::CorporateActions(m) => json_of(m),
            Self::FinancialResults(m) => json_of(m),
            Self::InsiderTrading(m) => json_of(m),
            Self::ShareholdingPattern(m) => json_of(m),
            Self::VotingResults(m) => json_of(m),
        }
    }

    fn description_fields(&self) -> BseDescriptionFields {
        let mut fields = BseDescriptionFields::default();
        match self {
            Self::Announcements(m) => fields.scripcode = m.scripcode.clone(),
            Self::AnnualReports(m) => {
                fields.scripcode = m.scripcode.clone();
                fields.as_on_date = m.as_on_date;
            }
            Self::BoardMeetings(m) => {
                fields.scripcode = m.scripcode.clone();
                fields.meeting_date = m.meeting_date;
                fields.purpose = m.purpose.clone();
            }
            Self::CorporateActions(m) => {
                fields.scripcode = m.scripcode.clone();
                fields.segment = m.segment.clone();
                fields.purpose = m.purpose.clone();
                fields.rd_date = m.rd_date;
                fields.bc_start_date = m.bc_start_date;
                fields.bc_end_date = m.bc_end_date;
                fields.nd_start_date = m.nd_start_date;
                fields.nd_end_date = m.nd_end_date;
                fields.actual_payment_date = m.actual_payment_date;
            }
            Self::FinancialResults(m) => {
                fields.scripcode = m.scripcode.clone();
                fields.audited_unaudited = m.audited_unaudited.clone();
                fields.standalone_consolidated = m.standalone_consolidated.clone();
                fields.period_start_date = m.period_start_date;
                fields.period_end_date = m.period_end_date;
                fields.ind_as = m.ind_as.clone();
            }
            Self::InsiderTrading(m) => {
                fields.scripcode = m.scripcode.clone();
                fields.type_of_security = m.type_of_security.clone();
            }
            Self::ShareholdingPattern(m) => {
                fields.scripcode = m.scripcode.clone();
                fields.as_on_date = m.as_on_date;
                fields.promoter_and_group = m.promoter_and_group.clone();
                fields.public_val = m.public_val.clone();
                fields.emptr = m.emptr.clone();
                fields.status = m.status.clone();
                fields.submission_date = m.submission_date;
                fields.revised_filing_date = m.revised_filing_date;
            }
            Self::VotingResults(m) => {
                fields.scripcode = m.scripcode.clone();
                fields.meeting_date = m.meeting_date;
                fields.meeting_type = m.meeting_type.clone();
            }
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
        &[announcements::Column::Scripcode],
        db,
        backend,
        query,
        limit
    );
    search_sat!(
        ids,
        annual_reports,
        &[annual_reports::Column::Scripcode],
        db,
        backend,
        query,
        limit
    );
    search_sat!(
        ids,
        board_meetings,
        &[
            board_meetings::Column::Scripcode,
            board_meetings::Column::Purpose,
        ],
        db,
        backend,
        query,
        limit
    );
    search_sat!(
        ids,
        corporate_actions,
        &[
            corporate_actions::Column::Scripcode,
            corporate_actions::Column::Segment,
            corporate_actions::Column::Purpose,
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
            financial_results::Column::Scripcode,
            financial_results::Column::AuditedUnaudited,
            financial_results::Column::StandaloneConsolidated,
            financial_results::Column::IndAs,
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
            insider_trading::Column::Scripcode,
            insider_trading::Column::TypeOfSecurity,
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
            shareholding_pattern::Column::Scripcode,
            shareholding_pattern::Column::PromoterAndGroup,
            shareholding_pattern::Column::PublicVal,
            shareholding_pattern::Column::Emptr,
            shareholding_pattern::Column::Status,
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
            voting_results::Column::Scripcode,
            voting_results::Column::MeetingType,
        ],
        db,
        backend,
        query,
        limit
    );
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::list_filters::FilterKind;

    #[test]
    fn filter_fields_include_base_columns_for_every_feed() {
        for kind in BseFeedKind::ALL {
            let fields = filter_fields(*kind);
            assert_eq!(fields[0].key, "Title");
            assert_eq!(fields[0].kind, FilterKind::Text);
            assert_eq!(fields[1].key, "PubDate");
            assert_eq!(fields[1].kind, FilterKind::Date);
            let keys: Vec<_> = fields.iter().map(|f| f.key).collect();
            if kind.shows_description_column() {
                assert!(keys.contains(&"Description"));
                assert_eq!(fields.last().map(|f| f.key), Some("Description"));
            } else {
                assert!(!keys.contains(&"Description"));
            }
        }
    }

    #[test]
    fn filter_fields_include_extra_date_and_text_keys() {
        let sensex: Vec<_> = filter_fields(BseFeedKind::Sensex)
            .into_iter()
            .map(|f| f.key)
            .collect();
        assert_eq!(sensex, vec!["Title", "PubDate"]);

        let actions: Vec<_> = filter_fields(BseFeedKind::CorporateActions)
            .into_iter()
            .map(|f| (f.key, f.kind))
            .collect();
        assert!(actions.contains(&("Scripcode", FilterKind::Text)));
        assert!(actions.contains(&("RdDate", FilterKind::Date)));
        assert!(actions.contains(&("ActualPaymentDate", FilterKind::Date)));

        let announcements: Vec<_> = filter_fields(BseFeedKind::Announcements)
            .into_iter()
            .map(|f| f.key)
            .collect();
        assert!(announcements.contains(&"Description"));
        assert!(announcements.contains(&"Scripcode"));
    }
}
