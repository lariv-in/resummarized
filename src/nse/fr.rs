//! Financial-results XBRL instance linked from the Financial Results RSS feed.

use chrono::NaiveDate;

use super::entities::financial_results::{
    ActiveModel as FrAM, Column as FrColumn, Model as FrModel,
};
use super::xbrl::{
    first_text_facts, opt_date, opt_str, set_opt, set_opt_date, take_clean, take_date,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FrFacts {
    pub nse_symbol: Option<String>,
    pub scrip_code: Option<String>,
    pub msei_symbol: Option<String>,
    pub company_name: Option<String>,
    pub class_of_security: Option<String>,
    pub fy_start: Option<NaiveDate>,
    pub fy_end: Option<NaiveDate>,
    pub reporting_quarter: Option<String>,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub audited: Option<String>,
    pub nature: Option<String>,
    pub board_meeting: Option<NaiveDate>,
    pub revenue: Option<String>,
    pub profit: Option<String>,
}

impl FrFacts {
    pub fn any(&self) -> bool {
        self.nse_symbol.is_some() || self.company_name.is_some()
    }

    pub fn apply(&self, am: &mut FrAM) {
        set_opt(&mut am.nse_symbol, &self.nse_symbol);
        set_opt(&mut am.scrip_code, &self.scrip_code);
        set_opt(&mut am.msei_symbol, &self.msei_symbol);
        set_opt(&mut am.company_name, &self.company_name);
        set_opt(&mut am.class_of_security, &self.class_of_security);
        set_opt_date(&mut am.fy_start, self.fy_start);
        set_opt_date(&mut am.fy_end, self.fy_end);
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

pub fn parse_fr_xbrl(xml: &str) -> FrFacts {
    let map = first_text_facts(xml);
    FrFacts {
        nse_symbol: take_clean(&map, "Symbol").or_else(|| take_clean(&map, "NSESymbol")),
        scrip_code: take_clean(&map, "ScripCode"),
        msei_symbol: take_clean(&map, "MSEISymbol"),
        company_name: take_clean(&map, "NameOfTheCompany"),
        class_of_security: take_clean(&map, "ClassOfSecurity"),
        fy_start: take_date(&map, "DateOfStartOfFinancialYear"),
        fy_end: take_date(&map, "DateOfEndOfFinancialYear"),
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
pub enum FrField {
    NseSymbol,
    ScripCode,
    MseiSymbol,
    CompanyName,
    ClassOfSecurity,
    FyStart,
    FyEnd,
    ReportingQuarter,
    PeriodStart,
    PeriodEnd,
    Audited,
    Nature,
    BoardMeeting,
    Revenue,
    Profit,
}

impl FrField {
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
        Self::ClassOfSecurity,
        Self::FyStart,
        Self::FyEnd,
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
            Self::NseSymbol => "FrNseSymbol",
            Self::ScripCode => "FrScripCode",
            Self::MseiSymbol => "FrMseiSymbol",
            Self::CompanyName => "FrCompanyName",
            Self::ClassOfSecurity => "FrClassOfSecurity",
            Self::FyStart => "FrFyStart",
            Self::FyEnd => "FrFyEnd",
            Self::ReportingQuarter => "FrReportingQuarter",
            Self::PeriodStart => "FrPeriodStart",
            Self::PeriodEnd => "FrPeriodEnd",
            Self::Audited => "FrAudited",
            Self::Nature => "FrNature",
            Self::BoardMeeting => "FrBoardMeeting",
            Self::Revenue => "FrRevenue",
            Self::Profit => "FrProfit",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::NseSymbol => "NSE symbol",
            Self::ScripCode => "Scrip code",
            Self::MseiSymbol => "MSEI symbol",
            Self::CompanyName => "Company",
            Self::ClassOfSecurity => "Class of security",
            Self::FyStart => "FY start",
            Self::FyEnd => "FY end",
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

    pub fn column(self) -> FrColumn {
        match self {
            Self::NseSymbol => FrColumn::NseSymbol,
            Self::ScripCode => FrColumn::ScripCode,
            Self::MseiSymbol => FrColumn::MseiSymbol,
            Self::CompanyName => FrColumn::CompanyName,
            Self::ClassOfSecurity => FrColumn::ClassOfSecurity,
            Self::FyStart => FrColumn::FyStart,
            Self::FyEnd => FrColumn::FyEnd,
            Self::ReportingQuarter => FrColumn::ReportingQuarter,
            Self::PeriodStart => FrColumn::PeriodStart,
            Self::PeriodEnd => FrColumn::PeriodEnd,
            Self::Audited => FrColumn::Audited,
            Self::Nature => FrColumn::Nature,
            Self::BoardMeeting => FrColumn::BoardMeeting,
            Self::Revenue => FrColumn::Revenue,
            Self::Profit => FrColumn::Profit,
        }
    }

    pub fn display(self, item: &FrModel) -> String {
        match self {
            Self::NseSymbol => opt_str(&item.nse_symbol),
            Self::ScripCode => opt_str(&item.scrip_code),
            Self::MseiSymbol => opt_str(&item.msei_symbol),
            Self::CompanyName => opt_str(&item.company_name),
            Self::ClassOfSecurity => opt_str(&item.class_of_security),
            Self::FyStart => opt_date(item.fy_start),
            Self::FyEnd => opt_date(item.fy_end),
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
}

#[cfg(test)]
mod tests {
    use super::parse_fr_xbrl;

    const FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<xbrli:xbrl xmlns:in-bse-fin="http://www.bseindia.com/xbrl/fin/2020-03-31/in-bse-fin" xmlns:xbrli="http://www.xbrl.org/2003/instance">
<in-bse-fin:ScripCode contextRef="OneI">533221</in-bse-fin:ScripCode>
<in-bse-fin:Symbol contextRef="OneI">AHLWEST</in-bse-fin:Symbol>
<in-bse-fin:MSEISymbol contextRef="OneI">NA</in-bse-fin:MSEISymbol>
<in-bse-fin:NameOfTheCompany contextRef="OneI">Asian Hotels (West) Limited</in-bse-fin:NameOfTheCompany>
<in-bse-fin:ClassOfSecurity contextRef="OneI">Equity</in-bse-fin:ClassOfSecurity>
<in-bse-fin:ReportingQuarter contextRef="OneI">Yearly</in-bse-fin:ReportingQuarter>
<in-bse-fin:WhetherResultsAreAuditedOrUnaudited contextRef="OneI">Audited</in-bse-fin:WhetherResultsAreAuditedOrUnaudited>
<in-bse-fin:NatureOfReportStandaloneConsolidated contextRef="OneI">Standalone</in-bse-fin:NatureOfReportStandaloneConsolidated>
<in-bse-fin:RevenueFromOperations contextRef="OneI">148987000.00</in-bse-fin:RevenueFromOperations>
<in-bse-fin:ProfitLossForPeriod contextRef="OneI">-157084000.00</in-bse-fin:ProfitLossForPeriod>
</xbrli:xbrl>
"#;

    #[test]
    fn parses_identity_and_headline_pnl() {
        let facts = parse_fr_xbrl(FIXTURE);
        assert_eq!(facts.nse_symbol.as_deref(), Some("AHLWEST"));
        assert_eq!(facts.reporting_quarter.as_deref(), Some("Yearly"));
        assert_eq!(facts.nature.as_deref(), Some("Standalone"));
        assert_eq!(facts.revenue.as_deref(), Some("148987000.00"));
        assert_eq!(facts.profit.as_deref(), Some("-157084000.00"));
    }
}
