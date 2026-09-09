use chrono::{DateTime, NaiveDate, Utc};
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
    #[sea_orm(column_type = "Text", nullable)]
    pub subject: Option<String>,
    pub as_on_date: Option<NaiveDate>,
    pub original_submission_date: Option<DateTime<Utc>>,
    #[sea_orm(column_type = "Text", nullable)]
    pub series: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub purpose: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub face_value: Option<String>,
    pub record_date: Option<NaiveDate>,
    pub book_closure_start_date: Option<NaiveDate>,
    pub book_closure_end_date: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub relating_to: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub audited_unaudited: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub cumulative: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub consolidated: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub ind_as: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub period: Option<String>,
    pub period_ended: Option<NaiveDate>,
    pub for_quarter_ending: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub encumbered_promoter_names: Option<String>,
    pub period_end_date: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub acquirer_names: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub promoter_names: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub financial_year: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub submission_type: Option<String>,
    pub meeting_date: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub remarks: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub brsr_nse_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub brsr_scrip_code: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub brsr_msei_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub brsr_isin: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub brsr_cin: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub brsr_company_name: Option<String>,
    pub brsr_date_of_incorporation: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub brsr_registered_office: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub brsr_corporate_office: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub brsr_email: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub brsr_telephone: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub brsr_website: Option<String>,
    pub brsr_fy_start: Option<NaiveDate>,
    pub brsr_fy_end: Option<NaiveDate>,
    pub brsr_py_start: Option<NaiveDate>,
    pub brsr_py_end: Option<NaiveDate>,
    pub brsr_ppy_start: Option<NaiveDate>,
    pub brsr_ppy_end: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub brsr_paid_up_capital: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub brsr_contact_person: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub brsr_contact_phone: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub brsr_contact_email: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub brsr_reporting_boundary: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub brsr_core_assurance: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub brsr_turnover: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub brsr_net_worth: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub brsr_states_served: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub brsr_countries_served: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub brsr_board_size: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub brsr_female_directors: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub brsr_kmp: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub brsr_female_kmp: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub brsr_csr_applicable: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub uhp_scrip_code: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub uhp_nse_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub uhp_msei_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub uhp_sebi_registration: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub uhp_company_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub uhp_type_of_report: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub uhp_number_of_securities: Option<String>,
    pub uhp_reporting_period_start: Option<NaiveDate>,
    pub uhp_date_of_report: Option<NaiveDate>,
    pub uhp_fy_start: Option<NaiveDate>,
    pub uhp_fy_end: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub voting_scrip_code: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub voting_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub voting_msei_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub voting_isin: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub voting_company_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub voting_type_of_meeting: Option<String>,
    pub voting_date_of_meeting: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub voting_start_time: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub voting_end_time: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub voting_scrutinizer: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub voting_scrutinizer_firm: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub voting_scrutinizer_qualification: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub voting_scrutinizer_membership: Option<String>,
    pub voting_board_meeting_date: Option<NaiveDate>,
    pub voting_report_issuance_date: Option<NaiveDate>,
    pub voting_record_date: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub voting_shareholders_on_record: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub voting_promoters_in_person: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub voting_public_in_person: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub voting_promoters_vc: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub voting_public_vc: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub voting_resolutions_passed: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub sod_nse_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub sod_scrip_code: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub sod_msei_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub sod_isin: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub sod_company_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub sod_statement_count: Option<String>,
    pub sod_quarter_ended: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub sod_mode_of_fund_raising: Option<String>,
    pub sod_date_of_funds_raising: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub sod_amount_raised: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub sod_monitoring_agency: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub sod_monitoring_agency_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub sod_has_deviation: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub sod_deviation_explanation: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub sod_shareholder_approved: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub sod_audit_committee_comments: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub sod_auditor_comments: Option<String>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub sod_objects: Option<Json>,
    #[sea_orm(column_type = "Text", nullable)]
    pub sod_signatory: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub sod_designation: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub sod_place: Option<String>,
    pub sod_date_of_signing: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub shp_nse_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub shp_scrip_code: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub shp_msei_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub shp_isin: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub shp_company_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub shp_class_of_security: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub shp_type_of_report: Option<String>,
    pub shp_date_of_report: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub shp_filed_under: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub shp_promoter_pct: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub shp_public_pct: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub shp_promoter_shares: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub shp_public_shares: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub scr_nse_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub scr_scrip_code: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub scr_msei_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub scr_isin: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub scr_company_name: Option<String>,
    pub scr_fy_start: Option<NaiveDate>,
    pub scr_fy_end: Option<NaiveDate>,
    pub scr_date_of_report: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub scr_observations_reported: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub scr_previous_observations: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub scr_actions_taken: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub scr_certifying_firm: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub scr_pcs_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub scr_membership_type: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub scr_membership_number: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub scr_udin: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub scr_cp_number: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub scr_place: Option<String>,
    pub scr_pcs_report_date: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub rpt_nse_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub rpt_scrip_code: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub rpt_msei_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub rpt_company_name: Option<String>,
    pub rpt_fy_start: Option<NaiveDate>,
    pub rpt_fy_end: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub rpt_reporting_period: Option<String>,
    pub rpt_period_start: Option<NaiveDate>,
    pub rpt_period_end: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub rpt_has_related_party: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub rpt_entered_transactions: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub rpt_transaction_count: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub rpt_counterparty: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub rpt_transaction_type: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub rpt_amount: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub ic_nse_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub ic_scrip_code: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub ic_msei_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub ic_isin: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub ic_company_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub ic_class: Option<String>,
    pub ic_period_end: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub ic_submission_type: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub ic_pending_start: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub ic_received: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub ic_disposed: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub ic_pending_end: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub ic_scores_id: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub it_nse_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub it_scrip_code: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub it_msei_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub it_isin: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub it_company_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub it_regulation: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub it_instrument: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub it_person: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub it_category: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub it_txn_type: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub it_qty: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub it_value: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub it_mode: Option<String>,
    pub it_from_date: Option<NaiveDate>,
    pub it_to_date: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub it_prior_qty: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub it_prior_pct: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub it_post_qty: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub it_post_pct: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub it_signatory: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub it_designation: Option<String>,
    pub it_filing_date: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub it_exchange: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub iff_nse_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub iff_scrip_code: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub iff_msei_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub iff_isin: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub iff_company_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub iff_type_of_company: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub iff_class_of_security: Option<String>,
    pub iff_fy_start: Option<NaiveDate>,
    pub iff_fy_end: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub iff_reporting_period: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub iff_reporting_quarter: Option<String>,
    pub iff_period_start: Option<NaiveDate>,
    pub iff_period_end: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub iff_audited: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub iff_nature: Option<String>,
    pub iff_board_meeting: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub iff_revenue: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub iff_profit: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub fr_nse_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub fr_scrip_code: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub fr_msei_symbol: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub fr_company_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub fr_class_of_security: Option<String>,
    pub fr_fy_start: Option<NaiveDate>,
    pub fr_fy_end: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub fr_reporting_quarter: Option<String>,
    pub fr_period_start: Option<NaiveDate>,
    pub fr_period_end: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub fr_audited: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub fr_nature: Option<String>,
    pub fr_board_meeting: Option<NaiveDate>,
    #[sea_orm(column_type = "Text", nullable)]
    pub fr_revenue: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub fr_profit: Option<String>,
    #[sea_orm(unique, column_type = "Text")]
    pub content_hash: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
