//! Integrated-filing financials XBRL instance linked from that RSS feed.

use chrono::NaiveDate;

use super::entities::integrated_filing_financials::{
    ActiveModel as IffAM, Column as IffColumn, Model as IffModel,
};
use super::xbrl::{
    first_text_facts, opt_date, opt_str, set_opt, set_opt_date, take_clean, take_date,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IffFacts {
    pub nse_symbol: Option<String>,
    pub scrip_code: Option<String>,
    pub msei_symbol: Option<String>,
    pub isin: Option<String>,
    pub company_name: Option<String>,
    pub type_of_company: Option<String>,
    pub class_of_security: Option<String>,
    pub fy_start: Option<NaiveDate>,
    pub fy_end: Option<NaiveDate>,
    pub reporting_period: Option<String>,
    pub reporting_quarter: Option<String>,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub audited: Option<String>,
    pub nature: Option<String>,
    pub board_meeting: Option<NaiveDate>,
    pub revenue: Option<String>,
    pub profit: Option<String>,
}

impl IffFacts {
    pub fn any(&self) -> bool {
        self.nse_symbol.is_some() || self.company_name.is_some() || self.isin.is_some()
    }

    pub fn apply(&self, am: &mut IffAM) {
        set_opt(&mut am.nse_symbol, &self.nse_symbol);
        set_opt(&mut am.scrip_code, &self.scrip_code);
        set_opt(&mut am.msei_symbol, &self.msei_symbol);
        set_opt(&mut am.isin, &self.isin);
        set_opt(&mut am.company_name, &self.company_name);
        set_opt(&mut am.type_of_company, &self.type_of_company);
        set_opt(&mut am.class_of_security, &self.class_of_security);
        set_opt_date(&mut am.fy_start, self.fy_start);
        set_opt_date(&mut am.fy_end, self.fy_end);
        set_opt(&mut am.reporting_period, &self.reporting_period);
        set_opt(&mut am.reporting_quarter, &self.reporting_quarter);
        set_opt_date(&mut am.period_start, self.period_start);
        set_opt_date(&mut am.period_end, self.period_end);
        set_opt(&mut am.audited, &self.audited);
        set_opt(&mut am.nature, &self.nature);
        set_opt_date(&mut am.board_meeting, self.board_meeting);
        set_opt(&mut am.revenue, &self.revenue);
        set_opt(&mut am.profit, &self.profit);
    }
}

pub fn parse_iff_xbrl(xml: &str) -> IffFacts {
    let map = first_text_facts(xml);
    IffFacts {
        nse_symbol: take_clean(&map, "Symbol").or_else(|| take_clean(&map, "NSESymbol")),
        scrip_code: take_clean(&map, "ScripCode"),
        msei_symbol: take_clean(&map, "MSEISymbol"),
        isin: take_clean(&map, "ISIN"),
        company_name: take_clean(&map, "NameOfTheCompany"),
        type_of_company: take_clean(&map, "TypeOfCompany"),
        class_of_security: take_clean(&map, "ClassOfSecurity"),
        fy_start: take_date(&map, "DateOfStartOfFinancialYear"),
        fy_end: take_date(&map, "DateOfEndOfFinancialYear"),
        reporting_period: take_clean(&map, "TypeOfReportingPeriod"),
        reporting_quarter: take_clean(&map, "ReportingQuarter"),
        period_start: take_date(&map, "DateOfStartOfReportingPeriod"),
        period_end: take_date(&map, "DateOfEndOfReportingPeriod"),
        audited: take_clean(&map, "WhetherResultsAreAuditedOrUnaudited"),
        nature: take_clean(&map, "NatureOfReportStandaloneConsolidated"),
        board_meeting: take_date(&map, "DateOfBoardMeetingWhenFinancialResultsWereApproved"),
        revenue: take_clean(&map, "RevenueFromOperations"),
        profit: take_clean(&map, "ProfitLossForPeriod"),
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum IffField {
    NseSymbol,
    ScripCode,
    MseiSymbol,
    Isin,
    CompanyName,
    TypeOfCompany,
    ClassOfSecurity,
    FyStart,
    FyEnd,
    ReportingPeriod,
    ReportingQuarter,
    PeriodStart,
    PeriodEnd,
    Audited,
    Nature,
    BoardMeeting,
    Revenue,
    Profit,
}

impl IffField {
    pub const LIST: &'static [Self] = &[
        Self::NseSymbol,
        Self::ReportingQuarter,
        Self::Nature,
        Self::Revenue,
        Self::Profit,
    ];

    pub const DETAIL: &'static [Self] = &[
        Self::CompanyName,
        Self::NseSymbol,
        Self::ScripCode,
        Self::MseiSymbol,
        Self::Isin,
        Self::TypeOfCompany,
        Self::ClassOfSecurity,
        Self::FyStart,
        Self::FyEnd,
        Self::ReportingPeriod,
        Self::ReportingQuarter,
        Self::PeriodStart,
        Self::PeriodEnd,
        Self::Audited,
        Self::Nature,
        Self::BoardMeeting,
        Self::Revenue,
        Self::Profit,
    ];

    pub fn sort_key(self) -> &'static str {
        match self {
            Self::NseSymbol => "IffNseSymbol",
            Self::ScripCode => "IffScripCode",
            Self::MseiSymbol => "IffMseiSymbol",
            Self::Isin => "IffIsin",
            Self::CompanyName => "IffCompanyName",
            Self::TypeOfCompany => "IffTypeOfCompany",
            Self::ClassOfSecurity => "IffClassOfSecurity",
            Self::FyStart => "IffFyStart",
            Self::FyEnd => "IffFyEnd",
            Self::ReportingPeriod => "IffReportingPeriod",
            Self::ReportingQuarter => "IffReportingQuarter",
            Self::PeriodStart => "IffPeriodStart",
            Self::PeriodEnd => "IffPeriodEnd",
            Self::Audited => "IffAudited",
            Self::Nature => "IffNature",
            Self::BoardMeeting => "IffBoardMeeting",
            Self::Revenue => "IffRevenue",
            Self::Profit => "IffProfit",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::NseSymbol => "NSE symbol",
            Self::ScripCode => "Scrip code",
            Self::MseiSymbol => "MSEI symbol",
            Self::Isin => "ISIN",
            Self::CompanyName => "Company",
            Self::TypeOfCompany => "Type of company",
            Self::ClassOfSecurity => "Class of security",
            Self::FyStart => "FY start",
            Self::FyEnd => "FY end",
            Self::ReportingPeriod => "Reporting period",
            Self::ReportingQuarter => "Quarter",
            Self::PeriodStart => "Period start",
            Self::PeriodEnd => "Period end",
            Self::Audited => "Audited/Unaudited",
            Self::Nature => "Standalone/Consolidated",
            Self::BoardMeeting => "Board meeting",
            Self::Revenue => "Revenue",
            Self::Profit => "Profit",
        }
    }

    pub fn column(self) -> IffColumn {
        match self {
            Self::NseSymbol => IffColumn::NseSymbol,
            Self::ScripCode => IffColumn::ScripCode,
            Self::MseiSymbol => IffColumn::MseiSymbol,
            Self::Isin => IffColumn::Isin,
            Self::CompanyName => IffColumn::CompanyName,
            Self::TypeOfCompany => IffColumn::TypeOfCompany,
            Self::ClassOfSecurity => IffColumn::ClassOfSecurity,
            Self::FyStart => IffColumn::FyStart,
            Self::FyEnd => IffColumn::FyEnd,
            Self::ReportingPeriod => IffColumn::ReportingPeriod,
            Self::ReportingQuarter => IffColumn::ReportingQuarter,
            Self::PeriodStart => IffColumn::PeriodStart,
            Self::PeriodEnd => IffColumn::PeriodEnd,
            Self::Audited => IffColumn::Audited,
            Self::Nature => IffColumn::Nature,
            Self::BoardMeeting => IffColumn::BoardMeeting,
            Self::Revenue => IffColumn::Revenue,
            Self::Profit => IffColumn::Profit,
        }
    }

    pub fn display(self, item: &IffModel) -> String {
        match self {
            Self::NseSymbol => opt_str(&item.nse_symbol),
            Self::ScripCode => opt_str(&item.scrip_code),
            Self::MseiSymbol => opt_str(&item.msei_symbol),
            Self::Isin => opt_str(&item.isin),
            Self::CompanyName => opt_str(&item.company_name),
            Self::TypeOfCompany => opt_str(&item.type_of_company),
            Self::ClassOfSecurity => opt_str(&item.class_of_security),
            Self::FyStart => opt_date(item.fy_start),
            Self::FyEnd => opt_date(item.fy_end),
            Self::ReportingPeriod => opt_str(&item.reporting_period),
            Self::ReportingQuarter => opt_str(&item.reporting_quarter),
            Self::PeriodStart => opt_date(item.period_start),
            Self::PeriodEnd => opt_date(item.period_end),
            Self::Audited => opt_str(&item.audited),
            Self::Nature => opt_str(&item.nature),
            Self::BoardMeeting => opt_date(item.board_meeting),
            Self::Revenue => opt_str(&item.revenue),
            Self::Profit => opt_str(&item.profit),
        }
    }

    pub fn filter_kind(self) -> crate::list_filters::FilterKind {
        match self {
            Self::FyStart
            | Self::FyEnd
            | Self::PeriodStart
            | Self::PeriodEnd
            | Self::BoardMeeting => crate::list_filters::FilterKind::Date,
            _ => crate::list_filters::FilterKind::Text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_iff_xbrl;
    use chrono::NaiveDate;

    const FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<xbrli:xbrl xmlns:in-bse-fin="http://www.bseindia.com/xbrl/fin/2020-03-31/in-bse-fin" xmlns:xbrli="http://www.xbrl.org/2003/instance">
<in-bse-fin:ScripCode contextRef="OneI">542233</in-bse-fin:ScripCode>
<in-bse-fin:Symbol contextRef="OneI">TREJHARA</in-bse-fin:Symbol>
<in-bse-fin:MSEISymbol contextRef="OneI">NOTLISTED</in-bse-fin:MSEISymbol>
<in-bse-fin:ISIN contextRef="OneI">INE00CA01015</in-bse-fin:ISIN>
<in-bse-fin:NameOfTheCompany contextRef="OneI">TREJHARA SOLUTIONS LIMITED</in-bse-fin:NameOfTheCompany>
<in-bse-fin:TypeOfCompany contextRef="OneI">Main Board</in-bse-fin:TypeOfCompany>
<in-bse-fin:ClassOfSecurity contextRef="OneI">Equity</in-bse-fin:ClassOfSecurity>
<in-bse-fin:DateOfStartOfFinancialYear contextRef="OneI">2024-04-01</in-bse-fin:DateOfStartOfFinancialYear>
<in-bse-fin:DateOfEndOfFinancialYear contextRef="OneI">2025-03-31</in-bse-fin:DateOfEndOfFinancialYear>
<in-bse-fin:TypeOfReportingPeriod contextRef="OneI">Quarterly</in-bse-fin:TypeOfReportingPeriod>
<in-bse-fin:ReportingQuarter contextRef="OneI">Fourth quarter</in-bse-fin:ReportingQuarter>
<in-bse-fin:DateOfStartOfReportingPeriod contextRef="OneI">2025-01-01</in-bse-fin:DateOfStartOfReportingPeriod>
<in-bse-fin:DateOfEndOfReportingPeriod contextRef="OneI">2025-03-31</in-bse-fin:DateOfEndOfReportingPeriod>
<in-bse-fin:WhetherResultsAreAuditedOrUnaudited contextRef="OneI">Audited</in-bse-fin:WhetherResultsAreAuditedOrUnaudited>
<in-bse-fin:NatureOfReportStandaloneConsolidated contextRef="OneI">Standalone</in-bse-fin:NatureOfReportStandaloneConsolidated>
<in-bse-fin:DateOfBoardMeetingWhenFinancialResultsWereApproved contextRef="OneI">2025-05-30</in-bse-fin:DateOfBoardMeetingWhenFinancialResultsWereApproved>
<in-bse-fin:RevenueFromOperations contextRef="OneI">42435000</in-bse-fin:RevenueFromOperations>
<in-bse-fin:ProfitLossForPeriod contextRef="OneI">16673000</in-bse-fin:ProfitLossForPeriod>
</xbrli:xbrl>
"#;

    #[test]
    fn parses_identity_period_and_headline_pnl() {
        let facts = parse_iff_xbrl(FIXTURE);
        assert_eq!(facts.nse_symbol.as_deref(), Some("TREJHARA"));
        assert_eq!(facts.reporting_quarter.as_deref(), Some("Fourth quarter"));
        assert_eq!(facts.nature.as_deref(), Some("Standalone"));
        assert_eq!(facts.revenue.as_deref(), Some("42435000"));
        assert_eq!(facts.profit.as_deref(), Some("16673000"));
        assert_eq!(facts.board_meeting, NaiveDate::from_ymd_opt(2025, 5, 30));
    }
}
