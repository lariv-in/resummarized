//! REIT/InvIT unitholding-pattern XBRL instance linked from the Unitholding Patterns RSS feed.

use chrono::NaiveDate;
use std::collections::HashMap;

use super::entities::unitholding_patterns::{
    ActiveModel as UhpAM, Column as UhpColumn, Model as UhpModel,
};
use super::xbrl::{first_text_facts, opt_date, opt_str, set_opt, set_opt_date, take, take_date};

/// Document-level identity facts (not per-category holding tables).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UhpFacts {
    pub scrip_code: Option<String>,
    pub nse_symbol: Option<String>,
    pub msei_symbol: Option<String>,
    pub sebi_registration: Option<String>,
    pub company_name: Option<String>,
    pub type_of_report: Option<String>,
    pub number_of_securities: Option<String>,
    pub reporting_period_start: Option<NaiveDate>,
    pub date_of_report: Option<NaiveDate>,
    pub fy_start: Option<NaiveDate>,
    pub fy_end: Option<NaiveDate>,
}

impl UhpFacts {
    pub fn any(&self) -> bool {
        self.nse_symbol.is_some() || self.company_name.is_some() || self.sebi_registration.is_some()
    }

    pub fn apply(&self, am: &mut UhpAM) {
        set_opt(&mut am.scrip_code, &self.scrip_code);
        set_opt(&mut am.nse_symbol, &self.nse_symbol);
        set_opt(&mut am.msei_symbol, &self.msei_symbol);
        set_opt(&mut am.sebi_registration, &self.sebi_registration);
        set_opt(&mut am.company_name, &self.company_name);
        set_opt(&mut am.type_of_report, &self.type_of_report);
        set_opt(&mut am.number_of_securities, &self.number_of_securities);
        set_opt_date(&mut am.reporting_period_start, self.reporting_period_start);
        set_opt_date(&mut am.date_of_report, self.date_of_report);
        set_opt_date(&mut am.fy_start, self.fy_start);
        set_opt_date(&mut am.fy_end, self.fy_end);
    }
}

impl From<&HashMap<String, String>> for UhpFacts {
    fn from(map: &HashMap<String, String>) -> Self {
        Self {
            scrip_code: take(map, "ScripCode"),
            nse_symbol: take(map, "NSESymbol"),
            msei_symbol: take(map, "MSEISymbol"),
            sebi_registration: take(map, "SebiRegistrationNumber"),
            company_name: take(map, "NameOfTheCompany"),
            type_of_report: take(map, "TypeOfReportREITsINVITs"),
            number_of_securities: take(map, "NumberOfSecurities"),
            reporting_period_start: take_date(map, "DateOfStartOfReportingPeriod"),
            date_of_report: take_date(map, "DateOfReport"),
            fy_start: take_date(map, "DateOfStartOfFinancialYear"),
            fy_end: take_date(map, "DateOfEndOfFinancialYear"),
        }
    }
}

pub fn parse_uhp_xbrl(xml: &str) -> UhpFacts {
    UhpFacts::from(&first_text_facts(xml))
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum UhpField {
    NseSymbol,
    ScripCode,
    MseiSymbol,
    SebiRegistration,
    CompanyName,
    TypeOfReport,
    NumberOfSecurities,
    ReportingPeriodStart,
    DateOfReport,
    FyStart,
    FyEnd,
}

impl UhpField {
    pub const LIST: &'static [Self] = &[
        Self::NseSymbol,
        Self::TypeOfReport,
        Self::NumberOfSecurities,
    ];

    pub const DETAIL: &'static [Self] = &[
        Self::CompanyName,
        Self::NseSymbol,
        Self::ScripCode,
        Self::MseiSymbol,
        Self::SebiRegistration,
        Self::TypeOfReport,
        Self::NumberOfSecurities,
        Self::ReportingPeriodStart,
        Self::DateOfReport,
        Self::FyStart,
        Self::FyEnd,
    ];

    pub fn sort_key(self) -> &'static str {
        match self {
            Self::NseSymbol => "UhpNseSymbol",
            Self::ScripCode => "UhpScripCode",
            Self::MseiSymbol => "UhpMseiSymbol",
            Self::SebiRegistration => "UhpSebiRegistration",
            Self::CompanyName => "UhpCompanyName",
            Self::TypeOfReport => "UhpTypeOfReport",
            Self::NumberOfSecurities => "UhpNumberOfSecurities",
            Self::ReportingPeriodStart => "UhpReportingPeriodStart",
            Self::DateOfReport => "UhpDateOfReport",
            Self::FyStart => "UhpFyStart",
            Self::FyEnd => "UhpFyEnd",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::NseSymbol => "NSE symbol",
            Self::ScripCode => "Scrip code",
            Self::MseiSymbol => "MSEI symbol",
            Self::SebiRegistration => "SEBI registration",
            Self::CompanyName => "Trust",
            Self::TypeOfReport => "Type of report",
            Self::NumberOfSecurities => "Units outstanding",
            Self::ReportingPeriodStart => "Reporting period start",
            Self::DateOfReport => "Date of report",
            Self::FyStart => "FY start",
            Self::FyEnd => "FY end",
        }
    }

    pub fn column(self) -> UhpColumn {
        match self {
            Self::NseSymbol => UhpColumn::NseSymbol,
            Self::ScripCode => UhpColumn::ScripCode,
            Self::MseiSymbol => UhpColumn::MseiSymbol,
            Self::SebiRegistration => UhpColumn::SebiRegistration,
            Self::CompanyName => UhpColumn::CompanyName,
            Self::TypeOfReport => UhpColumn::TypeOfReport,
            Self::NumberOfSecurities => UhpColumn::NumberOfSecurities,
            Self::ReportingPeriodStart => UhpColumn::ReportingPeriodStart,
            Self::DateOfReport => UhpColumn::DateOfReport,
            Self::FyStart => UhpColumn::FyStart,
            Self::FyEnd => UhpColumn::FyEnd,
        }
    }

    pub fn display(self, item: &UhpModel) -> String {
        match self {
            Self::NseSymbol => opt_str(&item.nse_symbol),
            Self::ScripCode => opt_str(&item.scrip_code),
            Self::MseiSymbol => opt_str(&item.msei_symbol),
            Self::SebiRegistration => opt_str(&item.sebi_registration),
            Self::CompanyName => opt_str(&item.company_name),
            Self::TypeOfReport => opt_str(&item.type_of_report),
            Self::NumberOfSecurities => opt_str(&item.number_of_securities),
            Self::ReportingPeriodStart => opt_date(item.reporting_period_start),
            Self::DateOfReport => opt_date(item.date_of_report),
            Self::FyStart => opt_date(item.fy_start),
            Self::FyEnd => opt_date(item.fy_end),
        }
    }

    pub fn filter_kind(self) -> crate::list_filters::FilterKind {
        match self {
            Self::ReportingPeriodStart | Self::DateOfReport | Self::FyStart | Self::FyEnd => {
                crate::list_filters::FilterKind::Date
            }
            _ => crate::list_filters::FilterKind::Text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_uhp_xbrl;
    use chrono::NaiveDate;

    const FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<xbrli:xbrl xmlns:in-capmkt="https://www.sebi.gov.in/xbrl/2022-03-31/in-capmkt" xmlns:xbrli="http://www.xbrl.org/2003/instance">
<xbrli:context id="OneI"><xbrli:entity><xbrli:identifier scheme="http://www.nseindia.com/NSESymbol">INDIGRID</xbrli:identifier></xbrli:entity><xbrli:period><xbrli:instant>2026-06-30</xbrli:instant></xbrli:period></xbrli:context>
<in-capmkt:ScripCode contextRef="OneI">540565</in-capmkt:ScripCode>
<in-capmkt:NSESymbol contextRef="OneI">INDIGRID</in-capmkt:NSESymbol>
<in-capmkt:MSEISymbol contextRef="OneI">N.A.</in-capmkt:MSEISymbol>
<in-capmkt:SebiRegistrationNumber contextRef="OneI">IN/InvIT/16-17/0005</in-capmkt:SebiRegistrationNumber>
<in-capmkt:NameOfTheCompany contextRef="OneI">IndiGrid Infrastructure Trust</in-capmkt:NameOfTheCompany>
<in-capmkt:TypeOfReportREITsINVITs contextRef="OneI">Quarterly UHP</in-capmkt:TypeOfReportREITsINVITs>
<in-capmkt:NumberOfSecurities contextRef="OneI" unitRef="shares" decimals="INF">952564719</in-capmkt:NumberOfSecurities>
<in-capmkt:DateOfStartOfReportingPeriod contextRef="OneI">2026-04-01</in-capmkt:DateOfStartOfReportingPeriod>
<in-capmkt:DateOfReport contextRef="OneI">2026-06-30</in-capmkt:DateOfReport>
<in-capmkt:DateOfStartOfFinancialYear contextRef="OneI">2026-04-01</in-capmkt:DateOfStartOfFinancialYear>
<in-capmkt:DateOfEndOfFinancialYear contextRef="OneI">2027-03-31</in-capmkt:DateOfEndOfFinancialYear>
</xbrli:xbrl>
"#;

    #[test]
    fn parses_identity_and_ignores_context_identifier() {
        let facts = parse_uhp_xbrl(FIXTURE);
        assert_eq!(facts.nse_symbol.as_deref(), Some("INDIGRID"));
        assert_eq!(facts.scrip_code.as_deref(), Some("540565"));
        assert_eq!(facts.msei_symbol.as_deref(), Some("N.A."));
        assert_eq!(
            facts.sebi_registration.as_deref(),
            Some("IN/InvIT/16-17/0005")
        );
        assert_eq!(
            facts.company_name.as_deref(),
            Some("IndiGrid Infrastructure Trust")
        );
        assert_eq!(facts.type_of_report.as_deref(), Some("Quarterly UHP"));
        assert_eq!(facts.number_of_securities.as_deref(), Some("952564719"));
        assert_eq!(
            facts.reporting_period_start,
            NaiveDate::from_ymd_opt(2026, 4, 1)
        );
        assert_eq!(facts.date_of_report, NaiveDate::from_ymd_opt(2026, 6, 30));
        assert_eq!(facts.fy_start, NaiveDate::from_ymd_opt(2026, 4, 1));
        assert_eq!(facts.fy_end, NaiveDate::from_ymd_opt(2027, 3, 31));
    }
}
